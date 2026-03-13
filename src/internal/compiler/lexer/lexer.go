package lexer

type TokenType string

const (
	TOKEN_RENDER    TokenType = "RENDER"
	TOKEN_FUNCTION  TokenType = "FUNCTION"
	TOKEN_RETURN    TokenType = "RETURN"
	TOKEN_PRINT     TokenType = "PRINT"
	TOKEN_LET       TokenType = "LET"
	TOKEN_IMPORT    TokenType = "IMPORT"
	TOKEN_BEFORE    TokenType = "BEFORE"
	TOKEN_STRING    TokenType = "STRING_TYPE"
	TOKEN_IF        TokenType = "IF"
	TOKEN_ELSE      TokenType = "ELSE"
	TOKEN_WHILE     TokenType = "WHILE"
	TOKEN_IDENT     TokenType = "IDENT"
	TOKEN_VAR       TokenType = "VAR"
	TOKEN_LITERAL   TokenType = "LITERAL"
	TOKEN_INT       TokenType = "INT"
	TOKEN_LPAREN    TokenType = "LPAREN"
	TOKEN_RPAREN    TokenType = "RPAREN"
	TOKEN_LBRACE    TokenType = "LBRACE"
	TOKEN_RBRACE    TokenType = "RBRACE"
	TOKEN_COMMA     TokenType = "COMMA"
	TOKEN_SEMICOLON TokenType = "SEMICOLON"
	TOKEN_DOT       TokenType = "DOT"
	TOKEN_ASTERISK  TokenType = "ASTERISK"
	TOKEN_LBRACKET  TokenType = "LBRACKET"
	TOKEN_RBRACKET  TokenType = "RBRACKET"
	TOKEN_PLUS      TokenType = "PLUS"
	TOKEN_MINUS     TokenType = "MINUS"
	TOKEN_LT        TokenType = "LT"
	TOKEN_GT        TokenType = "GT"
	TOKEN_EQ        TokenType = "EQ"
	TOKEN_NOT_EQ    TokenType = "NOT_EQ"
	TOKEN_QUESTION  TokenType = "QUESTION"
	TOKEN_COALESCE  TokenType = "COALESCE"
	TOKEN_BANG      TokenType = "BANG"
	TOKEN_ASSIGN    TokenType = "ASSIGN"
	TOKEN_COLON     TokenType = "COLON"
	TOKEN_PIPE      TokenType = "PIPE"
	TOKEN_NULLSAFE  TokenType = "NULLSAFE"
	TOKEN_MATCH     TokenType = "MATCH"
	TOKEN_ARROW     TokenType = "ARROW"
	TOKEN_FN        TokenType = "FN"
	TOKEN_SPAWN     TokenType = "SPAWN"
	TOKEN_YIELD     TokenType = "YIELD"
	TOKEN_ENUM      TokenType = "ENUM"
	TOKEN_READONLY  TokenType = "READONLY"
	TOKEN_AND       TokenType = "AND"
	TOKEN_OR        TokenType = "OR"
	TOKEN_EOF       TokenType = "EOF"
	TOKEN_ILLEGAL   TokenType = "ILLEGAL"
)

type Token struct {
	Type      TokenType
	Literal   string
	Delimiter byte
}

type Lexer struct {
	input        string
	position     int
	readPosition int
	ch           byte
}

func New(input string) *Lexer {
	l := &Lexer{input: input}
	l.readChar()
	return l
}

func (l *Lexer) readChar() {
	if l.readPosition >= len(l.input) {
		l.ch = 0
	} else {
		l.ch = l.input[l.readPosition]
	}
	l.position = l.readPosition
	l.readPosition++
}

