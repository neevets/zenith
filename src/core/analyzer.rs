use crate::core::ast::{
    BlockStatement, Expression, ExpressionKind, Program, Statement, StatementKind,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LifeCycleMap {
    pub last_uses: HashMap<usize, Vec<String>>,
    pub errors: Vec<String>,
}

pub struct Analyzer {
    symbols: std::collections::HashSet<String>,
    last_uses: HashMap<String, usize>,
    lc_map: LifeCycleMap,
    type_checker: TypeChecker,
    in_loop: bool,
    re_sql_inj: Regex,
    re_sql_hard: Regex,
    re_path_trav: Regex,
}

impl Analyzer {
    pub fn new() -> Self {
        let mut symbols = std::collections::HashSet::new();
        for builtin in &[
            "db",
            "file",
            "ctx",
            "env",
            "println",
            "print",
            "json",
            "panic",
            "z_assert",
            "assertTrue",
            "true",
            "false",
            "null",
        ] {
            symbols.insert(builtin.to_string());
            symbols.insert(format!("${}", builtin));
        }

        Analyzer {
            symbols,
            last_uses: HashMap::new(),
            lc_map: LifeCycleMap::default(),
            type_checker: TypeChecker::new(),
            in_loop: false,
            re_sql_inj: Regex::new(r"(?i)(\bOR\b\s+.+?=|[;']|--|\bUNION\b\s+\bSELECT\b|\bINSERT\b\s+\bINTO\b|\bUPDATE\b\s+.+?\bSET\b|\bDELETE\b\s+\bFROM\b)").unwrap(),
            re_sql_hard: Regex::new(r"(?i)\b(DROP|TRUNCATE|ALTER)\b\s+\bTABLE\b").unwrap(),
            re_path_trav: Regex::new(r"(\.\./|\.\.\\|/etc/|C:\\|[a-zA-Z0-9_\-]+/(\.\./)+)").unwrap(),
        }
    }

    pub fn analyze(&mut self, program: &Program) -> LifeCycleMap {
        self.collect_definitions(program);
        self.traverse_program(program);
        let type_errors = self.type_checker.check(program);

        for err in type_errors {
            let msg = if program.is_strict {
                format!("[Strict] {}", err)
            } else {
                format!("Warning: {}", err)
            };
            self.lc_map.errors.push(msg);
        }

        if program.is_strict && !self.lc_map.errors.is_empty() {
            for err in &mut self.lc_map.errors {
                if !err.starts_with("[Strict]") && !err.starts_with("[Quantum Shield]") {
                    *err = format!("[Strict] {}", err);
                }
            }
        }

        std::mem::take(&mut self.lc_map)
    }

    fn collect_definitions(&mut self, program: &Program) {
        for stmt in &program.statements {
            match &stmt.kind {
                StatementKind::FunctionDefinition { name, .. } => {
                    self.symbols.insert(name.clone());
                }
                StatementKind::Enum { name, .. } => {
                    self.symbols.insert(name.clone());
                }
                StatementKind::Struct { name, .. } => {
                    self.symbols.insert(name.clone());
                }
                _ => {}
            }
        }
    }

    fn traverse_program(&mut self, program: &Program) {
        for (i, stmt) in program.statements.iter().enumerate() {
            self.analyze_statement_with_security(stmt, i);
        }
    }

    fn analyze_statement_with_security(&mut self, stmt: &Statement, index: usize) {
        self.analyze_statement(stmt, index);
        self.check_statement_security(stmt);
    }

    fn analyze_statement(&mut self, stmt: &Statement, index: usize) {
        match &stmt.kind {
            StatementKind::Let { name, value, .. } => {
                self.analyze_expression(value, index);
                self.symbols.insert(name.clone());
                self.last_uses.insert(name.clone(), index);
            }
            StatementKind::Expression(expr) => self.analyze_expression(expr, index),
            StatementKind::Return(expr) => self.analyze_expression(expr, index),
            StatementKind::If {
                condition,
                consequence,
                alternative,
            } => {
                self.analyze_expression(condition, index);
                self.analyze_block(consequence, index);
                if let Some(alt) = alternative {
                    self.analyze_block(alt, index);
                }
            }
            StatementKind::While { condition, body } => {
                self.in_loop = true;
                self.analyze_expression(condition, index);
                self.analyze_block(body, index);
                self.in_loop = false;
            }
            StatementKind::For {
                variable,
                iterable,
                body,
            } => {
                self.in_loop = true;
                self.analyze_expression(iterable, index);
                self.symbols.insert(variable.clone());
                self.analyze_block(body, index);
                self.in_loop = false;
            }
            StatementKind::FunctionDefinition {
                parameters, body, ..
            } => {
                for param in parameters {
                    self.symbols.insert(param.name.clone());
                }
                self.analyze_block(body, index);
            }
            StatementKind::TryCatch {
                try_block,
                catch_clauses,
                finally_block,
            } => {
                self.analyze_block(try_block, index);
                for clause in catch_clauses {
                    self.symbols.insert(clause.variable.clone());
                    self.analyze_block(&clause.body, index);
                }
                if let Some(finally) = finally_block {
                    self.analyze_block(finally, index);
                }
            }
            StatementKind::Test { body, .. } | StatementKind::Route { body, .. } => {
                self.analyze_block(body, index);
            }
            _ => {}
        }
    }

    fn analyze_block(&mut self, block: &BlockStatement, index: usize) {
        for stmt in &block.statements {
            self.analyze_statement_with_security(stmt, index);
        }
    }

    fn analyze_expression(&mut self, expr: &Expression, index: usize) {
        match &expr.kind {
            ExpressionKind::Identifier(name) => {
                if !self.symbols.contains(name) && !name.contains('.') && !name.contains('\\') {
                    self.lc_map
                        .errors
                        .push(format!("Undefined symbol: {}", name));
                }
            }
            ExpressionKind::Variable(name) => {
                if !self.symbols.contains(name) {
                    self.lc_map
                        .errors
                        .push(format!("Undefined variable: {}", name));
                }
                self.last_uses.insert(name.clone(), index);
            }
            ExpressionKind::CallExpression {
                function,
                arguments,
            } => {
                self.analyze_expression(function, index);
                for arg in arguments {
                    self.analyze_expression(arg, index);
                }
            }
            ExpressionKind::MethodCallExpression {
                object, arguments, ..
            } => {
                self.analyze_expression(object, index);
                for arg in arguments {
                    self.analyze_expression(arg, index);
                }
            }
            ExpressionKind::InfixExpression { left, right, .. } => {
                self.analyze_expression(left, index);
                self.analyze_expression(right, index);
            }
            ExpressionKind::PrefixExpression { right, .. } => {
                self.analyze_expression(right, index);
            }
            ExpressionKind::ArrayLiteral(els) => {
                for el in els {
                    self.analyze_expression(el, index);
                }
            }
            _ => {}
        }
    }

    fn check_statement_security(&mut self, stmt: &Statement) {
        match &stmt.kind {
            StatementKind::Expression(expr) => self.check_expression_security(expr),
            _ => {}
        }
    }

    fn check_expression_security(&mut self, expr: &Expression) {
        match &expr.kind {
            ExpressionKind::CallExpression {
                function,
                arguments,
            } => {
                if let ExpressionKind::Identifier(name) = &function.kind {
                    let unsafe_funcs = [
                        "shell_exec",
                        "system",
                        "passthru",
                        "exec",
                        "popen",
                        "proc_open",
                    ];
                    if unsafe_funcs.contains(&name.as_str()) {
                        self.lc_map.errors.push(format!(
                            "[Quantum Shield] Unsafe execution blocked: {} is restricted.",
                            name
                        ));
                    }
                }
                for arg in arguments {
                    self.check_expression_security(arg);
                }
            }
            ExpressionKind::MethodCallExpression {
                object,
                method,
                arguments,
                ..
            } => {
                if let ExpressionKind::Identifier(name) = &object.kind {
                    if name == "db" && (method == "query" || method == "execute") {
                        for arg in arguments {
                            if self.is_dynamic_sql(arg) {
                                self.lc_map.errors.push("[Quantum Shield] Possible SQL Injection: Avoid dynamic SQL construction. Use parameterized blocks.".into());
                            }
                        }
                    }
                    if name == "file"
                        && (method == "read"
                            || method == "write"
                            || method == "append"
                            || method == "delete")
                    {
                        for arg in arguments {
                            if self.is_suspicious_path(arg) {
                                self.lc_map.errors.push(format!(
                                    "[Quantum Shield] Path Traversal detected in {}.",
                                    method
                                ));
                            }
                        }
                    }
                }
                for arg in arguments {
                    self.check_expression_security(arg);
                }
            }
            ExpressionKind::SqlQueryExpression { query, .. } => {
                if self.re_sql_hard.is_match(query) {
                    self.lc_map.errors.push(
                        "[Quantum Shield] Forbidden SQL operation: DROP/TRUNCATE/ALTER blocked."
                            .into(),
                    );
                }
                if query.contains(" + ") || query.contains(" . ") || query.contains("$$") {
                    self.lc_map.errors.push("[Quantum Shield] Possible SQL Injection: Dynamic construction in query block.".into());
                }
            }
            _ => {}
        }
    }

    fn is_dynamic_sql(&self, expr: &Expression) -> bool {
        match &expr.kind {
            ExpressionKind::StringLiteral { value, .. } => self.re_sql_inj.is_match(value),
            ExpressionKind::InfixExpression { .. } | ExpressionKind::Variable(_) => true,
            _ => false,
        }
    }

    fn is_suspicious_path(&self, expr: &Expression) -> bool {
        match &expr.kind {
            ExpressionKind::StringLiteral { value, .. } => self.re_path_trav.is_match(value),
            ExpressionKind::InfixExpression { .. } | ExpressionKind::Variable(_) => true,
            _ => false,
        }
    }
}

pub struct TypeChecker {
    symbol_table: HashMap<String, String>,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            symbol_table: HashMap::new(),
        }
    }

    pub fn check(&mut self, program: &Program) -> Vec<String> {
        let mut errors = Vec::new();
        for stmt in &program.statements {
            match &stmt.kind {
                StatementKind::Let {
                    name,
                    value,
                    var_type,
                } => {
                    let val_type = self.infer_type(value);
                    if let Some(expected) = var_type {
                        if expected != &val_type && expected != "any" && val_type != "any" {
                            errors.push(format!(
                                "Type mismatch: cannot assign {} to {} {}",
                                val_type, expected, name
                            ));
                        }
                    }
                    self.symbol_table.insert(name.clone(), val_type);
                }
                _ => {}
            }
        }
        errors
    }

    fn infer_type(&self, expr: &Expression) -> String {
        match &expr.kind {
            ExpressionKind::IntegerLiteral(_) => "int".into(),
            ExpressionKind::FloatLiteral(_) => "float".into(),
            ExpressionKind::StringLiteral { .. } => "string".into(),
            ExpressionKind::Variable(name) => {
                self.symbol_table.get(name).cloned().unwrap_or("any".into())
            }
            _ => "any".into(),
        }
    }
}
