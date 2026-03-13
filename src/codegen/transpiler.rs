use crate::core::ast::{Program, Statement, Expression, BlockStatement};
use crate::core::analyzer::LifeCycleMap;

pub struct Transpiler {
    pub lc_map: Option<LifeCycleMap>,
}

impl Transpiler {
    pub fn new() -> Self {
        Transpiler { lc_map: None }
    }

    pub fn set_lifecycle_map(&mut self, m: LifeCycleMap) {
        self.lc_map = Some(m);
    }

    pub fn transpile(&self, program: &Program) -> String {
        let mut out = String::new();

        for stmt in &program.imports {
            if let Statement::Import(path) = stmt {
                let php_path = path.replace(".zen", ".php");
                out.push_str(&format!("require_once \"{}\";\n", php_path));
            }
        }

        if !program.imports.is_empty() {
            out.push('\n');
        }

        if let Some(middleware) = &program.middleware {
            out.push_str(&self.transpile_block(middleware));
            out.push('\n');
        }

        for stmt in &program.statements {
            out.push_str(&self.transpile_statement(stmt));
            out.push('\n');
        }

        out
    }

    pub fn transpile_statement(&self, stmt: &Statement) -> String {
        match stmt {
            Statement::Let { name, value, .. } => {
                let clean_name = name.trim_start_matches('$');
                format!("${} = {};", clean_name, self.transpile_expression(value))
            }
            Statement::Return(expr) => {
                format!("return {};", self.transpile_expression(expr))
            }
            Statement::Expression(expr) => {
                format!("{};", self.transpile_expression(expr))
            }
            Statement::If { condition, consequence, alternative } => {
                let mut out = format!("if ({}) {{\n{}}}", self.transpile_expression(condition), self.transpile_block(consequence));
                if let Some(alt) = alternative {
                    out.push_str(&format!(" else {{\n{}}}", self.transpile_block(alt)));
                }
                out
            }
            Statement::While { condition, body } => {
                format!("while ({}) {{\n{}}}", self.transpile_expression(condition), self.transpile_block(body))
            }
            Statement::For { variable, iterable, body } => {
                let it = self.transpile_expression(iterable);
                let var = if variable.starts_with('$') { variable.clone() } else { format!("${}", variable) };
                format!("foreach ({} as {}) {{\n{}}}", it, var, self.transpile_block(body))
            }
            Statement::FunctionDefinition { name, parameters, body, return_type, .. } => {
                let mut p_list = Vec::new();
                for p in parameters {
                    let mut s = String::new();
                    if let Some(ref t) = p.param_type {
                        if t != "any" {
                            s.push_str(t);
                            s.push(' ');
                        }
                    }
                    let name = if p.name.starts_with('$') { p.name.clone() } else { format!("${}", p.name) };
                    s.push_str(&name);
                    p_list.push(s);
                }
                let ret = if let Some(ref t) = return_type {
                    if t == "any" { "".into() } else { format!(": {}", t) }
                } else { "".into() };
                format!("function {}({}){} {{\n    global $file, $db, $ctx, $db_file;\n{}}}", name, p_list.join(", "), ret, self.transpile_block(body))
            }
            Statement::Enum { name, cases } => {
                let mut out = format!("enum {} {{\n", name);
                for case in cases {
                    out.push_str(&format!("    case {}", case.name));
                    if let Some(ref val) = case.value {
                        out.push_str(&format!(" = {}", self.transpile_expression(val)));
                    }
                    out.push_str(";\n");
                }
                out.push_str("}\n");
                out
            }
            Statement::Struct { name, fields } => {
                let mut out = format!("class {} {{\n", name);
                for field in fields {
                    let mut mod_str = String::from("public");
                    if field.is_readonly {
                        mod_str.push_str(" readonly");
                    }
                    let typ = field.field_type.as_ref().map(|t| format!("{} ", t)).unwrap_or_default();
                    out.push_str(&format!("    {} {}${};\n", mod_str, typ, field.name));
                }
                out.push_str("}\n");
                out
            }
            Statement::Yield(val) => {
                let v = val.as_ref().map(|e| self.transpile_expression(e)).unwrap_or_default();
                format!("Fiber::suspend({});", v)
            }
            _ => String::new(),
        }
    }

    fn transpile_block(&self, block: &BlockStatement) -> String {
        let mut out = String::new();
        for stmt in &block.statements {
            out.push_str("    ");
            out.push_str(&self.transpile_statement(stmt));
            out.push('\n');
        }
        out
    }