func (l *Lexer) NextToken() Token {
	var tok Token
	l.skipWhitespace()
	switch l.ch {
	case '(':
		tok = Token{Type: TOKEN_LPAREN, Literal: string(l.ch)}
	case ')':
		tok = Token{Type: TOKEN_RPAREN, Literal: string(l.ch)}
	case '{':
		tok = Token{Type: TOKEN_LBRACE, Literal: string(l.ch)}
	case '}':
		tok = Token{Type: TOKEN_RBRACE, Literal: string(l.ch)}
	case ':':
		tok = Token{Type: TOKEN_COLON, Literal: string(l.ch)}
	case ',':
		tok = Token{Type: TOKEN_COMMA, Literal: string(l.ch)}
	case '[':
		tok = Token{Type: TOKEN_LBRACKET, Literal: string(l.ch)}
	case ']':
		tok = Token{Type: TOKEN_RBRACKET, Literal: string(l.ch)}
	case '+':
		tok = Token{Type: TOKEN_PLUS, Literal: string(l.ch)}
	case '*':
		tok = Token{Type: TOKEN_ASTERISK, Literal: string(l.ch)}
	case ';':
		tok = Token{Type: TOKEN_SEMICOLON, Literal: string(l.ch)}
	case '/':
		if l.peekChar() == '/' {
			l.skipComment()
			return l.NextToken()
		}
		tok = Token{Type: TOKEN_IDENT, Literal: string(l.ch)}
	case '.':
		tok = Token{Type: TOKEN_DOT, Literal: string(l.ch)}
	case '?':
		if l.peekChar() == '?' {
			ch := l.ch
			l.readChar()
			tok = Token{Type: TOKEN_COALESCE, Literal: string(ch) + string(l.ch)}
		} else if l.peekChar() == '-' {
			if l.peekPeekChar() == '>' {
				ch := l.ch
				l.readChar()
				l.readChar()
				tok = Token{Type: TOKEN_NULLSAFE, Literal: string(ch) + "->"}
			} else {
				tok = Token{Type: TOKEN_QUESTION, Literal: string(l.ch)}
			}
		} else {
			tok = Token{Type: TOKEN_QUESTION, Literal: string(l.ch)}
		}
	case '-':
		tok = Token{Type: TOKEN_MINUS, Literal: string(l.ch)}
	case '<':
		tok = Token{Type: TOKEN_LT, Literal: string(l.ch)}
	case '>':
		tok = Token{Type: TOKEN_GT, Literal: string(l.ch)}
	case '!':
		if l.peekChar() == '=' {
			ch := l.ch
			l.readChar()
			tok = Token{Type: TOKEN_NOT_EQ, Literal: string(ch) + string(l.ch)}
		} else {
			tok = Token{Type: TOKEN_BANG, Literal: string(l.ch)}
		}
	case '|':
		if l.peekChar() == '>' {
			ch := l.ch
			l.readChar()
			tok = Token{Type: TOKEN_PIPE, Literal: string(ch) + string(l.ch)}
		} else {
			tok = Token{Type: TOKEN_OR, Literal: string(l.ch)}
		}
	case '&':
		tok = Token{Type: TOKEN_AND, Literal: string(l.ch)}
	case '=':
		if l.peekChar() == '=' {
			ch := l.ch
			l.readChar()
			tok = Token{Type: TOKEN_EQ, Literal: string(ch) + string(l.ch)}
		} else if l.peekChar() == '>' {
			ch := l.ch
			l.readChar()
			tok = Token{Type: TOKEN_ARROW, Literal: string(ch) + string(l.ch)}
		} else {
			tok = Token{Type: TOKEN_ASSIGN, Literal: string(l.ch)}
		}
	case '$':
		tok.Type = TOKEN_VAR
		l.readChar()
		tok.Literal = l.readIdentifier()
		return tok
	case '"':
		tok.Type = TOKEN_LITERAL
		tok.Delimiter = '"'
		tok.Literal = l.readQuotedString('"')
	case '\'':
		tok.Type = TOKEN_LITERAL
		tok.Delimiter = '\''
		tok.Literal = l.readQuotedString('\'')
	case 0:
		tok.Type = TOKEN_EOF
		tok.Literal = ""
	default:
		if isLetter(l.ch) {
			tok.Literal = l.readIdentifier()
			tok.Type = lookupIdent(tok.Literal)
			return tok
		} else if isDigit(l.ch) {
			tok.Type = TOKEN_INT
			tok.Literal = l.readNumber()
			return tok
		} else {
			tok = Token{Type: TOKEN_ILLEGAL, Literal: string(l.ch)}
		}
	}
	l.readChar()
	return tok
}

func (l *Lexer) readIdentifier() string {
	position := l.position
	for isLetter(l.ch) || isDigit(l.ch) {
		l.readChar()
	}
	return l.input[position:l.position]
}

func (l *Lexer) readQuotedString(delimiter byte) string {
	position := l.position + 1
	for {
		l.readChar()
		if l.ch == '\\' {
			l.readChar()
			continue
		}
		if l.ch == delimiter || l.ch == 0 {
			break
		}
	}
	return l.input[position:l.position]
}

func (l *Lexer) peekChar() byte {
	if l.readPosition >= len(l.input) {
		return 0
	}
	return l.input[l.readPosition]
}

func (l *Lexer) peekPeekChar() byte {
	if l.readPosition+1 >= len(l.input) {
		return 0
	}
	return l.input[l.readPosition+1]
}

func (l *Lexer) skipWhitespace() {
	for l.ch == ' ' || l.ch == '\t' || l.ch == '\n' || l.ch == '\r' {
		l.readChar()
	}
}

func (l *Lexer) skipComment() {
	for l.ch != '\n' && l.ch != 0 {
		l.readChar()
	}
}

func (l *Lexer) readNumber() string {
	position := l.position
	for isDigit(l.ch) {
		l.readChar()
	}
	return l.input[position:l.position]
}

func isLetter(ch byte) bool {
	return 'a' <= ch && ch <= 'z' || 'A' <= ch && ch <= 'Z' || ch == '_'
}

func isDigit(ch byte) bool {
	return '0' <= ch && ch <= '9'
}

var keywords = map[string]TokenType{
	"render":   TOKEN_RENDER,
	"function": TOKEN_FUNCTION,
	"return":   TOKEN_RETURN,
	"print":    TOKEN_PRINT,
	"let":      TOKEN_LET,
	"import":   TOKEN_IMPORT,
	"before":   TOKEN_BEFORE,
	"string":   TOKEN_STRING,
	"int":      TOKEN_IDENT,
	"bool":     TOKEN_IDENT,
	"float":    TOKEN_IDENT,
	"void":     TOKEN_IDENT,
	"any":      TOKEN_IDENT,
	"match":    TOKEN_MATCH,
	"fn":       TOKEN_FN,
	"spawn":    TOKEN_SPAWN,
	"yield":    TOKEN_YIELD,
	"enum":     TOKEN_ENUM,
	"readonly": TOKEN_READONLY,
	"default":  TOKEN_IDENT,
	"if":       TOKEN_IF,
	"else":     TOKEN_ELSE,
	"while":    TOKEN_WHILE,
	"error":    TOKEN_IDENT,
	"SELECT":   TOKEN_IDENT,
	"FROM":     TOKEN_IDENT,
	"WHERE":    TOKEN_IDENT,
}

func lookupIdent(ident string) TokenType {
	if tok, ok := keywords[ident]; ok {
		return tok
	}
	return TOKEN_IDENT
}
