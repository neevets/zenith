use logos::Span;

#[derive(Debug, Clone)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ExpressionKind {
    Identifier(String),
    Variable(String),
    IntegerLiteral(i64),
    FloatLiteral(f64),
    StringLiteral {
        value: String,
        is_render: bool,
        delimiter: char,
    },
    ArrayLiteral(Vec<Expression>),
    MapLiteral(Vec<(Expression, Expression)>),
    PrefixExpression {
        operator: String,
        right: Box<Expression>,
    },
    InfixExpression {
        left: Box<Expression>,
        operator: String,
        right: Box<Expression>,
    },
    IndexExpression {
        left: Box<Expression>,
        index: Box<Expression>,
    },
    CallExpression {
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
    MethodCallExpression {
        object: Box<Expression>,
        method: String,
        arguments: Vec<Expression>,
        is_nullsafe: bool,
    },
    MemberExpression {
        object: Box<Expression>,
        property: String,
        is_nullsafe: bool,
    },
    MatchExpression {
        condition: Box<Expression>,
        arms: Vec<MatchArm>,
    },
    ArrowFunctionExpression {
        parameters: Vec<Parameter>,
        body: Box<Expression>,
        return_type: Option<String>,
    },
    PipeExpression {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    NullCoalesceExpression {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    SpawnExpression {
        body: Box<Statement>,
    },
    AssignExpression {
        left: Box<Expression>,
        value: Box<Expression>,
    },
    SqlQueryExpression {
        query: String,
        args: Vec<Expression>,
        table: Option<String>,
        columns: Vec<String>,
    },
    QueryBlock {
        db: Option<Box<Expression>>,
        query: String,
        args: Vec<Expression>,
    },
    SanitizeExpression {
        left: Box<Expression>,
        sanitizer: Box<Expression>,
    },
    StructLiteral {
        name: String,
        fields: Vec<(String, Expression)>,
    },
    Block(BlockStatement),
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub kind: PatternKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum PatternKind {
    Literal(Expression),
    Identifier(String),
    Struct {
        name: String,
        fields: Vec<(String, Pattern)>,
    },
    Wildcard,
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub patterns: Vec<Pattern>,
    pub result: Expression,
    pub is_default: bool,
}

#[derive(Debug, Clone)]
pub struct Statement {
    pub attributes: Vec<Attribute>,
    pub kind: StatementKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum StatementKind {
    Import(String),
    Let {
        name: String,
        value: Expression,
        var_type: Option<String>,
    },
    Return(Expression),
    Expression(Expression),
    If {
        condition: Expression,
        consequence: BlockStatement,
        alternative: Option<BlockStatement>,
    },
    While {
        condition: Expression,
        body: BlockStatement,
    },
    For {
        variable: String,
        iterable: Expression,
        body: BlockStatement,
    },
    FunctionDefinition {
        name: String,
        parameters: Vec<Parameter>,
        body: BlockStatement,
        return_type: Option<String>,
        is_render: bool,
        is_memoized: bool,
    },
    Enum {
        name: String,
        cases: Vec<EnumCase>,
    },
    Struct {
        name: String,
        parent: Option<String>,
        fields: Vec<StructField>,
    },
    Yield(Option<Expression>),
    Test {
        name: String,
        body: BlockStatement,
    },
    Route {
        method: String,
        path: String,
        body: BlockStatement,
    },
    Middleware(BlockStatement),
}

#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: String,
    pub arguments: Vec<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AttributedStatement {
    pub attributes: Vec<Attribute>,
    pub statement: Statement,
}

#[derive(Debug, Clone)]
pub struct BlockStatement {
    pub statements: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub param_type: Option<String>,
    pub is_var: bool,
}

#[derive(Debug, Clone)]
pub struct EnumCase {
    pub name: String,
    pub value: Option<Expression>,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub field_type: Option<String>,
    pub is_readonly: bool,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub imports: Vec<Statement>,
    pub middleware: Option<BlockStatement>,
    pub statements: Vec<Statement>,
    pub span: Span,
}
