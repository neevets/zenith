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
    #[token("fn")]
    Fn,
    #[token("spawn")]
    Spawn,
    #[token("yield")]
    Yield,
    #[token("enum")]
    Enum,
    #[token("readonly")]
    Readonly,

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
    #[token("error")]
    Error,
    #[token("SELECT")]
    Select,
    #[token("FROM")]
    From,
    #[token("WHERE")]
    Where,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident,

    #[regex(r"\$[a-zA-Z_][a-zA-Z0-9_]*")]
    Var,

    #[regex(r"[0-9]+")]
    Int,

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
    #[token("?->")]
    Nullsafe,
    #[token("=>")]
    Arrow,
    #[token("&&")]
    And,
    #[token("||")]
    Or,

    #[regex(r#""([^"\\]|\\.)*""#, |lex| lex.slice()[1..lex.slice().len()-1].to_string())]
    #[regex(r#"'([^'\\]|\\.)*'"#, |lex| lex.slice()[1..lex.slice().len()-1].to_string())]
    Literal(String),

    #[regex(r"//.*", logos::skip)]
    Comment,

    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub literal: String,
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
        let token_type = self.lexer.next().unwrap_or(Ok(TokenType::Eof)).unwrap_or(TokenType::Eof);
        let literal = match &token_type {
            TokenType::Literal(s) => s.clone(),
            _ => self.lexer.slice().to_string(),
        };
        Token {
            token_type,
            literal,
        }
    }
}
