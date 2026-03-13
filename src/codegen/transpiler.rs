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

    fn transpile_statement(&self, stmt: &Statement) -> String {
        match stmt {
            Statement::Let { name, value, .. } => {
                format!("${} = {};", name, self.transpile_expression(value))
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

    fn transpile_expression(&self, expr: &Expression) -> String {
        match expr {
            Expression::Identifier(val) => val.clone(),
            Expression::Variable(val) => format!("${}", val),
            Expression::IntegerLiteral(val) => val.to_string(),
            Expression::StringLiteral { value, delimiter, .. } => {
                format!("{}{}{}", delimiter, value, delimiter)
            }
            Expression::InfixExpression { left, operator, right } => {
                let op = if operator == "+" { "." } else { operator };
                format!("({} {} {})", self.transpile_expression(left), op, self.transpile_expression(right))
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
            _ => String::new(),
        }
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
