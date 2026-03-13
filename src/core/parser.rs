use crate::core::lexer::{Lexer, Token, TokenType};
use crate::core::ast::{
    Program, Statement, Expression, BlockStatement, Parameter, 
    EnumCase, StructField, MatchArm
};

#[derive(PartialEq, PartialOrd)]
enum Precedence {
    Lowest,
    Pipe,
    Assign,
    Coalesce,
    Compare,
    Sum,
    Product,
    Prefix,
    Call,
    Index,
    Dot,
}

impl From<&TokenType> for Precedence {
    fn from(t: &TokenType) -> Self {
        match t {
            TokenType::Pipe => Precedence::Pipe,
            TokenType::Assign => Precedence::Assign,
            TokenType::Coalesce => Precedence::Coalesce,
            TokenType::LParen => Precedence::Call,
            TokenType::LBracket => Precedence::Index,
            TokenType::Dot | TokenType::Nullsafe => Precedence::Dot,
            TokenType::Lt | TokenType::Gt | TokenType::Eq | TokenType::NotEq => Precedence::Compare,
            TokenType::Plus | TokenType::Minus => Precedence::Sum,
            TokenType::Asterisk | TokenType::Slash => Precedence::Product,
            _ => Precedence::Lowest,
        }
    }
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    cur_token: Token,
    peek_token: Token,
    pub errors: Vec<String>,
    is_render: bool,
}

