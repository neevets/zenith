use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f]+")]
pub enum TokenType {
    #[token("render")]
    Render,
    #[token("function")]
    Function,
    #[token("return")]
    Return,
    #[token("print")]
    Print,
    #[token("println")]
    Println,
    #[token("let")]
    Let,
    #[token("import")]
    Import,
    #[token("before")]
    Before,
    #[token("string")]
    StringType,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("while")]
    While,
    #[token("for")]
    For,
    #[token("in")]
    In,
    #[token("match")]
    Match,
    #[token("route")]
    Route,
    #[token("GET")]
    Get,
    #[token("POST")]
    Post,
    #[token("PUT")]
    Put,
    #[token("DELETE")]
    Delete,
    #[token("PATCH")]
    Patch,
    #[token("query")]
    Query,
    #[token("spawn")]
    Spawn,
    #[token("yield")]
    Yield,
    #[token("enum")]
    Enum,
    #[token("readonly")]
    Readonly,
    #[token("test")]
    Test,
    #[token("struct")]
    Struct,
    #[token("try")]
    Try,
    #[token("catch")]
    Catch,
    #[token("finally")]
    Finally,

    #[token("int")]
    IntType,
    #[token("bool")]
    BoolType,
    #[token("float")]
    FloatType,
    #[token("void")]
    VoidType,
    #[token("any")]
    AnyType,
    #[token("default")]
    Default,
    #[token("<=")]
    Leq,
    #[token(">=")]
    Geq,
    #[token("error")]
    Error,
    #[token("SELECT")]
    Select,
    #[token("FROM")]
    From,
    #[token("WHERE")]
    Where,
    #[token("INSERT")]
    Insert,
    #[token("UPDATE")]
    Update,
    #[token("INTO")]
    Into,
    #[token("VALUES")]
    Values,
    #[token("SET")]
    Set,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident,

    #[regex(r"\$[a-zA-Z_][a-zA-Z0-9_]*")]
    Var,

    #[regex(r"[0-9]+")]
    Int,

    #[regex(r"[0-9]+\.[0-9]+")]
    Float,

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token(",")]
    Comma,
    #[token(";")]
    Semicolon,
    #[token(".")]
    Dot,
    #[token("*")]
    Asterisk,
    #[token("/")]
    Slash,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("#!")]
    HashBang,
    #[token("#[")]
    LBracketHash,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("==")]
    Eq,
    #[token("!=")]
    NotEq,
    #[token("?")]
    Question,
    #[token("??")]
    Coalesce,
    #[token("!")]
    Bang,
    #[token("=")]
    Assign,
    #[token(":")]
    Colon,
    #[token("|>")]
    Pipe,
    #[token("!>")]
    Sanitize,
    #[token("?->")]
    Nullsafe,
    #[token("=>")]
    Arrow,
    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token("@")]
    At,
    #[token("%")]
    Modulo,

    #[regex(r#""([^"\\]|\\.)*""#, |lex| lex.slice()[1..lex.slice().len()-1].to_string())]
    #[regex(r#"'([^'\\]|\\.)*'"#, |lex| lex.slice()[1..lex.slice().len()-1].to_string())]
    Literal(String),

    #[regex(r"//.*", logos::skip)]
    Comment,

    #[regex(r"/\*([^*]|\*[^/])*\*/", logos::skip)]
    MultiLineComment,

    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub literal: String,
    pub span: logos::Span,
}

pub struct Lexer<'a> {
    lexer: logos::Lexer<'a, TokenType>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            lexer: TokenType::lexer(input),
        }
    }

    pub fn next_token(&mut self) -> Token {
        let token_type = self
            .lexer
            .next()
            .unwrap_or(Ok(TokenType::Eof))
            .unwrap_or(TokenType::Eof);
        let span = self.lexer.span();
        let literal = match &token_type {
            TokenType::Literal(s) => s.clone(),
            _ => self.lexer.slice().to_string(),
        };
        Token {
            token_type,
            literal,
            span,
        }
    }
}
