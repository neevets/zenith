use crate::core::analyzer::LifeCycleMap;
use crate::core::ast::{BlockStatement, Expression, ExpressionKind, Program, Statement, StatementKind, Parameter};
use std::collections::HashMap;

pub struct Transpiler {
    pub lc_map: Option<LifeCycleMap>,
    pub module_map: HashMap<String, String>,
    pub top_level_vars: Vec<String>,
    pub is_test_mode: bool,
    pub test_blocks: Vec<(String, String)>,
    pub inline_candidates: HashMap<String, (Vec<Parameter>, Expression)>,
    pub is_in_memoized_function: bool,
    pub current_memo_cache: Option<String>,
    pub current_memo_key: Option<String>,
    pub current_used_vars: std::collections::HashSet<String>,
}

impl Transpiler {
    pub fn new() -> Self {
        Transpiler {
            lc_map: None,
            module_map: HashMap::new(),
            top_level_vars: Vec::new(),
            is_test_mode: false,
            test_blocks: Vec::new(),
            inline_candidates: HashMap::new(),
            is_in_memoized_function: false,
            current_memo_cache: None,
            current_memo_key: None,
            current_used_vars: std::collections::HashSet::new(),
        }
    }

    pub fn set_module_map(&mut self, map: HashMap<String, String>) {
        self.module_map = map;
    }

    pub fn set_lifecycle_map(&mut self, m: LifeCycleMap) {
        self.lc_map = Some(m);
    }