    pub fn transpile_expression(&self, expr: &Expression) -> String {
        match expr {
            Expression::Identifier(val) => val.clone(),
            Expression::Variable(val) => {
                let clean_val = val.trim_start_matches('$');
                format!("${}", clean_val)
            }
            Expression::IntegerLiteral(val) => val.to_string(),
            Expression::StringLiteral { value, delimiter, is_render } => {
                let mut val = value.clone();
                if *is_render && *delimiter == '"' {
                    val = self.apply_xss_protection(&val);
                }
                format!("{}{}{}", delimiter, val, delimiter)
            }
            Expression::InfixExpression { left, operator, right } => {
                let op = if operator == "+" { "." } else { operator };
                format!("({} {} {})", self.transpile_expression(left), op, self.transpile_expression(right))
            }
            Expression::PrefixExpression { operator, right } => {
                format!("({}{})", operator, self.transpile_expression(right))
            }
            Expression::CallExpression { function, arguments } => {
                let func_name = self.transpile_expression(function);
                let args: Vec<String> = arguments.iter().map(|a| self.transpile_expression(a)).collect();
                if func_name == "print" {
                    format!("echo {}", args.join(", "))
                } else {
                    format!("{}({})", func_name, args.join(", "))
                }
            }
            Expression::MethodCallExpression { object, method, arguments, is_nullsafe } => {
                let obj = self.transpile_expression(object);
                let args: Vec<String> = arguments.iter().map(|a| self.transpile_expression(a)).collect();
                let op = if *is_nullsafe { "?->" } else { "->" };
                match method.as_str() {
                    "length" => format!("strlen({})", obj),
                    "push" => format!("array_push({}, {})", obj, args.join(", ")),
                    "parse" if obj == "json" => format!("json_decode({}, true)", args.join(", ")),
                    "stringify" if obj == "json" => format!("json_encode({})", args.join(", ")),
                    _ => {
                        let mut use_obj = obj;
                        if use_obj == "file" { use_obj = "$file".into(); }
                        if use_obj == "$ctx" {
                            match method.as_str() {
                                "query" => if let Some(a) = args.get(0) { return format!("$_GET[{}]", a); } else { return "$_GET".into(); },
                                "body" => if let Some(a) = args.get(0) { return format!("$_POST[{}]", a); } else { return "$_POST".into(); },
                                _ => {}
                            }
                        }
                        format!("{}{}{}({})", use_obj, op, method, args.join(", "))
                    }
                }
            }
            Expression::MemberExpression { object, property, is_nullsafe } => {
                let obj = self.transpile_expression(object);
                let op = if *is_nullsafe { "?->" } else { "->" };
                if obj == "$ctx" {
                    match property.as_str() {
                        "query" => return "$_GET".into(),
                        "body" => return "$_POST".into(),
                        _ => {}
                    }
                }
                format!("{}{}{}", obj, op, property)
            }
            Expression::ArrayLiteral(elements) => {
                let elms: Vec<String> = elements.iter().map(|e| self.transpile_expression(e)).collect();
                format!("[{}]", elms.join(", "))
            }
            Expression::MapLiteral(pairs) => {
                let p: Vec<String> = pairs.iter().map(|(k, v)| format!("{} => {}", self.transpile_expression(k), self.transpile_expression(v))).collect();
                format!("[{}]", p.join(", "))
            }
            Expression::IndexExpression { left, index } => {
                format!("{}[{}]", self.transpile_expression(left), self.transpile_expression(index))
            }
            Expression::NullCoalesceExpression { left, right } => {
                format!("({} ?? {})", self.transpile_expression(left), self.transpile_expression(right))
            }
            Expression::AssignExpression { left, value } => {
                format!("{} = {}", self.transpile_expression(left), self.transpile_expression(value))
            }
            Expression::MatchExpression { condition, arms } => {
                let mut out = format!("match ({}) {{\n", self.transpile_expression(condition));
                for arm in arms {
                    out.push_str("        ");
                    if arm.is_default {
                        out.push_str("default");
                    } else {
                        let vals: Vec<String> = arm.values.iter().map(|v| self.transpile_expression(v)).collect();
                        out.push_str(&vals.join(", "));
                    }
                    let result_str = match &arm.result {
                        Expression::Block(b) => {
                            let block_code = self.transpile_block(b);
                            format!("(function() use ($file, $db, $ctx) {{\n            {}\n        }})()", block_code.trim())
                        }
                        _ => self.transpile_expression(&arm.result),
                    };
                    out.push_str(&format!(" => {},\n", result_str));
                }
                out.push_str("    }");
                out
            }
            Expression::SqlQueryExpression { query, args, .. } => {
                let trans_args: Vec<String> = args.iter().map(|a| self.transpile_expression(a)).collect();
                let _use_vars = vec!["$db"];
                // This is simplified but mirrors the Go logic
                let use_clause = if !trans_args.is_empty() {
                    let mut v = trans_args.clone();
                    v.push("$db".into());
                    v.join(", ")
                } else {
                    "$db".into()
                };
                format!("(function() use ({}) {{ try {{ $stmt = $db->prepare(\"{}\"); $stmt->execute([{}]); return $stmt->fetchAll(); }} catch (Exception $e) {{ return null; }} }})()",
                    use_clause, query, trans_args.join(", "))
            }
            Expression::PipeExpression { left, right } => {
                 let lhs = self.transpile_expression(left);
                 // Similar to Go, we need to handle if right is a call or just an identifier
                 match right.as_ref() {
                     Expression::CallExpression { function, arguments } => {
                         let mut args = vec![lhs];
                         args.extend(arguments.iter().map(|a| self.transpile_expression(a)));
                         format!("{}({})", self.transpile_expression(function), args.join(", "))
                     }
                     _ => format!("{}({})", self.transpile_expression(right), lhs),
                 }
            }
            Expression::ArrowFunctionExpression { parameters, body, return_type } => {
                let mut p_list = Vec::new();
                for p in parameters {
                    let mut s = String::new();
                    if let Some(ref t) = p.param_type {
                        if t != "any" {
                            s.push_str(t);
                            s.push(' ');
                        }
                    }
                    let name = if p.name.starts_with('$') { p.name.clone() } else { format!("${}", p.name) };
                    s.push_str(&name);
                    p_list.push(s);
                }
                let ret = return_type.as_ref().map(|t| format!(": {}", t)).unwrap_or_default();
                format!("fn({}){} => {}", p_list.join(", "), ret, self.transpile_expression(body))
            }
            Expression::SpawnExpression { body } => {
                let b = self.transpile_statement(body);
                format!("(function() use ($file, $db, $ctx) {{ $f = new Fiber(function() use ($file, $db, $ctx) {{ {} }}); $f->start(); return $f; }})()", b)
            }
            Expression::Block(b) => {
                let block_code = self.transpile_block(b);
                format!("(function() use ($file, $db, $ctx) {{\n    {}\n}})()", block_code.trim())
            }
        }
    }

