use crate::core::ast::{BlockStatement, Expression, ExpressionKind, Program, Statement, StatementKind};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LifeCycleMap {
    pub last_uses: HashMap<usize, Vec<String>>,
    pub errors: Vec<String>,
}

pub struct Analyzer {
    last_uses: HashMap<String, usize>,
    lc_map: LifeCycleMap,
    type_checker: TypeChecker,
    in_loop: bool,
}

impl Analyzer {
    pub fn new() -> Self {
        Analyzer {
            last_uses: HashMap::new(),
            lc_map: LifeCycleMap::default(),
            type_checker: TypeChecker::new(),
            in_loop: false,
        }
    }

    pub fn analyze(&mut self, program: &Program) -> LifeCycleMap {
        self.traverse_program(program);
        let mut type_errors = self.type_checker.check(program);
        self.lc_map.errors.append(&mut type_errors);
        std::mem::take(&mut self.lc_map)
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
                self.last_uses.insert(name.clone(), index);
            }
            StatementKind::Expression(expr) => self.analyze_expression(expr, index),
            StatementKind::Return(expr) => self.analyze_expression(expr, index),
            StatementKind::If { condition, consequence, alternative } => {
                self.analyze_expression(condition, index);
                self.analyze_block(consequence, index);
                if let Some(alt) = alternative { self.analyze_block(alt, index); }
            }
            StatementKind::While { condition, body } => {
                self.in_loop = true;
                self.analyze_expression(condition, index);
                self.analyze_block(body, index);
                self.in_loop = false;
            }
            StatementKind::For { iterable, body, .. } => {
                self.in_loop = true;
                self.analyze_expression(iterable, index);
                self.analyze_block(body, index);
                self.in_loop = false;
            }
            _ => {}
        }
    }

    fn analyze_block(&mut self, block: &BlockStatement, index: usize) {
        for stmt in &block.statements {
            self.analyze_statement(stmt, index);
        }
    }

    fn analyze_expression(&mut self, expr: &Expression, index: usize) {
        match &expr.kind {
            ExpressionKind::Variable(name) => { self.last_uses.insert(name.clone(), index); }
            ExpressionKind::CallExpression { function, arguments } => {
                self.analyze_expression(function, index);
                for arg in arguments { self.analyze_expression(arg, index); }
            }
            ExpressionKind::MethodCallExpression { object, arguments, .. } => {
                self.analyze_expression(object, index);
                for arg in arguments { self.analyze_expression(arg, index); }
            }
            ExpressionKind::MemberExpression { object, .. } => { self.analyze_expression(object, index); }
            ExpressionKind::InfixExpression { left, right, .. } => {
                self.analyze_expression(left, index);
                self.analyze_expression(right, index);
            }
            ExpressionKind::PrefixExpression { right, .. } => { self.analyze_expression(right, index); }
            ExpressionKind::ArrayLiteral(elements) => {
                for el in elements { self.analyze_expression(el, index); }
            }
            ExpressionKind::MapLiteral(pairs) => {
                for (k, v) in pairs { self.analyze_expression(k, index); self.analyze_expression(v, index); }
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
            ExpressionKind::CallExpression { function, arguments } => {
                if let ExpressionKind::Identifier(name) = &function.kind {
                    if name == "shell_exec" || name == "system" {
                        self.lc_map.errors.push(format!("[Quantum Shield] Unsafe shell execution detected at line {}", "unknown"));
                    }
                }
                for arg in arguments { self.check_expression_security(arg); }
            }
            ExpressionKind::MethodCallExpression { object, method, arguments, .. } => {
                if let ExpressionKind::Identifier(name) = &object.kind {
                    if name == "db" && method == "query" {
                        for arg in arguments {
                            if let ExpressionKind::InfixExpression { .. } = &arg.kind {
                                self.lc_map.errors.push("[Quantum Shield] Possible SQL Injection: Concatenation in query detected.".into());
                            }
                        }
                    }
                }
                for arg in arguments { self.check_expression_security(arg); }
            }
            ExpressionKind::SqlQueryExpression { query, .. } => {
                let q_upper = query.to_uppercase();
                if q_upper.contains("DROP TABLE") || q_upper.contains("TRUNCATE") {
                    self.lc_map.errors.push("[Quantum Shield] High-risk SQL operation blocked: DROP/TRUNCATE.".into());
                }
            }
            _ => {}
        }
    }
}

pub struct TypeChecker {
    symbol_table: HashMap<String, String>,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker { symbol_table: HashMap::new() }
    }

    pub fn check(&mut self, program: &Program) -> Vec<String> {
        let mut errors = Vec::new();
        for stmt in &program.statements {
            match &stmt.kind {
                StatementKind::Let { name, value, var_type } => {
                    let val_type = self.infer_type(value);
                    if let Some(expected) = var_type {
                        if !self.types_match(expected, &val_type) {
                            errors.push(format!("Type mismatch: cannot assign {} to {} {}", val_type, expected, name));
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
            ExpressionKind::ArrayLiteral(_) => "array".into(),
            ExpressionKind::Variable(name) => self.symbol_table.get(name).cloned().unwrap_or("any".into()),
            _ => "any".into(),
        }
    }

    fn types_match(&self, expected: &str, actual: &str) -> bool {
        if expected == "any" || actual == "any" { return true; }
        expected == actual
    }
}