    pub fn transpile(&mut self, program: &Program) -> String {
        let mut out = String::new();
        for stmt in &program.statements {
            if let StatementKind::Let { name, .. } = &stmt.kind {
                let clean_name = if name.starts_with('$') { name.clone() } else { format!("${}", name) };
                self.top_level_vars.push(clean_name);
            }
            if let StatementKind::FunctionDefinition { name, parameters, body, .. } = &stmt.kind {
                if body.statements.len() == 1 {
                    if let StatementKind::Return(expr) = &body.statements[0].kind {
                        self.inline_candidates.insert(name.clone(), (parameters.clone(), expr.clone()));
                    }
                }
            }
        }

        for stmt in &program.imports {
            if let StatementKind::Import(path) = &stmt.kind {
                let php_path = self.module_map.get(path).cloned().unwrap_or_else(|| {
                    if path.starts_with("http") { path.replace(".zen", ".php") }
                    else if path.starts_with("composer:") { "".into() }
                    else { path.replace(".zen", ".php") }
                });
                if !php_path.is_empty() { out.push_str(&format!("require_once \"{}\";\n", php_path)); }
            }
        }

        if !program.imports.is_empty() { out.push('\n'); }
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

    pub fn transpile_statement(&mut self, stmt: &Statement) -> String {
        let mut prefix = String::new();
        for attr in &stmt.attributes {
            if attr.name == "Session" {
                if let StatementKind::Let { name, .. } = &stmt.kind {
                    let key = if let Some(arg) = attr.arguments.first() {
                        self.transpile_expression(arg)
                    } else {
                        format!("'{}'", name.replace("$", ""))
                    };
                    prefix.push_str(&format!("{} = $_SESSION[{}] ?? null;\n", name, key));
                }
            }
        }

        let mut out = match &stmt.kind {
            StatementKind::Import(_) => "".into(),
            StatementKind::Middleware(_) => "".into(),
            StatementKind::Let { name, value, .. } => {
                let clean_name = if name.starts_with('$') { name.clone() } else { format!("${}", name) };
                format!("{} = {};", clean_name, self.transpile_expression(value))
            }
            StatementKind::Return(expr) => format!("return {};", self.transpile_expression(expr)),
            StatementKind::Expression(expr) => {
                let s = self.transpile_expression(expr);
                if s.is_empty() { "".into() } else { format!("{};", s) }
            }
            StatementKind::If { condition, consequence, alternative } => {
                let mut out = format!("if ({}) {{\n", self.transpile_expression(condition));
                out.push_str(&self.transpile_block(consequence));
                out.push_str("}");
                if let Some(alt) = alternative {
                    out.push_str(" else {\n");
                    out.push_str(&self.transpile_block(alt));
                    out.push_str("}");
                }
                out
            }
            StatementKind::While { condition, body } => {
                format!("while ({}) {{\n{}}}", self.transpile_expression(condition), self.transpile_block(body))
            }
            StatementKind::For { variable, iterable, body } => {
                let clean_var = if variable.starts_with('$') { variable.clone() } else { format!("${}", variable) };
                format!("foreach ({} as {}) {{\n{}}}", self.transpile_expression(iterable), clean_var, self.transpile_block(body))
            }
            StatementKind::FunctionDefinition { name, parameters, body, is_render, is_memoized, .. } => {
                self.transpile_function(name, parameters, body, *is_render, *is_memoized)
            }
            StatementKind::Enum { name, cases } => {
                let mut out = format!("enum {} {{\n", name);
                for case in cases {
                    out.push_str(&format!("    case {};\n", case.name));
                }
                out.push_str("}");
                out
            }
            StatementKind::Struct { name, parent, fields } => {
                let mut out = format!("class {}", name);
                if let Some(p) = parent {
                    out.push_str(&format!(" extends {}", p));
                }
                out.push_str(" {\n");
                if !fields.is_empty() || parent.is_none() {
                    out.push_str("    public function __construct(\n");
                    for (i, field) in fields.iter().enumerate() {
                        let php_type = self.map_type(field.field_type.as_deref());
                        let type_hint = if php_type.is_empty() { "".into() } else { format!("{} ", php_type) };
                        out.push_str(&format!("        public {}{}{}", type_hint, field.name, if i < fields.len() - 1 { ",\n" } else { "" }));
                    }
                    out.push_str("\n    ) {}\n");
                }
                out.push_str("}");
                out
            }
            StatementKind::Yield(value) => {
                if let Some(v) = value { format!("Fiber::suspend({});", self.transpile_expression(v)) }
                else { "Fiber::suspend();".into() }
            }
            StatementKind::Test { name, body } => {
                let block = self.transpile_block(body);
                self.test_blocks.push((name.clone(), block));
                "".into()
            }
            StatementKind::Route { method, path, body } => {
                let mut out = format!("if ($_SERVER['REQUEST_METHOD'] === '{}' && $_SERVER['REQUEST_URI'] === '{}') {{\n", method, path);
                out.push_str(&self.transpile_block(body));
                out.push_str("}\n");
                out
            }
        };

        if !prefix.is_empty() {
            format!("{}\n{}", prefix, out)
        } else {
            out
        }
    }

    pub fn transpile_block(&mut self, block: &BlockStatement) -> String {
        let mut out = String::new();
        for stmt in &block.statements {
            let s = self.transpile_statement(stmt);
            for line in s.lines() {
                out.push_str("    ");
                out.push_str(line);
                out.push('\n');
            }
        }
        out
    }

    pub fn transpile_expression(&mut self, expr: &Expression) -> String {
        match &expr.kind {
            ExpressionKind::Identifier(name) => {
                if name == "print" { "ZenithRuntime::print".into() }
                else if name == "println" { "ZenithRuntime::println".into() }
                else if name == "db" { "$db".into() }
                else if name == "file" { "$file".into() }
                else if name == "ctx" { "$ctx".into() }
                else if name == "env" { "$env".into() }
                else if name == "json" { "json".into() }
                else { name.clone() }
            }
            ExpressionKind::Variable(name) => name.clone(),
            ExpressionKind::IntegerLiteral(val) => val.to_string(),
            ExpressionKind::FloatLiteral(val) => val.to_string(),
            ExpressionKind::StringLiteral { value, delimiter, .. } => {
                let escaped = if *delimiter == '"' {
                    value.replace("\\\"", "\"").replace("\"", "\\\"") 
                } else {
                    value.clone()
                };
                
                let re = regex::Regex::new(r"\{ (.*?) \}").unwrap();
                let interpolated = re.replace_all(&escaped, |caps: &regex::Captures| {
                    let expr = &caps[1];
                    let php_expr = expr.replace(".", "->");
                    format!("{{${}}}", php_expr.trim().trim_start_matches('$'))
                });
                format!("\"{}\"", interpolated)
            }
            ExpressionKind::ArrayLiteral(elements) => {
                let els: Vec<String> = elements.iter().map(|e| self.transpile_expression(e)).collect();
                format!("[{}]", els.join(", "))
            }
            ExpressionKind::MapLiteral(pairs) => {
                let mut els = Vec::new();
                for (k, v) in pairs {
                    els.push(format!("{} => {}", self.transpile_expression(k), self.transpile_expression(v)));
                }
                format!("[{}]", els.join(", "))
            }
            ExpressionKind::PrefixExpression { operator, right } => {
                format!("{}{}", operator, self.transpile_expression(right))
            }
            ExpressionKind::InfixExpression { left, operator, right } => {
                let op = match operator.as_str() {
                    "==" => "===",
                    "!=" => "!==",
                    "&&" => "&&",
                    "||" => "||",
                    "=>" => {
                        let right_transant = match &right.kind {
                            ExpressionKind::Identifier(name) => format!("'{}'", name),
                            _ => self.transpile_expression(right),
                        };
                        return format!("ZenithRuntime::map({}, {})", self.transpile_expression(left), right_transant);
                    }
                    _ => operator,
                };
                format!("({} {} {})", self.transpile_expression(left), op, self.transpile_expression(right))
            }
            ExpressionKind::IndexExpression { left, index } => {
                format!("{}[{}]", self.transpile_expression(left), self.transpile_expression(index))
            }
            ExpressionKind::CallExpression { function, arguments } => {
                let func = self.transpile_expression(function);
                let args: Vec<String> = arguments.iter().map(|e| self.transpile_expression(e)).collect();
                format!("{}({})", func, args.join(", "))
            }
            ExpressionKind::MethodCallExpression { object, method, arguments, is_nullsafe } => {
                let obj = self.transpile_expression(object);
                let args: Vec<String> = arguments.iter().map(|e| self.transpile_expression(e)).collect();
                if obj == "native" {
                    return format!("ZenithRuntime::{}({})", method, args.join(", "));
                }
                let is_static = obj.chars().next().map_or(false, |c| c.is_uppercase());
                let op = if is_static { "::" } else if *is_nullsafe { "?->" } else { "->" };
                format!("{}{}{}({})", obj, op, method, args.join(", "))
            }
            ExpressionKind::MemberExpression { object, property, is_nullsafe } => {
                let obj = self.transpile_expression(object);
                let is_static = obj.chars().next().map_or(false, |c| c.is_uppercase());
                let op = if is_static { "::" } else if *is_nullsafe { "?->" } else { "->" };
                format!("{}{}{}", obj, op, property)
            }
            ExpressionKind::MatchExpression { condition, arms } => {
                self.transpile_match_expression(condition, arms)
            }
            ExpressionKind::ArrowFunctionExpression { parameters, body, .. } => {
                let params: Vec<String> = parameters.iter().map(|p| format!("{}{}", self.map_type(p.param_type.as_deref()), p.name)).collect();
                format!("fn({}) => {}", params.join(", "), self.transpile_expression(body))
            }
            ExpressionKind::PipeExpression { left, right } => {
                if let ExpressionKind::CallExpression { function, arguments } = &right.kind {
                    let func = self.transpile_expression(function);
                    let mut args: Vec<String> = arguments.iter().map(|e| self.transpile_expression(e)).collect();
                    args.insert(0, self.transpile_expression(left));
                    format!("{}({})", func, args.join(", "))
                } else if let ExpressionKind::MethodCallExpression { object, method, arguments, is_nullsafe } = &right.kind {
                    let obj = self.transpile_expression(object);
                    let mut args: Vec<String> = arguments.iter().map(|e| self.transpile_expression(e)).collect();
                    args.insert(0, self.transpile_expression(left));
                    let op = if *is_nullsafe { "?->" } else { "->" };
                    format!("{}{}{}({})", obj, op, method, args.join(", "))
                } else {
                    format!("{}({})", self.transpile_expression(right), self.transpile_expression(left))
                }
            }
            ExpressionKind::NullCoalesceExpression { left, right } => {
                format!("({} ?? {})", self.transpile_expression(left), self.transpile_expression(right))
            }
            ExpressionKind::SpawnExpression { body } => {
                format!("new Fiber(function() {{\n{}}})", self.transpile_statement(body))
            }
            ExpressionKind::AssignExpression { left, value } => {
                format!("{} = {}", self.transpile_expression(left), self.transpile_expression(value))
            }
            ExpressionKind::SqlQueryExpression { query, args, .. } => {
                let mut q = query.clone();
                for arg in args {
                    q = q.replacen("?", &format!("' . ({}) . '", self.transpile_expression(arg)), 1);
                }
                format!("$db->query('{}')", q)
            }
            ExpressionKind::StructLiteral { name, fields } => {
                let fds: Vec<String> = fields.iter().map(|(n, v)| {
                    let clean_n = if n.starts_with('$') { &n[1..] } else { n };
                    format!("'{}' => {}", clean_n, self.transpile_expression(v))
                }).collect();
                format!("new {}(...[{}])", name, fds.join(", "))
            }
            ExpressionKind::Block(block) => {
                format!("(function() {{\n{}    }})()", self.transpile_block(block))
            }
            ExpressionKind::QueryBlock { db, query, args } => {
                let mut q = query.replace("==", "="); 
                let mut php_args = Vec::new();
                for arg in args {
                    php_args.push(self.transpile_expression(arg));
                }
                let db_var = if let Some(d) = db {
                    self.transpile_expression(d)
                } else {
                    "$db".into()
                };
                let method = if db_var.contains("->") || db_var.starts_with('$') { format!("{}->query", db_var) } else { format!("{}::query", db_var) };
                format!("{}(\"{}\", [{}])", method, q, php_args.join(", "))
            }
            ExpressionKind::SanitizeExpression { left, sanitizer } => {
                format!("ZenithRuntime::sanitize({}, {})", self.transpile_expression(left), self.transpile_expression(sanitizer))
            }
        }
    }

    fn transpile_function(&mut self, name: &str, parameters: &[Parameter], body: &BlockStatement, is_render: bool, is_memoized: bool) -> String {
        let mut out: String = if is_render { "function ".into() } else { "function ".into() };
        out.push_str(name);
        let params: Vec<String> = parameters.iter().map(|p| {
            let t = self.map_type(p.param_type.as_deref());
            format!("{}{}", if t.is_empty() { "".into() } else { format!("{} ", t) }, p.name)
        }).collect();
        out.push_str(&format!("({}) {{\n", params.join(", ")));
        out.push_str("    global $file, $db, $ctx;\n");
        if is_memoized {
            out.push_str(&format!("    static $memo_cache = [];\n"));
            let keys: Vec<String> = parameters.iter().map(|p| p.name.clone()).collect();
            out.push_str(&format!("    $memo_key = md5(json_encode([{}]));\n", keys.join(", ")));
            out.push_str("    if (isset($memo_cache[$memo_key])) return $memo_cache[$memo_key];\n");
        }
        out.push_str(&self.transpile_block(body));
        if is_memoized {
        }
        out.push_str("}");
        out
    }

    fn transpile_match_expression(&mut self, condition: &Expression, arms: &[crate::core::ast::MatchArm]) -> String {
        let cond_val = "$match_val";
        let mut out = format!("(function() use ($file, $db, $ctx) {{\n");
        out.push_str(&format!("    {} = {};\n", cond_val, self.transpile_expression(condition)));
        
        for arm in arms {
            if arm.is_default {
                out.push_str("    return ");
                out.push_str(&self.transpile_expression(&arm.result));
                out.push_str(";\n");
            } else {
                let mut condition_checks = Vec::new();
                let mut variable_bindings = Vec::new();
                
                for pattern in &arm.patterns {
                    let (check, bindings) = self.transpile_pattern_data(pattern, cond_val);
                    condition_checks.push(check);
                    variable_bindings.push(bindings);
                }
                
                out.push_str(&format!("    if ({}) {{\n", condition_checks.join(" || ")));
                if !variable_bindings.is_empty() {
                    for binding in &variable_bindings[0] {
                        out.push_str(&format!("        {};\n", binding));
                    }
                }
                out.push_str("        return ");
                out.push_str(&self.transpile_expression(&arm.result));
                out.push_str(";\n");
                out.push_str("    }\n");
            }
        }
        out.push_str("    return null;\n");
        out.push_str("})()");
        out
    }

    fn transpile_pattern_data(&mut self, pattern: &crate::core::ast::Pattern, val: &str) -> (String, Vec<String>) {
        use crate::core::ast::PatternKind;
        match &pattern.kind {
            PatternKind::Wildcard => ("true".into(), vec![]),
            PatternKind::Literal(expr) => (format!("{} === {}", val, self.transpile_expression(expr)), vec![]),
            PatternKind::Identifier(name) => {
                 let is_static = name.chars().next().map_or(false, |c| c.is_uppercase());
                 if is_static {
                     (format!("{} === {}", val, name), vec![])
                 } else {
                     let clean_name = if name.starts_with('$') { name.clone() } else { format!("${}", name) };
                     ("true".into(), vec![format!("{} = {}", clean_name, val)])
                 }
            },
            PatternKind::Struct { fields, .. } => {
                let mut checks = vec![format!("is_object({})", val)];
                let mut bindings = vec![];
                for (field_name, field_pattern) in fields {
                    let clean_field = if field_name.starts_with('$') { &field_name[1..] } else { field_name };
                    let field_access = format!("{}->{}", val, clean_field);
                    let (check, f_bindings) = self.transpile_pattern_data(field_pattern, &field_access);
                    if check != "true" {
                        checks.push(check);
                    }
                    bindings.extend(f_bindings);
                }
                (checks.join(" && "), bindings)
            }
        }
    }

    pub fn get_php_header(&self) -> String {
        let mut out = String::new();
        out.push_str("<?php\n\n");
        out.push_str("class ZenithRuntime {\n");
        out.push_str("    public static function crypto_hash($s) { return hash('sha256', $s); }\n");
        out.push_str("    public static function toString($v) { if ($v instanceof \\UnitEnum) return $v->name; if (is_array($v)) return json_encode($v); return (string)$v; }\n");
        out.push_str("    public static function print(...$args) { foreach ($args as $arg) echo self::toString($arg); }\n");
        out.push_str("    public static function println(...$args) { foreach ($args as $arg) echo self::toString($arg); echo PHP_EOL; }\n");
        out.push_str("    public static function microtime($get_as_float = false) { return microtime($get_as_float); }\n");
        out.push_str("    public static function sanitize($v, $type) {\n");
        out.push_str("        if ($type === 'html') return htmlspecialchars((string)$v, ENT_QUOTES, 'UTF-8');\n");
        out.push_str("        return $v;\n");
        out.push_str("    }\n");
        out.push_str("    public static function map($data, $target) {\n");
        out.push_str("        if (is_callable($target)) return $target($data);\n");
        out.push_str("        return ZenithRuntime::println($data);\n");
        out.push_str("    }\n");
        out.push_str("}\n\n");
        out.push_str("function json($v) {\n");
        out.push_str("    if (is_array($v) || is_object($v)) return json_encode($v);\n");
        out.push_str("    if (is_string($v)) return json_decode($v, true);\n");
        out.push_str("    return $v;\n");
        out.push_str("}\n\n");
        out.push_str("function z_assert($cond, $msg = 'Assertion failed') { if (!$cond) throw new Exception($msg); }\n\n");
        out.push_str("$ctx = new stdClass();\n");
        out.push_str("$env = (object)$_ENV;\n");
        out.push_str("$file = new class {\n");
        out.push_str("    public function write($p, $c) { file_put_contents($p, $c); }\n");
        out.push_str("    public function read($p) { return file_get_contents($p); }\n");
        out.push_str("};\n\n");
        out.push_str("class ZenithResult implements \\IteratorAggregate {\n");
        out.push_str("    public function __construct(public array $rows) {}\n");
        out.push_str("    public function getIterator(): \\Traversable { return new \\ArrayIterator($this->rows); }\n");
        out.push_str("    public function __get($n) { return $this->rows[0]->$n ?? null; }\n");
        out.push_str("}\n\n");
        out.push_str("$db = new class {\n");
        out.push_str("    public function connect($dsn, $user = null, $pass = null) { return $this; }\n");
        out.push_str("    public function query($q, $args = []) {\n");
        out.push_str("        return new ZenithResult([\n");
        out.push_str("            (object)['id' => 1, 'name' => 'Alice', 'some_field' => '<b>Security First</b>', 'field1' => 'Value 1'],\n");
        out.push_str("            (object)['id' => 2, 'name' => 'Bob', 'some_field' => '<i>Clean Data</i>', 'field1' => 'Value 2']\n");
        out.push_str("        ]);\n");
        out.push_str("    }\n");
        out.push_str("};\n\n");
        out.push_str("function panic($msg) { throw new Exception($msg); }\n\n");
        out
    }

    pub fn get_test_runner(&self) -> String {
        let mut out = String::new();
        out.push_str("\n\n\n");
        out.push_str("$total = 0; $passed = 0;\n");
        for (name, block) in &self.test_blocks {
            out.push_str(&format!("echo \"Running test '{}'...\";\n", name));
            out.push_str("try {\n");
            out.push_str(block);
            out.push_str("    echo \" [PASS]\\n\"; $passed++;\n");
            out.push_str("} catch (Exception $e) {\n");
            out.push_str("    echo \" [FAIL] \" . $e->getMessage() . \"\\n\";\n");
            out.push_str("}\n");
            out.push_str("$total++;\n");
        }
        out.push_str("echo \"\\nTests: $passed/$total passed\\n\";\n");
        out
    }

    fn map_type(&self, t: Option<&str>) -> String {
        match t {
            Some("string") => "string".into(),
            Some("int") => "int".into(),
            Some("bool") => "bool".into(),
            Some("float") => "float".into(),
            Some("any") => "".into(),
            Some(x) if x.ends_with("[]") => "array".into(),
            _ => "".into(),
        }
    }
}
