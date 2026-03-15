use crate::core::ast::{
    BlockStatement, EnumCase, Expression, ExpressionKind, MatchArm, Parameter, Pattern, PatternKind,
    Program, Statement, StatementKind, StructField,
};
use crate::core::lexer::{Lexer, Token, TokenType};
use logos::Span;

#[derive(Debug, Clone)]
pub struct ParserError {
    pub message: String,
    pub span: Span,
    pub label: Option<String>,
    pub help: Option<String>,
}

#[derive(PartialEq, PartialOrd)]
enum Precedence {
    Lowest,
    Pipe,
    Assign,
    Coalesce,
    Or,
    And,
    Comparison,
    Sum,
    Product,
    Modulo,
    Prefix,
    Call,
    Index,
}

impl From<&TokenType> for Precedence {
    fn from(t: &TokenType) -> Self {
        match t {
            TokenType::Pipe => Precedence::Pipe,
            TokenType::Assign => Precedence::Assign,
            TokenType::Coalesce => Precedence::Coalesce,
            TokenType::Or => Precedence::Or,
            TokenType::And => Precedence::And,
            TokenType::LParen => Precedence::Call,
            TokenType::LBracket | TokenType::Nullsafe => Precedence::Index,
            TokenType::Dot => Precedence::Call,
            TokenType::Lt | TokenType::Gt | TokenType::Leq | TokenType::Geq | TokenType::Eq | TokenType::NotEq => Precedence::Comparison,
            TokenType::Plus | TokenType::Minus => Precedence::Sum,
            TokenType::Asterisk | TokenType::Slash => Precedence::Product,
            TokenType::Modulo => Precedence::Modulo,
            _ => Precedence::Lowest,
        }
    }
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    cur_token: Token,
    peek_token: Token,
    pub errors: Vec<ParserError>,
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
            span: self.cur_token.span.clone(),
        };

        if self.cur_token_is(TokenType::Before) {
            self.next_token();
            if self.cur_token_is(TokenType::LBrace) {
                program.middleware = Some(self.parse_block_statement());
            }
            if !self.cur_token_is(TokenType::Eof) {
                self.next_token();
            }
        }

        while !self.cur_token_is(TokenType::Eof) {
            if let Some(stmt) = self.parse_statement() {
                if let StatementKind::Import(_) = &stmt.kind {
                    program.imports.push(stmt);
                } else {
                    program.statements.push(stmt);
                }
            }
            if !self.cur_token_is(TokenType::Eof) {
                self.next_token();
            }
        }

        program.span.end = self.cur_token.span.end;
        program
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        match self.cur_token.token_type {
            TokenType::Render => Some(self.parse_function_definition(true, false)),
            TokenType::Function | TokenType::Fn => Some(self.parse_function_definition(false, false)),
            TokenType::Return => Some(self.parse_return_statement()),
            TokenType::Let => Some(self.parse_let_statement()),
            TokenType::If => Some(self.parse_if_statement()),
            TokenType::While => Some(self.parse_while_statement()),
            TokenType::For => Some(self.parse_for_statement()),
            TokenType::Yield => Some(self.parse_yield_statement()),
            TokenType::Enum => Some(self.parse_enum_statement()),
            TokenType::Test => Some(self.parse_test_statement()),
            TokenType::Struct => Some(self.parse_struct_definition()),
            TokenType::At => {
                self.next_token();
                if self.cur_token.literal == "memoize" {
                    self.next_token();
                    if self.cur_token_is(TokenType::Function) {
                        Some(self.parse_function_definition(false, true))
                    } else if self.cur_token_is(TokenType::Render) {
                        Some(self.parse_function_definition(true, true))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => {
                if self.peek_token_is(TokenType::LBrace) {
                    Some(self.parse_struct_definition())
                } else {
                    Some(self.parse_expression_statement())
                }
            }
        }
    }

    fn parse_function_definition(&mut self, is_render: bool, is_memoized: bool) -> Statement {
        let start_span = self.cur_token.span.clone();
        let old_render = self.is_render;
        self.is_render = is_render;

        if is_render {
            self.expect_peek(TokenType::Function);
        }

        self.expect_peek(TokenType::Ident);
        let name = self.cur_token.literal.clone();

        self.expect_peek(TokenType::LParen);
        let parameters = self.parse_parameters();

        let mut return_type = None;
        if self.peek_token_is(TokenType::Colon) {
            self.next_token();
            self.next_token();
            return_type = Some(self.parse_type());
        }

        self.expect_peek(TokenType::LBrace);
        let body = self.parse_block_statement();
        self.is_render = old_render;

        Statement {
            kind: StatementKind::FunctionDefinition {
                name,
                parameters,
                body: body.clone(),
                return_type,
                is_render,
                is_memoized,
            },
            span: start_span.start..body.span.end,
        }
    }

    fn parse_let_statement(&mut self) -> Statement {
        let start_span = self.cur_token.span.clone();
        self.expect_peek(TokenType::Var);
        let name = self.cur_token.literal.clone();

        let mut var_type = None;
        if self.peek_token_is(TokenType::Colon) {
            self.next_token();
            self.next_token();
            var_type = Some(self.parse_type());
        }

        self.expect_peek(TokenType::Assign);
        self.next_token();
        let value = self.parse_expression(Precedence::Lowest);

        if self.peek_token_is(TokenType::Semicolon) {
            self.next_token();
        }

        Statement {
            kind: StatementKind::Let { name, value, var_type },
            span: start_span.start..self.cur_token.span.end,
        }
    }

    fn parse_return_statement(&mut self) -> Statement {
        let start_span = self.cur_token.span.clone();
        self.next_token();
        let value = self.parse_expression(Precedence::Lowest);

        if self.peek_token_is(TokenType::Semicolon) {
            self.next_token();
        }

        Statement {
            kind: StatementKind::Return(value.clone()),
            span: start_span.start..self.cur_token.span.end,
        }
    }

    fn parse_struct_definition(&mut self) -> Statement {
        let start_span = self.cur_token.span.clone();
        if self.cur_token_is(TokenType::Struct) {
            self.next_token();
        }
        let name = self.cur_token.literal.clone();

        self.expect_peek(TokenType::LBrace);
        let mut fields = Vec::new();

        while !self.peek_token_is(TokenType::RBrace) && !self.peek_token_is(TokenType::Eof) {
            self.next_token();
            let mut is_readonly = false;
            if self.cur_token_is(TokenType::Readonly) {
                is_readonly = true;
                self.next_token();
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

            if self.peek_token_is(TokenType::Semicolon) || self.peek_token_is(TokenType::Comma) {
                self.next_token();
            }
        }

        self.expect_peek(TokenType::RBrace);

        Statement {
            kind: StatementKind::Struct { name, fields },
            span: start_span.start..self.cur_token.span.end,
        }
    }

    fn parse_block_statement(&mut self) -> BlockStatement {
        let start_span = self.cur_token.span.clone();
        let mut statements = Vec::new();

        self.next_token();
        while !self.cur_token_is(TokenType::RBrace) && !self.cur_token_is(TokenType::Eof) {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            }
            self.next_token();
        }

        BlockStatement {
            statements,
            span: start_span.start..self.cur_token.span.end,
        }
    }

    fn parse_expression_statement(&mut self) -> Statement {
        let start_span = self.cur_token.span.clone();
        let expression = self.parse_expression(Precedence::Lowest);

        if self.peek_token_is(TokenType::Semicolon) {
            self.next_token();
        }

        Statement {
            kind: StatementKind::Expression(expression.clone()),
            span: start_span.start..self.cur_token.span.end,
        }
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Expression {
        let mut left = match self.cur_token.token_type {
            TokenType::Ident => {
                if self.peek_token_is(TokenType::LBrace) {
                    self.parse_struct_literal()
                } else {
                    self.parse_identifier()
                }
            }
            TokenType::Var => {
                Expression {
                    kind: ExpressionKind::Variable(self.cur_token.literal.clone()),
                    span: self.cur_token.span.clone(),
                }
            }
            TokenType::Literal(_) => self.parse_literal_expression(),
            TokenType::Int => self.parse_integer_literal(),
            TokenType::Float => self.parse_float_literal(),
            TokenType::Bang | TokenType::Minus => self.parse_prefix_expression(),
            TokenType::LParen => self.parse_grouped_expression(),
            TokenType::LBracket => self.parse_collection_literal(),
            TokenType::Select
            | TokenType::From
            | TokenType::Where
            | TokenType::Insert
            | TokenType::Update
            | TokenType::Delete
            | TokenType::Into
            | TokenType::Values
            | TokenType::Set => self.parse_sql_query_expression(self.cur_token.span.start),
            TokenType::Fn => self.parse_arrow_function_expression(),
            TokenType::Spawn => self.parse_spawn_expression(),
            TokenType::Match => self.parse_match_expression(),
            _ => {
                self.errors.push(ParserError {
                    message: format!("no prefix parse function for {:?}", self.cur_token.token_type),
                    span: self.cur_token.span.clone(),
                    label: Some("unexpected token here".into()),
                    help: Some("Start expressions with a value (identifier, literal, `(`, `[`, `{`) or a unary operator (`!`, `-`).".into()),
                });
                Expression {
                    kind: ExpressionKind::Identifier("error".into()),
                    span: self.cur_token.span.clone(),
                }
            }
        };

        while !self.peek_token_is(TokenType::Semicolon)
            && !self.peek_token_is(TokenType::Eof)
            && precedence < Precedence::from(&self.peek_token.token_type)
        {
            match self.peek_token.token_type {
                TokenType::Plus
                | TokenType::Minus
                | TokenType::Asterisk
                | TokenType::Slash
                | TokenType::Modulo
                | TokenType::Lt
                | TokenType::Gt
                | TokenType::Leq
                | TokenType::Geq
                | TokenType::Eq
                | TokenType::NotEq
                | TokenType::And
                | TokenType::Or => {
                    self.next_token();
                    left = self.parse_infix_expression(left);
                }
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
                TokenType::Pipe => {
                    self.next_token();
                    left = self.parse_pipe_expression(left);
                }
                TokenType::Coalesce => {
                    self.next_token();
                    left = self.parse_null_coalesce_expression(left);
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

    fn parse_identifier(&mut self) -> Expression {
        Expression {
            kind: ExpressionKind::Identifier(self.cur_token.literal.clone()),
            span: self.cur_token.span.clone(),
        }
    }

    fn parse_integer_literal(&mut self) -> Expression {
        Expression {
            kind: ExpressionKind::IntegerLiteral(self.cur_token.literal.parse().unwrap_or(0)),
            span: self.cur_token.span.clone(),
        }
    }

    fn parse_float_literal(&mut self) -> Expression {
        Expression {
            kind: ExpressionKind::FloatLiteral(self.cur_token.literal.parse().unwrap_or(0.0)),
            span: self.cur_token.span.clone(),
        }
    }

    fn parse_literal_expression(&mut self) -> Expression {
        Expression {
            kind: ExpressionKind::StringLiteral {
                value: self.cur_token.literal.clone(),
                is_render: self.is_render,
                delimiter: if self.cur_token.literal.starts_with('\'') { '\'' } else { '"' },
            },
            span: self.cur_token.span.clone(),
        }
    }

    fn parse_prefix_expression(&mut self) -> Expression {
        let start_span = self.cur_token.span.clone();
        let operator = self.cur_token.literal.clone();
        self.next_token();
        let right = self.parse_expression(Precedence::Prefix);

        Expression {
            kind: ExpressionKind::PrefixExpression {
                operator,
                right: Box::new(right.clone()),
            },
            span: start_span.start..right.span.end,
        }
    }

    fn parse_infix_expression(&mut self, left: Expression) -> Expression {
        let start_span = left.span.clone();
        let operator = self.cur_token.literal.clone();
        let precedence = Precedence::from(&self.cur_token.token_type);
        self.next_token();
        let right = self.parse_expression(precedence);

        Expression {
            kind: ExpressionKind::InfixExpression {
                left: Box::new(left),
                operator,
                right: Box::new(right.clone()),
            },
            span: start_span.start..right.span.end,
        }
    }

    fn parse_method_call_expression(&mut self, object: Expression) -> Expression {
        let start_span = object.span.clone();
        let is_nullsafe = self.cur_token_is(TokenType::Nullsafe);
        self.next_token();
        let method = self.cur_token.literal.clone();

        if self.peek_token_is(TokenType::LParen) {
            self.next_token();
            if let ExpressionKind::Identifier(ref name) = object.kind {
                if name == "db" {
                    match self.peek_token.token_type {
                        TokenType::Select | TokenType::Insert | TokenType::Update | TokenType::Delete | TokenType::From | TokenType::Where | TokenType::Into | TokenType::Values | TokenType::Set => {
                            self.next_token();
                            let expr = self.parse_sql_query_expression(start_span.start);
                            self.expect_peek(TokenType::RParen);
                            return expr;
                        }
                        _ => {
                            if method == "query" {
                                self.next_token();
                                let expr = self.parse_sql_query_expression(start_span.start);
                                self.expect_peek(TokenType::RParen);
                                return expr;
                            }
                        }
                    }
                }
            }
            let arguments = self.parse_expression_list(TokenType::RParen);
            Expression {
                kind: ExpressionKind::MethodCallExpression {
                    object: Box::new(object),
                    method,
                    arguments,
                    is_nullsafe,
                },
                span: start_span.start..self.cur_token.span.end,
            }
        } else {
            Expression {
                kind: ExpressionKind::MemberExpression {
                    object: Box::new(object),
                    property: method,
                    is_nullsafe,
                },
                span: start_span.start..self.cur_token.span.end,
            }
        }
    }

    fn parse_call_expression(&mut self, function: Expression) -> Expression {
        let start_span = function.span.clone();
        let arguments = self.parse_expression_list(TokenType::RParen);
        Expression {
            kind: ExpressionKind::CallExpression {
                function: Box::new(function),
                arguments,
            },
            span: start_span.start..self.cur_token.span.end,
        }
    }

    fn parse_index_expression(&mut self, left: Expression) -> Expression {
        let start_span = left.span.clone();
        self.next_token();
        let index = self.parse_expression(Precedence::Lowest);
        self.expect_peek(TokenType::RBracket);

        Expression {
            kind: ExpressionKind::IndexExpression {
                left: Box::new(left),
                index: Box::new(index),
            },
            span: start_span.start..self.cur_token.span.end,
        }
    }

    fn parse_collection_literal(&mut self) -> Expression {
        let start_span = self.cur_token.span.clone();
        if self.peek_token_is(TokenType::RBracket) {
            self.next_token();
            return Expression {
                kind: ExpressionKind::ArrayLiteral(Vec::new()),
                span: start_span.start..self.cur_token.span.end,
            };
        }

        self.next_token();
        let first_expr = self.parse_expression(Precedence::Lowest);

        if self.peek_token_is(TokenType::Colon) {
            self.next_token();
            self.next_token();
            let first_value = self.parse_expression(Precedence::Lowest);
            let mut pairs = vec![(first_expr, first_value)];

            while self.peek_token_is(TokenType::Comma) {
                self.next_token();
                if self.peek_token_is(TokenType::RBracket) { break; }
                self.next_token();
                let key = self.parse_expression(Precedence::Lowest);
                self.expect_peek(TokenType::Colon);
                self.next_token();
                let value = self.parse_expression(Precedence::Lowest);
                pairs.push((key, value));
            }
            self.expect_peek(TokenType::RBracket);
            Expression {
                kind: ExpressionKind::MapLiteral(pairs),
                span: start_span.start..self.cur_token.span.end,
            }
        } else {
            let mut elements = vec![first_expr];
            while self.peek_token_is(TokenType::Comma) {
                self.next_token();
                if self.peek_token_is(TokenType::RBracket) { break; }
                self.next_token();
                elements.push(self.parse_expression(Precedence::Lowest));
            }
            self.expect_peek(TokenType::RBracket);
            Expression {
                kind: ExpressionKind::ArrayLiteral(elements),
                span: start_span.start..self.cur_token.span.end,
            }
        }
    }

    fn parse_sql_query_expression(&mut self, start_pos: usize) -> Expression {
        let mut query = String::new();
        let mut args = Vec::new();
        let mut columns = Vec::new();
        let mut table = None;
        let mut is_select = false;
        let mut is_from = false;
        let mut paren_depth = 0;

        while !self.cur_token_is(TokenType::Eof) {
            if self.cur_token_is(TokenType::RParen) && paren_depth == 0 { break; }
            if self.cur_token_is(TokenType::LParen) { paren_depth += 1; }
            else if self.cur_token_is(TokenType::RParen) { paren_depth -= 1; }

            let literal = self.cur_token.literal.clone();
            let upper = literal.to_uppercase();

            if upper == "SELECT" { is_select = true; }
            else if upper == "FROM" { is_select = false; is_from = true; }
            else if is_select && literal != "," && !self.cur_token_is(TokenType::LBrace) {
                columns.push(literal.clone());
            } else if is_from && !self.cur_token_is(TokenType::LBrace) {
                table = Some(literal.clone());
                is_from = false;
            }

            if self.cur_token_is(TokenType::LBrace) {
                self.next_token();
                args.push(self.parse_expression(Precedence::Lowest));
                query.push('?');
                self.expect_peek(TokenType::RBrace);
            } else {
                if !query.is_empty() && literal != "," && literal != "." && literal != "(" && literal != ")" { query.push(' '); }
                if self.cur_token_is(TokenType::Var) { query.push('$'); }
                if self.cur_token_is(TokenType::Literal("".into())) {
                    query.push('"');
                    query.push_str(&literal);
                    query.push('"');
                } else {
                    query.push_str(&literal);
                }
            }

            if self.peek_token_is(TokenType::RParen) && paren_depth == 0 { break; }
            if self.peek_token_is(TokenType::Eof) { break; }
            self.next_token();
        }

        Expression {
            kind: ExpressionKind::SqlQueryExpression {
                query: query.trim().to_string(),
                args,
                table,
                columns,
            },
            span: start_pos..self.cur_token.span.end,
        }
    }

    fn parse_arrow_function_expression(&mut self) -> Expression {
        let start_span = self.cur_token.span.clone();
        self.expect_peek(TokenType::LParen);
        let parameters = self.parse_parameters();

        let mut return_type = None;
        if self.peek_token_is(TokenType::Colon) {
            self.next_token();
            self.next_token();
            return_type = Some(self.parse_type());
        }

        self.expect_peek(TokenType::Arrow);
        self.next_token();
        let body = self.parse_expression(Precedence::Lowest);

        Expression {
            kind: ExpressionKind::ArrowFunctionExpression {
                parameters,
                body: Box::new(body.clone()),
                return_type,
            },
            span: start_span.start..body.span.end,
        }
    }

    fn parse_match_expression(&mut self) -> Expression {
        let start_span = self.cur_token.span.clone();
        self.expect_peek(TokenType::LParen);
        self.next_token();
        let condition = self.parse_expression(Precedence::Lowest);
        self.expect_peek(TokenType::RParen);
        self.expect_peek(TokenType::LBrace);

        let mut arms = Vec::new();
        while !self.peek_token_is(TokenType::RBrace) && !self.peek_token_is(TokenType::Eof) {
            self.next_token();
            let mut patterns = Vec::new();
            let mut is_default = false;

            if self.cur_token_is(TokenType::Default) {
                is_default = true;
            } else {
                patterns.push(self.parse_pattern());
                while self.peek_token_is(TokenType::Comma) {
                    self.next_token();
                    self.next_token();
                    patterns.push(self.parse_pattern());
                }
            }

            self.expect_peek(TokenType::Arrow);
            self.next_token();
            let result = self.parse_expression(Precedence::Lowest);
            arms.push(MatchArm { patterns, result, is_default });

            if self.peek_token_is(TokenType::Comma) { self.next_token(); }
        }

        self.expect_peek(TokenType::RBrace);
        Expression {
            kind: ExpressionKind::MatchExpression {
                condition: Box::new(condition),
                arms,
            },
            span: start_span.start..self.cur_token.span.end,
        }
    }

    fn parse_pattern(&mut self) -> Pattern {
        let start_token = self.cur_token.clone();
        let start_span = start_token.span.clone();

        match &start_token.token_type {
            TokenType::Ident | TokenType::Var if start_token.literal == "_" => Pattern {
                kind: PatternKind::Wildcard,
                span: start_span,
            },
            TokenType::Ident | TokenType::Var => {
                let name = start_token.literal.clone();
                if self.peek_token_is(TokenType::LBrace) {
                    self.next_token(); // move to LBrace
                    self.next_token(); // move to first field
                    let mut fields = Vec::new();
                    while !self.cur_token_is(TokenType::RBrace) && !self.cur_token_is(TokenType::Eof) {
                        let field_name = if self.cur_token.literal.starts_with('$') {
                            self.cur_token.literal[1..].to_string()
                        } else {
                            self.cur_token.literal.clone()
                        };
                        self.expect_peek(TokenType::Colon);
                        self.next_token(); // move to pattern
                        fields.push((field_name, self.parse_pattern()));
                        if self.peek_token_is(TokenType::Comma) {
                            self.next_token();
                        }
                        self.next_token();
                    }
                    Pattern {
                        kind: PatternKind::Struct { name, fields },
                        span: start_span.start..self.cur_token.span.end,
                    }
                } else {
                    Pattern {
                        kind: PatternKind::Identifier(name),
                        span: start_span,
                    }
                }
            }
            _ => {
                let expr = self.parse_expression(Precedence::Lowest);
                Pattern {
                    kind: PatternKind::Literal(expr.clone()),
                    span: expr.span,
                }
            }
        }
    }

    fn parse_spawn_expression(&mut self) -> Expression {
        let start_span = self.cur_token.span.clone();
        self.next_token();
        let body = if self.cur_token_is(TokenType::LBrace) {
            self.parse_block_statement_as_statement()
        } else {
            self.parse_statement().unwrap()
        };

        Expression {
            kind: ExpressionKind::SpawnExpression { body: Box::new(body.clone()) },
            span: start_span.start..body.span.end,
        }
    }

    fn parse_block_statement_as_statement(&mut self) -> Statement {
        let block = self.parse_block_statement();
        Statement {
            kind: StatementKind::Expression(Expression {
                kind: ExpressionKind::Block(block.clone()),
                span: block.span.clone(),
            }),
            span: block.span,
        }
    }

    fn parse_struct_literal(&mut self) -> Expression {
        let start_span = self.cur_token.span.clone();
        let name = self.cur_token.literal.clone();
        self.expect_peek(TokenType::LBrace);

        let mut fields = Vec::new();
        while !self.peek_token_is(TokenType::RBrace) && !self.peek_token_is(TokenType::Eof) {
            self.next_token();
            let field_name = self.cur_token.literal.clone();
            self.expect_peek(TokenType::Colon);
            self.next_token();
            fields.push((field_name, self.parse_expression(Precedence::Lowest)));

            if self.peek_token_is(TokenType::Comma) { self.next_token(); }
        }

        self.expect_peek(TokenType::RBrace);
        Expression {
            kind: ExpressionKind::StructLiteral { name, fields },
            span: start_span.start..self.cur_token.span.end,
        }
    }

    fn parse_pipe_expression(&mut self, left: Expression) -> Expression {
        let start_span = left.span.clone();
        let precedence = Precedence::from(&self.cur_token.token_type);
        self.next_token();
        let right = self.parse_expression(precedence);

        Expression {
            kind: ExpressionKind::PipeExpression {
                left: Box::new(left),
                right: Box::new(right.clone()),
            },
            span: start_span.start..right.span.end,
        }
    }

    fn parse_null_coalesce_expression(&mut self, left: Expression) -> Expression {
        let start_span = left.span.clone();
        let precedence = Precedence::from(&self.cur_token.token_type);
        self.next_token();
        let right = self.parse_expression(precedence);

        Expression {
            kind: ExpressionKind::NullCoalesceExpression {
                left: Box::new(left),
                right: Box::new(right.clone()),
            },
            span: start_span.start..right.span.end,
        }
    }

    fn parse_assign_expression(&mut self, left: Expression) -> Expression {
        let start_span = left.span.clone();
        self.next_token();
        let value = self.parse_expression(Precedence::Lowest);

        Expression {
            kind: ExpressionKind::AssignExpression {
                left: Box::new(left),
                value: Box::new(value.clone()),
            },
            span: start_span.start..value.span.end,
        }
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
        let mut is_var = false;
        if self.cur_token_is(TokenType::Var) {
            is_var = true;
        }
        let name = self.cur_token.literal.clone();
        let mut param_type = None;

        if self.peek_token_is(TokenType::Colon) {
            self.next_token();
            self.next_token();
            param_type = Some(self.parse_type());
        }

        Parameter { name, param_type, is_var }
    }

    fn parse_type(&mut self) -> String {
        let mut t = self.cur_token.literal.clone();
        if self.peek_token_is(TokenType::LBracket) {
            self.next_token();
            self.expect_peek(TokenType::RBracket);
            t.push_str("[]");
        }
        t
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

    fn parse_if_statement(&mut self) -> Statement {
        let start_span = self.cur_token.span.clone();
        self.expect_peek(TokenType::LParen);
        self.next_token();
        let condition = self.parse_expression(Precedence::Lowest);
        self.expect_peek(TokenType::RParen);

        self.expect_peek(TokenType::LBrace);
        let consequence = self.parse_block_statement();

        let mut alternative = None;
        if self.peek_token_is(TokenType::Else) {
            self.next_token();
            self.expect_peek(TokenType::LBrace);
            alternative = Some(self.parse_block_statement());
        }

        let end_span = alternative.as_ref().map(|a| a.span.clone()).unwrap_or(consequence.span.clone());

        Statement {
            kind: StatementKind::If { condition, consequence: consequence.clone(), alternative: alternative.clone() },
            span: start_span.start..end_span.end,
        }
    }

    fn parse_while_statement(&mut self) -> Statement {
        let start_span = self.cur_token.span.clone();
        self.expect_peek(TokenType::LParen);
        self.next_token();
        let condition = self.parse_expression(Precedence::Lowest);
        self.expect_peek(TokenType::RParen);

        self.expect_peek(TokenType::LBrace);
        let body = self.parse_block_statement();

        Statement {
            kind: StatementKind::While { condition, body: body.clone() },
            span: start_span.start..body.span.end,
        }
    }

    fn parse_for_statement(&mut self) -> Statement {
        let start_span = self.cur_token.span.clone();
        self.expect_peek(TokenType::Var);
        let variable = self.cur_token.literal.clone();

        self.expect_peek(TokenType::In);
        self.next_token();
        let iterable = self.parse_expression(Precedence::Lowest);

        self.expect_peek(TokenType::LBrace);
        let body = self.parse_block_statement();

        Statement {
            kind: StatementKind::For { variable, iterable, body: body.clone() },
            span: start_span.start..body.span.end,
        }
    }

    fn parse_yield_statement(&mut self) -> Statement {
        let start_span = self.cur_token.span.clone();
        let mut value = None;
        if !self.peek_token_is(TokenType::Semicolon) {
            self.next_token();
            value = Some(self.parse_expression(Precedence::Lowest));
        }

        if self.peek_token_is(TokenType::Semicolon) {
            self.next_token();
        }

        Statement {
            kind: StatementKind::Yield(value),
            span: start_span.start..self.cur_token.span.end,
        }
    }

    fn parse_enum_statement(&mut self) -> Statement {
        let start_span = self.cur_token.span.clone();
        self.expect_peek(TokenType::Ident);
        let name = self.cur_token.literal.clone();

        self.expect_peek(TokenType::LBrace);
        let mut cases = Vec::new();

        while !self.peek_token_is(TokenType::RBrace) && !self.peek_token_is(TokenType::Eof) {
            self.next_token();
            let case_name = self.cur_token.literal.clone();
            let mut value = None;
            if self.peek_token_is(TokenType::Assign) {
                self.next_token();
                self.next_token();
                value = Some(self.parse_expression(Precedence::Lowest));
            }
            cases.push(EnumCase { name: case_name, value });
            if self.peek_token_is(TokenType::Comma) { self.next_token(); }
        }

        self.expect_peek(TokenType::RBrace);
        Statement {
            kind: StatementKind::Enum { name, cases },
            span: start_span.start..self.cur_token.span.end,
        }
    }

    fn parse_test_statement(&mut self) -> Statement {
        let start_span = self.cur_token.span.clone();
        self.expect_peek(TokenType::Literal("".into()));
        let name = self.cur_token.literal.clone();

        self.expect_peek(TokenType::LBrace);
        let body = self.parse_block_statement();

        Statement {
            kind: StatementKind::Test { name, body: body.clone() },
            span: start_span.start..body.span.end,
        }
    }

    fn parse_grouped_expression(&mut self) -> Expression {
        let start_span = self.cur_token.span.clone();
        self.next_token();
        let mut expr = self.parse_expression(Precedence::Lowest);
        self.expect_peek(TokenType::RParen);
        expr.span = start_span.start..self.cur_token.span.end;
        expr
    }

    fn parse_index_expression_internal(&mut self, left: Expression) -> Expression {
        let start_span = left.span.clone();
        self.next_token();
        let index = self.parse_expression(Precedence::Lowest);
        self.expect_peek(TokenType::RBracket);
        Expression {
            kind: ExpressionKind::IndexExpression {
                left: Box::new(left),
                index: Box::new(index),
            },
            span: start_span.start..self.cur_token.span.end,
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
        let msg = format!("expected {:?}, found {:?}", t, self.peek_token.token_type);
        self.errors.push(ParserError {
            message: msg,
            span: self.peek_token.span.clone(),
            label: Some(format!("expected {:?} here", t)),
            help: Some("Check for a missing delimiter, comma, semicolon, or closing bracket before this token.".into()),
        });
    }
}