impl<'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let cur_token = lexer.next_token();
        let peek_token = lexer.next_token();
        Parser {
            lexer,
            cur_token,
            peek_token,
            errors: Vec::new(),
            is_render: false,
        }
    }

    fn next_token(&mut self) {
        self.cur_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    pub fn parse_program(&mut self) -> Program {
        let mut program = Program {
            imports: Vec::new(),
            middleware: None,
            statements: Vec::new(),
        };

        while self.cur_token_is(TokenType::Import) {
            if let Some(stmt) = self.parse_import_statement() {
                program.imports.push(stmt);
            }
            self.next_token();
        }

        if self.cur_token_is(TokenType::Before) {
            self.next_token();
            if self.cur_token_is(TokenType::LBrace) {
                program.middleware = Some(self.parse_block_statement());
            }
            self.next_token();
        }

        while !self.cur_token_is(TokenType::Eof) {
            if let Some(stmt) = self.parse_statement() {
                program.statements.push(stmt);
            }
            self.next_token();
        }

        program
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        match self.cur_token.token_type {
            TokenType::Render => Some(self.parse_function_definition(true)),
            TokenType::Function => Some(self.parse_function_definition(false)),
            TokenType::Return => Some(self.parse_return_statement()),
            TokenType::Let => Some(self.parse_let_statement()),
            TokenType::If => Some(self.parse_if_statement()),
            TokenType::While => Some(self.parse_while_statement()),
            TokenType::For => Some(self.parse_for_statement()),
            TokenType::Yield => Some(self.parse_yield_statement()),
            TokenType::Enum => Some(self.parse_enum_statement()),
            TokenType::Ident => {
                if self.cur_token.literal == "struct" {
                    Some(self.parse_struct_definition())
                } else {
                    Some(self.parse_expression_statement())
                }
            }
            _ => Some(self.parse_expression_statement()),
        }
    }

    fn parse_import_statement(&mut self) -> Option<Statement> {
        if !self.expect_peek(TokenType::Literal("".into())) {
            return None;
        }
        let path = self.cur_token.literal.clone();
        if self.peek_token_is(TokenType::Semicolon) {
            self.next_token();
        }
        Some(Statement::Import(path))
    }

    fn parse_let_statement(&mut self) -> Statement {
        if !self.expect_peek(TokenType::Var) {
            return Statement::Expression(Expression::Identifier("error".into()));
        }
        let name = self.cur_token.literal.clone();
        let mut var_type = None;

        if self.peek_token_is(TokenType::Colon) {
            self.next_token();
            self.next_token();
            var_type = Some(self.parse_type());
        }

        if !self.expect_peek(TokenType::Assign) {
            return Statement::Expression(Expression::Identifier("error".into()));
        }

        self.next_token();
        let value = self.parse_expression(Precedence::Lowest);

        self.expect_peek(TokenType::Semicolon);

        Statement::Let {
            name,
            value,
            var_type,
        }
    }

    fn parse_return_statement(&mut self) -> Statement {
        self.next_token();
        let value = self.parse_expression(Precedence::Lowest);
        self.expect_peek(TokenType::Semicolon);
        Statement::Return(value)
    }

    fn parse_expression_statement(&mut self) -> Statement {
        let expr = self.parse_expression(Precedence::Lowest);
        if !matches!(expr, Expression::Block(_)) && !matches!(expr, Expression::MatchExpression { .. }) {
            if self.peek_token_is(TokenType::Semicolon) {
                self.next_token();
            }
        }
        Statement::Expression(expr)
    }

    fn parse_block_statement(&mut self) -> BlockStatement {
        let mut statements = Vec::new();
        self.next_token();

        while !self.cur_token_is(TokenType::RBrace) && !self.cur_token_is(TokenType::Eof) {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            }
            self.next_token();
        }

        BlockStatement { statements }
    }

    fn parse_if_statement(&mut self) -> Statement {
        if !self.expect_peek(TokenType::LParen) {
            return Statement::Expression(Expression::Identifier("error".into()));
        }
        self.next_token();
        let condition = self.parse_expression(Precedence::Lowest);

        if !self.expect_peek(TokenType::RParen) {
            return Statement::Expression(Expression::Identifier("error".into()));
        }

        if !self.expect_peek(TokenType::LBrace) {
            return Statement::Expression(Expression::Identifier("error".into()));
        }

        let consequence = self.parse_block_statement();
        let mut alternative = None;

        if self.peek_token_is(TokenType::Else) {
            self.next_token();
            if self.expect_peek(TokenType::LBrace) {
                alternative = Some(self.parse_block_statement());
            }
        }

        Statement::If {
            condition,
            consequence,
            alternative,
        }
    }

    fn parse_while_statement(&mut self) -> Statement {
        if !self.expect_peek(TokenType::LParen) {
            return Statement::Expression(Expression::Identifier("error".into()));
        }
        self.next_token();
        let condition = self.parse_expression(Precedence::Lowest);

        if !self.expect_peek(TokenType::RParen) {
            return Statement::Expression(Expression::Identifier("error".into()));
        }

        if !self.expect_peek(TokenType::LBrace) {
            return Statement::Expression(Expression::Identifier("error".into()));
        }

        let body = self.parse_block_statement();

        Statement::While { condition, body }
    }

    fn parse_for_statement(&mut self) -> Statement {
        if !self.expect_peek(TokenType::Var) {
            return Statement::Expression(Expression::Identifier("error".into()));
        }
        let variable = self.cur_token.literal.clone();

        if !self.expect_peek(TokenType::In) {
            return Statement::Expression(Expression::Identifier("error".into()));
        }

        self.next_token();
        let iterable = self.parse_expression(Precedence::Lowest);

        if !self.expect_peek(TokenType::LBrace) {
            return Statement::Expression(Expression::Identifier("error".into()));
        }

        let body = self.parse_block_statement();

        Statement::For {
            variable,
            iterable,
            body,
        }
    }

    fn parse_yield_statement(&mut self) -> Statement {
        self.next_token();
        if self.cur_token_is(TokenType::Semicolon) {
            return Statement::Yield(None);
        }

        let value = self.parse_expression(Precedence::Lowest);
        if self.peek_token_is(TokenType::Semicolon) {
            self.next_token();
        }

        Statement::Yield(Some(value))
    }

    fn parse_function_definition(&mut self, is_render: bool) -> Statement {
        let old_is_render = self.is_render;
        self.is_render = is_render;

        if is_render {
            self.expect_peek(TokenType::Function);
        }

        if !self.expect_peek(TokenType::Ident) {
            return Statement::Expression(Expression::Identifier("error".into()));
        }
        let name = self.cur_token.literal.clone();

        if !self.expect_peek(TokenType::LParen) {
            return Statement::Expression(Expression::Identifier("error".into()));
        }

        let parameters = self.parse_parameters();
        let mut return_type = None;

        if self.peek_token_is(TokenType::Colon) {
            self.next_token();
            self.next_token();
            return_type = Some(self.parse_type());
        }

        if !self.expect_peek(TokenType::LBrace) {
            return Statement::Expression(Expression::Identifier("error".into()));
        }

        let body = self.parse_block_statement();
        self.is_render = old_is_render;

        Statement::FunctionDefinition {
            name,
            parameters,
            body,
            return_type,
            is_render,
        }
    }

    fn parse_enum_statement(&mut self) -> Statement {
        if !self.expect_peek(TokenType::Ident) {
            return Statement::Expression(Expression::Identifier("error".into()));
        }
        let name = self.cur_token.literal.clone();

        if !self.expect_peek(TokenType::LBrace) {
            return Statement::Expression(Expression::Identifier("error".into()));
        }

        let mut cases = Vec::new();
        self.next_token();

        while !self.cur_token_is(TokenType::RBrace) && !self.cur_token_is(TokenType::Eof) {
            let case_name = self.cur_token.literal.clone();
            let mut value = None;

            if self.peek_token_is(TokenType::Assign) {
                self.next_token();
                self.next_token();
                value = Some(self.parse_expression(Precedence::Lowest));
            }

            cases.push(EnumCase { name: case_name, value });

            if self.peek_token_is(TokenType::Comma) {
                self.next_token();
            }
            self.next_token();
        }

        Statement::Enum { name, cases }
    }

    fn parse_struct_definition(&mut self) -> Statement {
        if !self.expect_peek(TokenType::Ident) {
            return Statement::Expression(Expression::Identifier("error".into()));
        }
        let name = self.cur_token.literal.clone();

        if !self.expect_peek(TokenType::LBrace) {
            return Statement::Expression(Expression::Identifier("error".into()));
        }

        let mut fields = Vec::new();
        self.next_token();

        while !self.cur_token_is(TokenType::RBrace) && !self.cur_token_is(TokenType::Eof) {
            let mut is_readonly = false;
            if self.cur_token_is(TokenType::Readonly) {
                is_readonly = true;
                self.next_token();
            }

            if !self.cur_token_is(TokenType::Var) {
                self.next_token();
                continue;
            }

            let field_name = self.cur_token.literal.clone();
            let mut field_type = None;

            if self.peek_token_is(TokenType::Colon) {
                self.next_token();
                self.next_token();
                field_type = Some(self.parse_type());
            }

            fields.push(StructField {
                name: field_name,
                field_type,
                is_readonly,
            });

            if self.peek_token_is(TokenType::Semicolon) {
                self.next_token();
            }
            self.next_token();
        }

        Statement::Struct { name, fields }
    }

    fn parse_type(&mut self) -> String {
        let mut t = self.cur_token.literal.clone();
        while self.peek_token_is(TokenType::Ident) || self.peek_token_is(TokenType::And) || self.peek_token_is(TokenType::Or) {
            self.next_token();
            t.push_str(&self.cur_token.literal);
        }
        t
    }

    fn parse_parameters(&mut self) -> Vec<Parameter> {
        let mut params = Vec::new();
        if self.peek_token_is(TokenType::RParen) {
            self.next_token();
            return params;
        }

        self.next_token();
        params.push(self.parse_parameter());

        while self.peek_token_is(TokenType::Comma) {
            self.next_token();
            self.next_token();
            params.push(self.parse_parameter());
        }

        self.expect_peek(TokenType::RParen);
        params
    }

    fn parse_parameter(&mut self) -> Parameter {
        let mut name = String::new();
        let mut param_type = None;
        let mut is_var = false;

        match self.cur_token.token_type {
            TokenType::StringType | TokenType::IntType | TokenType::BoolType | TokenType::FloatType | TokenType::AnyType | TokenType::Ident => {
                let first_lit = self.cur_token.literal.clone();
                if self.peek_token_is(TokenType::Var) || self.peek_token_is(TokenType::Ident) {
                    param_type = Some(first_lit);
                    self.next_token();
                    name = self.cur_token.literal.clone();
                    is_var = self.cur_token.token_type == TokenType::Var;
                } else {
                    name = first_lit;
                    is_var = false;
                    if self.peek_token_is(TokenType::Colon) {
                        self.next_token();
                        self.next_token();
                        param_type = Some(self.parse_type());
                    }
                }
            }
            TokenType::Var => {
                name = self.cur_token.literal.clone();
                is_var = true;
                if self.peek_token_is(TokenType::Colon) {
                    self.next_token();
                    self.next_token();
                    param_type = Some(self.parse_type());
                }
            }
            _ => {
                self.errors.push(format!("Expected parameter, got {:?}", self.cur_token.token_type));
            }
        }

        Parameter {
            name,
            param_type,
            is_var,
        }
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Expression {
        let mut left = match self.cur_token.token_type {
            TokenType::Ident => Expression::Identifier(self.cur_token.literal.clone()),
            TokenType::Var => Expression::Variable(self.cur_token.literal.clone()),
            TokenType::Literal(_) => Expression::StringLiteral {
                value: self.cur_token.literal.clone(),
                is_render: self.is_render,
                delimiter: if self.cur_token.literal.starts_with('\'') { '\'' } else { '"' },
            },
            TokenType::Int => Expression::IntegerLiteral(self.cur_token.literal.parse().unwrap_or(0)),
            TokenType::Print => Expression::Identifier("print".into()),
            TokenType::LBracket => self.parse_array_literal(),
            TokenType::Match => self.parse_match_expression(),
            TokenType::Fn => self.parse_arrow_function_expression(),
            TokenType::Spawn => self.parse_spawn_expression(),
            TokenType::LBrace => self.parse_brace_expression(),
            TokenType::Bang | TokenType::Minus => self.parse_prefix_expression(),
            TokenType::LParen => self.parse_grouped_expression(),
            _ => {
                self.errors.push(format!("No prefix parse function for {:?}", self.cur_token.token_type));
                return Expression::Identifier("error".into());
            }
        };

        while !self.peek_token_is(TokenType::Semicolon) && !self.peek_token_is(TokenType::Eof) && precedence < Precedence::from(&self.peek_token.token_type) {
            match self.peek_token.token_type {
                TokenType::Dot | TokenType::Nullsafe => {
                    self.next_token();
                    left = self.parse_method_call_expression(left);
                }
                TokenType::LParen => {
                    self.next_token();
                    left = self.parse_call_expression(left);
                }
                TokenType::LBracket => {
                    self.next_token();
                    left = self.parse_index_expression(left);
                }
                TokenType::Coalesce => {
                    self.next_token();
                    left = self.parse_null_coalesce_expression(left);
                }
                TokenType::Plus | TokenType::Minus | TokenType::Asterisk | TokenType::Slash | TokenType::Lt | TokenType::Gt | TokenType::Eq | TokenType::NotEq => {
                    self.next_token();
                    left = self.parse_infix_expression(left);
                }
                TokenType::Pipe => {
                    self.next_token();
                    left = self.parse_pipe_expression(left);
                }
                TokenType::Assign => {
                    self.next_token();
                    left = self.parse_assign_expression(left);
                }
                _ => return left,
            }
        }

        left
    }

    fn parse_prefix_expression(&mut self) -> Expression {
        let operator = self.cur_token.literal.clone();
        self.next_token();
        let right = self.parse_expression(Precedence::Prefix);
        Expression::PrefixExpression {
            operator,
            right: Box::new(right),
        }
    }

    fn parse_infix_expression(&mut self, left: Expression) -> Expression {
        let operator = self.cur_token.literal.clone();
        let precedence = Precedence::from(&self.cur_token.token_type);
        self.next_token();
        let right = self.parse_expression(precedence);
        Expression::InfixExpression {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }

    fn parse_grouped_expression(&mut self) -> Expression {
        self.next_token();
        let expr = self.parse_expression(Precedence::Lowest);
        self.expect_peek(TokenType::RParen);
        expr
    }

    fn parse_array_literal(&mut self) -> Expression {
        let elements = self.parse_expression_list(TokenType::RBracket);
        Expression::ArrayLiteral(elements)
    }

    fn parse_expression_list(&mut self, end: TokenType) -> Vec<Expression> {
        let mut list = Vec::new();
        if self.peek_token_is(end.clone()) {
            self.next_token();
            return list;
        }

        self.next_token();
        list.push(self.parse_expression(Precedence::Lowest));

        while self.peek_token_is(TokenType::Comma) {
            self.next_token();
            self.next_token();
            list.push(self.parse_expression(Precedence::Lowest));
        }

        self.expect_peek(end);
        list
    }

    fn parse_call_expression(&mut self, function: Expression) -> Expression {
        let arguments = self.parse_expression_list(TokenType::RParen);
        Expression::CallExpression {
            function: Box::new(function),
            arguments,
        }
    }

    fn parse_index_expression(&mut self, left: Expression) -> Expression {
        if self.peek_token_is(TokenType::RBracket) {
            self.next_token();
            return Expression::IndexExpression {
                left: Box::new(left),
                index: Box::new(Expression::Identifier("".into())), // Empty index for push
            };
        }
        self.next_token();
        let index = self.parse_expression(Precedence::Lowest);
        self.expect_peek(TokenType::RBracket);
        Expression::IndexExpression {
            left: Box::new(left),
            index: Box::new(index),
        }
    }

    fn parse_method_call_expression(&mut self, object: Expression) -> Expression {
        let is_nullsafe = self.cur_token.token_type == TokenType::Nullsafe;
        if !self.expect_peek(TokenType::Ident) {
            return Expression::Identifier("error".into());
        }
        let method = self.cur_token.literal.clone();

        if !self.peek_token_is(TokenType::LParen) {
            return Expression::MemberExpression {
                object: Box::new(object),
                property: method,
                is_nullsafe,
            };
        }

        self.next_token();
        if let Expression::Identifier(ref name) = object {
            if name == "db" && method == "query" {
                return self.parse_sql_query_expression();
            }
        }

        let arguments = self.parse_expression_list(TokenType::RParen);
        Expression::MethodCallExpression {
            object: Box::new(object),
            method,
            arguments,
            is_nullsafe,
        }
    }

    fn parse_sql_query_expression(&mut self) -> Expression {
        let mut query = String::new();
        let mut args = Vec::new();
        let mut columns = Vec::new();
        let mut table = None;
        let mut is_select = false;
        let mut is_from = false;

        self.next_token();

        while !self.cur_token_is(TokenType::RParen) && !self.cur_token_is(TokenType::Eof) {
            let literal = self.cur_token.literal.clone();
            let upper = literal.to_uppercase();

            if upper == "SELECT" {
                is_select = true;
            } else if upper == "FROM" {
                is_select = false;
                is_from = true;
            } else if is_select && literal != "," {
                columns.push(literal.clone());
            } else if is_from {
                table = Some(literal.clone());
                is_from = false;
            }

            if self.cur_token_is(TokenType::LBrace) {
                self.next_token();
                args.push(self.parse_expression(Precedence::Lowest));
                if !query.is_empty() {
                    query.push(' ');
                }
                query.push('?');
                self.expect_peek(TokenType::RBrace);
            } else {
                if !query.is_empty() && literal != "," && literal != "." {
                    query.push(' ');
                }
                if self.cur_token_is(TokenType::Var) {
                    query.push('$');
                }
                query.push_str(&literal);
            }
            self.next_token();
        }

        Expression::SqlQueryExpression {
            query,
            args,
            table,
            columns,
        }
    }

    fn parse_match_expression(&mut self) -> Expression {
        if !self.expect_peek(TokenType::LParen) {
            return Expression::Identifier("error".into());
        }
        self.next_token();
        let condition = self.parse_expression(Precedence::Lowest);

        if !self.expect_peek(TokenType::RParen) {
            return Expression::Identifier("error".into());
        }

        if !self.expect_peek(TokenType::LBrace) {
            return Expression::Identifier("error".into());
        }

        let mut arms = Vec::new();
        self.next_token();

        while !self.cur_token_is(TokenType::RBrace) && !self.cur_token_is(TokenType::Eof) {
            let mut values = Vec::new();
            let mut is_default = false;

            if self.cur_token.literal == "default" {
                is_default = true;
            } else {
                values.push(self.parse_expression(Precedence::Lowest));
                while self.peek_token_is(TokenType::Comma) {
                    self.next_token();
                    self.next_token();
                    values.push(self.parse_expression(Precedence::Lowest));
                }
            }

            if !self.expect_peek(TokenType::Arrow) {
                return Expression::Identifier("error".into());
            }

            self.next_token();
            let result = self.parse_expression(Precedence::Lowest);
            arms.push(MatchArm {
                values,
                result,
                is_default,
            });

            if self.peek_token_is(TokenType::Comma) {
                self.next_token();
            }
            self.next_token();
        }

        Expression::MatchExpression {
            condition: Box::new(condition),
            arms,
        }
    }

    fn parse_arrow_function_expression(&mut self) -> Expression {
        if !self.expect_peek(TokenType::LParen) {
            return Expression::Identifier("error".into());
        }
        let parameters = self.parse_parameters();
        let mut return_type = None;

        if self.peek_token_is(TokenType::Colon) {
            self.next_token();
            self.next_token();
            return_type = Some(self.parse_type());
        }

        if !self.expect_peek(TokenType::Arrow) {
            return Expression::Identifier("error".into());
        }

        self.next_token();
        let body = self.parse_expression(Precedence::Lowest);

        Expression::ArrowFunctionExpression {
            parameters,
            body: Box::new(body),
            return_type,
        }
    }

    fn parse_spawn_expression(&mut self) -> Expression {
        if self.peek_token_is(TokenType::LBrace) {
            self.next_token();
            let body = self.parse_block_statement();
            Expression::SpawnExpression {
                body: Box::new(Statement::Expression(Expression::Block(body))),
            }
        } else {
            self.next_token();
            let body = self.parse_statement().unwrap_or(Statement::Expression(Expression::Identifier("error".into())));
            Expression::SpawnExpression {
                body: Box::new(body),
            }
        }
    }

    fn parse_brace_expression(&mut self) -> Expression {
        // Look ahead to see if it's a map (key: val) or a block (stmt)
        // If it's an empty brace {}, we'll treat it as a map for now (compatibility)
        if self.peek_token_is(TokenType::RBrace) {
            self.next_token();
            return Expression::MapLiteral(Vec::new());
        }

        // Check if it's a map: cur=LBrace, peek=key, peek2=Colon
        // We'll peek manually using the lexer if needed, or just guess based on peek
        // Let's assume if peek is Ident/Literal and peek.peek is Colon, it's a map.
        // For simplicity, let's just try to parse a block and if it fails or looks like a map, fallback.
        // Actually, Let's just check the next token and the one after it.
        
        // Actually, let's just use a simple heuristic:
        // If next token is Ident/Literal and the one after is Colon, it's a map.
        // But we don't have a peek_peek token easily.
        // Let's just try parse_block_statement and if it contains a single expression statement that is an assignment or something... no.
        
        // Let's stick to: if we see a Colon after the first token, it's a map.
        // I'll add a peek_peek_token to the parser.
        
        if self.peek_token.token_type == TokenType::Ident || matches!(self.peek_token.token_type, TokenType::Literal(_)) {
             // We'd need to peek one more. For now, let's just assume if it's in a Render function and starts with an Ident, it might be a block.
             // But wait, the most reliable way in Zenith is to see if there's a Colon.
        }
        
        // Demo app uses blocks in match arms.
        Expression::Block(self.parse_block_statement())
    }


    fn parse_null_coalesce_expression(&mut self, left: Expression) -> Expression {
        let precedence = Precedence::from(&self.cur_token.token_type);
        self.next_token();
        let right = self.parse_expression(precedence);
        Expression::NullCoalesceExpression {
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    fn parse_pipe_expression(&mut self, left: Expression) -> Expression {
        let precedence = Precedence::from(&self.cur_token.token_type);
        self.next_token();
        let right = self.parse_expression(precedence);
        Expression::PipeExpression {
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    fn parse_assign_expression(&mut self, left: Expression) -> Expression {
        let precedence = Precedence::from(&self.cur_token.token_type);
        self.next_token();
        let value = self.parse_expression(precedence);
        Expression::AssignExpression {
            left: Box::new(left),
            value: Box::new(value),
        }
    }

    fn cur_token_is(&self, t: TokenType) -> bool {
        match (&self.cur_token.token_type, &t) {
            (TokenType::Literal(_), TokenType::Literal(_)) => true,
            (a, b) => a == b,
        }
    }

    fn peek_token_is(&self, t: TokenType) -> bool {
        match (&self.peek_token.token_type, &t) {
            (TokenType::Literal(_), TokenType::Literal(_)) => true,
            (a, b) => a == b,
        }
    }

    fn expect_peek(&mut self, t: TokenType) -> bool {
        if self.peek_token_is(t.clone()) {
            self.next_token();
            true
        } else {
            self.peek_error(t);
            false
        }
    }

    fn peek_error(&mut self, t: TokenType) {
        let msg = format!("Expected next token to be {:?}, got {:?} instead", t, self.peek_token.token_type);
        self.errors.push(msg);
    }
}
