use crate::core::ast::{BlockStatement, Expression, Program, Statement};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Default)]
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
        match stmt {
            Statement::Expression(expr) => self.check_security(expr),
            Statement::Let { value, .. } => self.check_security(value),
            _ => {}
        }
    }

    fn analyze_statement(&mut self, stmt: &Statement, index: usize) {
        match stmt {
            Statement::Let { name, value, .. } => {
                self.analyze_expression(value, index);
                self.last_uses.insert(name.clone(), index);
            }
            Statement::Expression(expr) => self.analyze_expression(expr, index),
            Statement::Return(expr) => self.analyze_expression(expr, index),
            Statement::If {
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
            Statement::While { condition, body } => {
                self.in_loop = true;
                self.analyze_expression(condition, index);
                self.analyze_block(body, index);
                self.in_loop = false;
            }
            Statement::For { iterable, body, .. } => {
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
        match expr {
            Expression::Variable(name) => {
                self.last_uses.insert(name.clone(), index);
            }
            Expression::CallExpression {
                function,
                arguments,
            } => {
                self.analyze_expression(function, index);
                for arg in arguments {
                    self.analyze_expression(arg, index);
                }
            }
            Expression::InfixExpression { left, right, .. } => {
                self.analyze_expression(left, index);
                self.analyze_expression(right, index);
            }
            _ => {}
        }
    }

    fn check_security(&mut self, expr: &Expression) {
        match expr {
            Expression::SqlQueryExpression { query, .. } => {
                if query.contains('$') || query.contains('{') {
                    self.lc_map.errors.push("Quantum Shield Alert: Potential SQL Injection detected. Use parameter binding instead of variable interpolation.".into());
                }
            }
            Expression::MethodCallExpression {
                object,
                method,
                arguments,
                ..
            } => {
                if let Expression::Identifier(name) = object.as_ref() {
                    if name == "file" && (method == "read" || method == "write") {
                        if let Some(Expression::StringLiteral { value, .. }) = arguments.get(0) {
                            if value.contains("..") {
                                self.lc_map.errors.push("Quantum Shield Alert: Potential Path Traversal detected in file access.".into());
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ZenithType {
    Int,
    String,
    Bool,
    Array,
    Map,
    Any,
    Void,
}

impl From<&str> for ZenithType {
    fn from(s: &str) -> Self {
        match s {
            "int" => ZenithType::Int,
            "string" => ZenithType::String,
            "bool" => ZenithType::Bool,
            "array" => ZenithType::Array,
            "map" => ZenithType::Map,
            "void" => ZenithType::Void,
            _ => ZenithType::Any,
        }
    }
}

pub struct TypeChecker {
    pub symbols: HashMap<String, ZenithType>,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            symbols: HashMap::new(),
        }
    }

    pub fn check(&mut self, program: &Program) -> Vec<String> {
        let mut errors = Vec::new();
        for stmt in &program.statements {
            self.check_statement(stmt, &mut errors);
        }
        errors
    }

    fn check_statement(&mut self, stmt: &Statement, errors: &mut Vec<String>) {
        match stmt {
            Statement::Let {
                name,
                value,
                var_type,
            } => {
                let inferred = self.infer_type(value);
                if let Some(target_str) = var_type {
                    let target = ZenithType::from(target_str.as_str());
                    if target != ZenithType::Any && !self.is_compatible(&target, &inferred) {
                        errors.push(format!(
                            "Type mismatch: cannot assign {:?} to variable {} of type {:?}",
                            inferred, name, target
                        ));
                    }
                    self.symbols.insert(name.clone(), target);
                } else {
                    self.symbols.insert(name.clone(), inferred);
                }
            }
            Statement::FunctionDefinition {
                parameters, body, ..
            } => {
                for param in parameters {
                    let p_type = param
                        .param_type
                        .as_deref()
                        .map(ZenithType::from)
                        .unwrap_or(ZenithType::Any);
                    self.symbols.insert(param.name.clone(), p_type);
                }
                for s in &body.statements {
                    self.check_statement(s, errors);
                }
            }
            _ => {}
        }
    }

    pub fn infer_type(&self, expr: &Expression) -> ZenithType {
        match expr {
            Expression::IntegerLiteral(_) => ZenithType::Int,
            Expression::StringLiteral { .. } => ZenithType::String,
            Expression::Variable(name) => {
                self.symbols.get(name).cloned().unwrap_or(ZenithType::Any)
            }
            Expression::InfixExpression {
                left,
                operator,
                right,
            } => {
                let l_type = self.infer_type(left);
                let r_type = self.infer_type(right);
                match operator.as_str() {
                    "+" | "-" | "*" | "/" => {
                        if l_type == ZenithType::Int && r_type == ZenithType::Int {
                            ZenithType::Int
                        } else {
                            ZenithType::Any
                        }
                    }
                    "==" | "!=" | "<" | ">" => ZenithType::Bool,
                    _ => ZenithType::Any,
                }
            }
            _ => ZenithType::Any,
        }
    }

    fn is_compatible(&self, target: &ZenithType, actual: &ZenithType) -> bool {
        target == actual || *target == ZenithType::Any || *actual == ZenithType::Any
    }
}

#[derive(Serialize, Deserialize)]
pub struct SchemaMetadata {
    pub tables: HashMap<String, TableMetadata>,
}

#[derive(Serialize, Deserialize)]
pub struct TableMetadata {
    pub columns: HashMap<String, String>,
}

pub struct SchemaManager {
    schema: Option<SchemaMetadata>,
}

impl SchemaManager {
    pub fn new(path: &str) -> Self {
        let schema = fs::read_to_string(path)
            .ok()
            .and_then(|data| serde_json::from_str(&data).ok());

        SchemaManager { schema }
    }

    pub fn validate_query(&self, table: &str, columns: &[String]) -> Vec<String> {
        let mut errors = Vec::new();
        if let Some(schema) = &self.schema {
            if let Some(tbl) = schema.tables.get(table) {
                for col in columns {
                    if col != "*" && !tbl.columns.contains_key(col) {
                        errors.push(format!(
                            "Column '{}' does not exist in table '{}'",
                            col, table
                        ));
                    }
                }
            } else {
                errors.push(format!("Table '{}' does not exist in schema", table));
            }
        }
        errors
    }
}