    fn apply_xss_protection(&self, input: &str) -> String {
        // Simple implementation of the bracket-based XSS protection from Go
        // This is a placeholder for the logic that finds {expr} and wraps it in htmlspecialchars
        input.to_string()
    }

    pub fn get_php_header(&self) -> String {
        let mut out = String::from("<?php\n\n");
        out.push_str("if (!class_exists('Context')) { class Context { public $path; public $query; public $body; } }\n\n");
        
        let functions = vec![
            ("fetch", "function fetch($url) {\n    $opts = [\"http\" => [\"header\" => \"User-Agent: ZenithRuntime/1.0\\r\\n\"]];\n    return file_get_contents($url, false, stream_context_create($opts));\n}"),
            ("json", "function json($data) {\n    return is_string($data) ? json_decode($data, true) : json_encode($data);\n}"),
            ("env", "function env($key) {\n    return getenv($key);\n}"),
            ("println", "function println($data) {\n    echo $data . \"\\n\";\n}"),
            ("redirect", "function redirect($url) {\n    header(\"Location: \" . $url);\n    exit;\n}"),
            ("z_assert", "function z_assert($condition, $message = \"Assertion failed\") {\n    if ($condition) {\n        echo \"  [OK] Pass: \" . $message . \"\\n\";\n    } else {\n        echo \"  [FAIL] FAIL: \" . $message . \"\\n\";\n        exit(1);\n    }\n}"),
        ];

        for (name, body) in functions {
            out.push_str(&format!("if (!function_exists('{}')) {{\n{}\n}}\n\n", name, body));
        }

        out.push_str("class ZenithFile {\n    public function read($path) { return file_get_contents($path); }\n    public function write($path, $data) { return file_put_contents($path, $data); }\n}\n");
        out.push_str("$file = new ZenithFile();\n");
        out.push_str("$ctx = new Context();\n");
        out.push_str("$ctx->path = parse_url($_SERVER['REQUEST_URI'] ?? '/', PHP_URL_PATH);\n");
        out.push_str("$ctx->query = $_GET ?? [];\n");
        out.push_str("$ctx->body = $_POST ?? [];\n\n");

        out
    }
}
