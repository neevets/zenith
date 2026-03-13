package parser

import (
	"fmt"
	"strings"
	"strconv"
	"github.com/neevets/zenith/src/internal/compiler/lexer"
)

const (
	_ int = iota
	LOWEST
	PIPE
	ASSIGN
	COALESCE
	COMPARE
	SUM
	PRODUCT
	CALL
	INDEX
	DOT
)

var precedences = map[lexer.TokenType]int{
	lexer.TOKEN_PIPE:     PIPE,
	lexer.TOKEN_ASSIGN:   ASSIGN,
	lexer.TOKEN_COALESCE: COALESCE,
	lexer.TOKEN_LPAREN:   CALL,
	lexer.TOKEN_LBRACKET: INDEX,
	lexer.TOKEN_DOT:      DOT,
	lexer.TOKEN_NULLSAFE: DOT,
	lexer.TOKEN_LT:       COMPARE,
	lexer.TOKEN_GT:       COMPARE,
	lexer.TOKEN_EQ:       COMPARE,
	lexer.TOKEN_NOT_EQ:   COMPARE,
	lexer.TOKEN_PLUS:     SUM,
	lexer.TOKEN_MINUS:    SUM,
}

type Parser struct {
	l         *lexer.Lexer
	curToken  lexer.Token
	peekToken lexer.Token
	errors    []string
	isRender  bool
}

func New(l *lexer.Lexer) *Parser {
	p := &Parser{l: l, errors: []string{}}
	p.nextToken()
	p.nextToken()
	return p
}

func (p *Parser) nextToken() {
	p.curToken = p.peekToken
	p.peekToken = p.l.NextToken()
}

func (p *Parser) ParseProgram() *Program {
	program := &Program{}
	program.Imports = []*ImportStatement{}
	program.Statements = []Statement{}

	for p.curTokenIs(lexer.TOKEN_IMPORT) {
		stmt := p.parseImportStatement()
		if stmt != nil {
			program.Imports = append(program.Imports, stmt)
		}
		p.nextToken()
	}

	if p.curTokenIs(lexer.TOKEN_BEFORE) {
		p.nextToken()
		if p.curTokenIs(lexer.TOKEN_LBRACE) {
			program.Middleware = p.parseBlockStatement()
		}
		p.nextToken()
	}

	for p.curToken.Type != lexer.TOKEN_EOF {
		stmt := p.parseStatement()
		if stmt != nil {
			program.Statements = append(program.Statements, stmt)
		}
		p.nextToken()
	}

	return program
}

func (p *Parser) parseStatement() Statement {
	switch p.curToken.Type {
	case lexer.TOKEN_RENDER:
		return p.parseFunctionDefinition(true)
	case lexer.TOKEN_FUNCTION:
		return p.parseFunctionDefinition(false)
	case lexer.TOKEN_RETURN:
		return p.parseReturnStatement()
	case lexer.TOKEN_LET:
		return p.parseLetStatement()
	case lexer.TOKEN_IF:
		return p.parseIfStatement()
	case lexer.TOKEN_WHILE:
		return p.parseWhileStatement()
	case lexer.TOKEN_FOR:
		return p.parseForStatement()
	case lexer.TOKEN_YIELD:
		return p.parseYieldStatement()
	case lexer.TOKEN_ENUM:
		return p.parseEnumStatement()
	case lexer.TOKEN_IDENT:
		if p.curToken.Literal == "struct" {
			return p.parseStructDefinition()
		}
		fallthrough
	default:
		return p.parseExpressionStatement()
	}
}

func (p *Parser) parseImportStatement() *ImportStatement {
	stmt := &ImportStatement{Token: p.curToken}
	if !p.expectPeek(lexer.TOKEN_LITERAL) {
		return nil
	}
	stmt.Path = p.curToken.Literal
	if p.peekTokenIs(lexer.TOKEN_SEMICOLON) {
		p.nextToken()
	}
	return stmt
}

func (p *Parser) parseLetStatement() *LetStatement {
	stmt := &LetStatement{Token: p.curToken}
	if !p.expectPeek(lexer.TOKEN_VAR) {
		return nil
	}
	stmt.Name = &Variable{Token: p.curToken, Value: p.curToken.Literal}
	if p.peekTokenIs(lexer.TOKEN_COLON) {
		p.nextToken()
		p.nextToken()
		stmt.Type = p.parseType()
	}
	if !p.expectPeek(lexer.TOKEN_ASSIGN) {
		return nil
	}
	p.nextToken()
	stmt.Value = p.ParseExpression(LOWEST)
	if p.peekTokenIs(lexer.TOKEN_SEMICOLON) {
		p.nextToken()
	}
	return stmt
}

func (p *Parser) parseReturnStatement() *ReturnStatement {
	stmt := &ReturnStatement{Token: p.curToken}
	p.nextToken()
	stmt.ReturnValue = p.ParseExpression(0)
	if p.peekTokenIs(lexer.TOKEN_SEMICOLON) {
		p.nextToken()
	}
	return stmt
}

func (p *Parser) parseExpressionStatement() *ExpressionStatement {
	stmt := &ExpressionStatement{Token: p.curToken}
	stmt.Expression = p.ParseExpression(0)
	if p.peekTokenIs(lexer.TOKEN_SEMICOLON) {
		p.nextToken()
	}
	return stmt
}

func (p *Parser) parseBlockStatement() *BlockStatement {
	block := &BlockStatement{Token: p.curToken}
	block.Statements = []Statement{}
	p.nextToken()
	for !p.curTokenIs(lexer.TOKEN_RBRACE) && !p.curTokenIs(lexer.TOKEN_EOF) {
		stmt := p.parseStatement()
		if stmt != nil {
			block.Statements = append(block.Statements, stmt)
		}
		p.nextToken()
	}
	return block
}

func (p *Parser) parseIfStatement() *IfStatement {
	stmt := &IfStatement{Token: p.curToken}
	if !p.expectPeek(lexer.TOKEN_LPAREN) {
		return nil
	}
	p.nextToken()
	stmt.Condition = p.ParseExpression(LOWEST)
	if !p.expectPeek(lexer.TOKEN_RPAREN) {
		return nil
	}
	if !p.expectPeek(lexer.TOKEN_LBRACE) {
		return nil
	}
	stmt.Consequence = p.parseBlockStatement()
	if p.peekTokenIs(lexer.TOKEN_ELSE) {
		p.nextToken()
		if !p.expectPeek(lexer.TOKEN_LBRACE) {
			return nil
		}
		stmt.Alternative = p.parseBlockStatement()
	}
	return stmt
}

func (p *Parser) parseWhileStatement() *WhileStatement {
	stmt := &WhileStatement{Token: p.curToken}
	if !p.expectPeek(lexer.TOKEN_LPAREN) {
		return nil
	}
	p.nextToken()
	stmt.Condition = p.ParseExpression(LOWEST)
	if !p.expectPeek(lexer.TOKEN_RPAREN) {
		return nil
	}
	if !p.expectPeek(lexer.TOKEN_LBRACE) {
		return nil
	}
	stmt.Body = p.parseBlockStatement()
	return stmt
}

func (p *Parser) parseForStatement() *ForStatement {
	stmt := &ForStatement{Token: p.curToken}
	if !p.expectPeek(lexer.TOKEN_VAR) {
		return nil
	}
	stmt.Variable = p.curToken.Literal
	if !p.expectPeek(lexer.TOKEN_IN) {
		return nil
	}
	p.nextToken()
	stmt.Iterable = p.ParseExpression(LOWEST)
	if !p.expectPeek(lexer.TOKEN_LBRACE) {
		return nil
	}
	stmt.Body = p.parseBlockStatement()
	return stmt
}

func (p *Parser) parseYieldStatement() *YieldStatement {
	stmt := &YieldStatement{Token: p.curToken}
	p.nextToken()
	if p.curTokenIs(lexer.TOKEN_SEMICOLON) {
		return stmt
	}
	stmt.Value = p.ParseExpression(LOWEST)
	if p.peekTokenIs(lexer.TOKEN_SEMICOLON) {
		p.nextToken()
	}
	return stmt
}

func (p *Parser) parseFunctionDefinition(isRender bool) Statement {
	stmt := &FunctionDefinition{Token: p.curToken, IsRender: isRender}
	oldIsRender := p.isRender
	p.isRender = isRender
	defer func() { p.isRender = oldIsRender }()
	if isRender {
		if !p.expectPeek(lexer.TOKEN_FUNCTION) {
			return nil
		}
	}
	if !p.expectPeek(lexer.TOKEN_IDENT) {
		return nil
	}
	stmt.Name = &Identifier{Token: p.curToken, Value: p.curToken.Literal}
	if !p.expectPeek(lexer.TOKEN_LPAREN) {
		return nil
	}
	stmt.Parameters = p.parseParameters()
	if p.peekTokenIs(lexer.TOKEN_COLON) {
		p.nextToken()
		p.nextToken()
		stmt.ReturnType = p.parseType()
	}
	if !p.expectPeek(lexer.TOKEN_LBRACE) {
		return nil
	}
	stmt.Body = p.parseBlockStatement()
	return stmt
}

func (p *Parser) parseEnumStatement() *EnumStatement {
	stmt := &EnumStatement{Token: p.curToken}
	if !p.expectPeek(lexer.TOKEN_IDENT) {
		return nil
	}
	stmt.Name = &Identifier{Token: p.curToken, Value: p.curToken.Literal}
	if !p.expectPeek(lexer.TOKEN_LBRACE) {
		return nil
	}
	p.nextToken()
	for !p.curTokenIs(lexer.TOKEN_RBRACE) && !p.curTokenIs(lexer.TOKEN_EOF) {
		caseItem := &EnumCase{Token: p.curToken}
		caseItem.Name = &Identifier{Token: p.curToken, Value: p.curToken.Literal}
		if p.peekTokenIs(lexer.TOKEN_ASSIGN) {
			p.nextToken()
			p.nextToken()
			caseItem.Value = p.ParseExpression(LOWEST)
		}
		stmt.Cases = append(stmt.Cases, caseItem)
		if p.peekTokenIs(lexer.TOKEN_COMMA) {
			p.nextToken()
		}
		p.nextToken()
	}
	return stmt
}

func (p *Parser) parseStructDefinition() *StructDefinition {
	stmt := &StructDefinition{Token: p.curToken}
	if !p.expectPeek(lexer.TOKEN_IDENT) {
		return nil
	}
	stmt.Name = &Identifier{Token: p.curToken, Value: p.curToken.Literal}
	if !p.expectPeek(lexer.TOKEN_LBRACE) {
		return nil
	}
	p.nextToken()
	for !p.curTokenIs(lexer.TOKEN_RBRACE) && !p.curTokenIs(lexer.TOKEN_EOF) {
		field := &StructField{Token: p.curToken}
		if p.curTokenIs(lexer.TOKEN_READONLY) {
			field.IsReadonly = true
			p.nextToken()
		}
		if !p.curTokenIs(lexer.TOKEN_VAR) {
			p.nextToken()
			continue
		}
		field.Name = p.curToken.Literal
		if p.peekTokenIs(lexer.TOKEN_COLON) {
			p.nextToken()
			p.nextToken()
			field.Type = p.parseType()
		}
		stmt.Fields = append(stmt.Fields, field)
		if p.peekTokenIs(lexer.TOKEN_SEMICOLON) {
			p.nextToken()
		}
		p.nextToken()
	}
	return stmt
}

func (p *Parser) parseType() string {
	t := p.curToken.Literal
	for p.peekTokenIs(lexer.TOKEN_IDENT) || p.peekTokenIs(lexer.TOKEN_AND) || p.peekTokenIs(lexer.TOKEN_OR) {
		p.nextToken()
		t += p.curToken.Literal
	}
	return t
}

func (p *Parser) parseParameters() []*Parameter {
	params := []*Parameter{}
	if p.peekTokenIs(lexer.TOKEN_RPAREN) {
		p.nextToken()
		return params
	}
	p.nextToken()
	params = append(params, p.parseParameter())
	for p.peekTokenIs(lexer.TOKEN_COMMA) {
		p.nextToken()
		p.nextToken()
		params = append(params, p.parseParameter())
	}
	if !p.expectPeek(lexer.TOKEN_RPAREN) {
		return nil
	}
	return params
}

func (p *Parser) parseParameter() *Parameter {
	param := &Parameter{}
	if p.curToken.Type == lexer.TOKEN_STRING || p.curToken.Type == lexer.TOKEN_IDENT {
		param.Type = p.curToken.Literal
		if !p.expectPeek(lexer.TOKEN_VAR) {
			return nil
		}
		param.Name = p.curToken.Literal
		param.IsVar = true
	} else if p.curToken.Type == lexer.TOKEN_VAR {
		param.Name = p.curToken.Literal
		param.IsVar = true
		if p.peekTokenIs(lexer.TOKEN_COLON) {
			p.nextToken()
			if p.peekTokenIs(lexer.TOKEN_IDENT) || p.peekTokenIs(lexer.TOKEN_STRING) {
				p.nextToken()
				param.Type = p.parseType()
			}
		}
	}
	return param
}

func (p *Parser) ParseExpression(precedence int) Expression {
	var leftExp Expression
	switch p.curToken.Type {
	case lexer.TOKEN_IDENT:
		leftExp = &Identifier{Token: p.curToken, Value: p.curToken.Literal}
	case lexer.TOKEN_VAR:
		leftExp = &Variable{Token: p.curToken, Value: p.curToken.Literal}
	case lexer.TOKEN_LITERAL:
		leftExp = &StringLiteral{
			Token:     p.curToken,
			Value:     p.curToken.Literal,
			IsRender:  p.isRender,
			Delimiter: p.curToken.Delimiter,
		}
	case lexer.TOKEN_INT:
		leftExp = p.parseIntegerLiteral()
	case lexer.TOKEN_PRINT:
		leftExp = &Identifier{Token: p.curToken, Value: p.curToken.Literal}
	case lexer.TOKEN_LBRACKET:

		leftExp = p.parseArrayLiteral()
	case lexer.TOKEN_MATCH:
		leftExp = p.parseMatchExpression()
	case lexer.TOKEN_FN:
		leftExp = p.parseArrowFunctionExpression()
	case lexer.TOKEN_SPAWN:
		leftExp = p.parseSpawnExpression()
	}
	if leftExp == nil {
		return nil
	}
	for !p.peekTokenIs(lexer.TOKEN_SEMICOLON) && !p.peekTokenIs(lexer.TOKEN_EOF) && precedence < p.peekPrecedence() {
		switch p.peekToken.Type {
		case lexer.TOKEN_DOT, lexer.TOKEN_NULLSAFE:
			p.nextToken()
			leftExp = p.parseMethodCallExpression(leftExp)
		case lexer.TOKEN_LPAREN:
			p.nextToken()
			leftExp = p.parseCallExpression(leftExp)
		case lexer.TOKEN_LBRACKET:
			p.nextToken()
			leftExp = p.parseIndexExpression(leftExp)
		case lexer.TOKEN_COALESCE:
			p.nextToken()
			leftExp = p.parseNullCoalesceExpression(leftExp)
		case lexer.TOKEN_PLUS, lexer.TOKEN_MINUS, lexer.TOKEN_LT, lexer.TOKEN_GT, lexer.TOKEN_EQ, lexer.TOKEN_NOT_EQ:
			p.nextToken()
			leftExp = p.parseInfixExpression(leftExp)
		case lexer.TOKEN_PIPE:
			p.nextToken()
			leftExp = p.parsePipeExpression(leftExp)
		case lexer.TOKEN_ASSIGN:
			p.nextToken()
			leftExp = p.parseAssignExpression(leftExp)
		default:
			return leftExp
		}
	}
	return leftExp
}

func (p *Parser) parseIntegerLiteral() Expression {
	lit := &IntegerLiteral{Token: p.curToken}
	value, err := strconv.ParseInt(p.curToken.Literal, 0, 64)
	if err != nil {
		p.errors = append(p.errors, fmt.Sprintf("could not parse %q as integer", p.curToken.Literal))
		return nil
	}
	lit.Value = value
	return lit
}

func (p *Parser) parseInfixExpression(left Expression) Expression {
	exp := &InfixExpression{
		Token:    p.curToken,
		Operator: p.curToken.Literal,
		Left:     left,
	}
	precedence := p.curPrecedence()
	p.nextToken()
	exp.Right = p.ParseExpression(precedence)
	return exp
}

func (p *Parser) parseIndexExpression(left Expression) Expression {
	exp := &IndexExpression{Token: p.curToken, Left: left}
	p.nextToken()
	exp.Index = p.ParseExpression(LOWEST)
	if !p.expectPeek(lexer.TOKEN_RBRACKET) {
		return nil
	}
	return exp
}

func (p *Parser) parseCallExpression(function Expression) Expression {
	exp := &CallExpression{Token: p.curToken, Function: function}
	exp.Arguments = p.parseExpressionList(lexer.TOKEN_RPAREN)
	return exp
}

func (p *Parser) parseMethodCallExpression(object Expression) Expression {
	token := p.curToken
	isNullsafe := token.Type == lexer.TOKEN_NULLSAFE
	if !p.expectPeek(lexer.TOKEN_IDENT) {
		return nil
	}
	methodName := p.curToken.Literal
	methodIdent := &Identifier{Token: p.curToken, Value: methodName}
	if !p.peekTokenIs(lexer.TOKEN_LPAREN) {
		return &MemberExpression{Token: token, Object: object, Property: methodIdent, IsNullsafe: isNullsafe}
	}
	p.nextToken()
	if ident, ok := object.(*Identifier); ok && ident.Value == "db" && methodName == "query" {
		return p.parseSqlQueryExpression(p.curToken)
	}
	exp := &MethodCallExpression{Token: token, Object: object, Method: methodIdent, IsNullsafe: isNullsafe}
	exp.Arguments = p.parseExpressionList(lexer.TOKEN_RPAREN)
	return exp
}

func (p *Parser) parseSqlQueryExpression(token lexer.Token) Expression {
	exp := &SqlQueryExpression{Token: token, Args: []Expression{}, Columns: []string{}}
	var query strings.Builder
	p.nextToken()
	isSelect, isFrom := false, false
	for !p.curTokenIs(lexer.TOKEN_RPAREN) && !p.curTokenIs(lexer.TOKEN_EOF) {
		literal := p.curToken.Literal
		if strings.ToUpper(literal) == "SELECT" {
			isSelect = true
		} else if strings.ToUpper(literal) == "FROM" {
			isSelect, isFrom = false, true
		} else if isSelect && literal != "," {
			exp.Columns = append(exp.Columns, literal)
		} else if isFrom {
			exp.Table, isFrom = literal, false
		}
		if p.curTokenIs(lexer.TOKEN_LBRACE) {
			p.nextToken()
			exp.Args = append(exp.Args, p.ParseExpression(LOWEST))
			if query.Len() > 0 {
				query.WriteString(" ")
			}
			query.WriteString("?")
			if !p.expectPeek(lexer.TOKEN_RBRACE) {
				return nil
			}
		} else {
			if query.Len() > 0 && p.curToken.Literal != "," && p.curToken.Literal != "." {
				query.WriteString(" ")
			}
			literal := p.curToken.Literal
			if p.curToken.Type == lexer.TOKEN_VAR {
				literal = "$" + literal
			}
			query.WriteString(literal)
		}
		p.nextToken()
	}
	exp.Query = query.String()
	return exp
}

func (p *Parser) parseArrowFunctionExpression() Expression {
	exp := &ArrowFunctionExpression{Token: p.curToken}
	if !p.expectPeek(lexer.TOKEN_LPAREN) {
		return nil
	}
	exp.Parameters = p.parseParameters()
	if p.peekTokenIs(lexer.TOKEN_COLON) {
		p.nextToken()
		p.nextToken()
		exp.ReturnType = p.parseType()
	}
	if !p.expectPeek(lexer.TOKEN_ARROW) {
		return nil
	}
	p.nextToken()
	exp.Body = p.ParseExpression(LOWEST)
	return exp
}

func (p *Parser) parseSpawnExpression() Expression {
	exp := &SpawnExpression{Token: p.curToken}
	if p.peekTokenIs(lexer.TOKEN_LBRACE) {
		p.nextToken()
		exp.Body = p.parseBlockStatement()
	} else {
		p.nextToken()
		exp.Body = p.ParseExpression(LOWEST)
	}
	return exp
}

func (p *Parser) parseMatchExpression() Expression {
	exp := &MatchExpression{Token: p.curToken}
	if !p.expectPeek(lexer.TOKEN_LPAREN) {
		return nil
	}
	p.nextToken()
	exp.Condition = p.ParseExpression(LOWEST)
	if !p.expectPeek(lexer.TOKEN_RPAREN) {
		return nil
	}
	if !p.expectPeek(lexer.TOKEN_LBRACE) {
		return nil
	}
	p.nextToken()
	exp.Arms = []*MatchArm{}
	for !p.curTokenIs(lexer.TOKEN_RBRACE) && !p.curTokenIs(lexer.TOKEN_EOF) {
		arm := &MatchArm{Token: p.curToken}
		if p.curToken.Literal == "default" {
			arm.IsDefault = true
		} else {
			arm.Values = []Expression{p.ParseExpression(LOWEST)}
			for p.peekTokenIs(lexer.TOKEN_COMMA) {
				p.nextToken()
				p.nextToken()
				arm.Values = append(arm.Values, p.ParseExpression(LOWEST))
			}
		}
		if !p.expectPeek(lexer.TOKEN_ARROW) {
			return nil
		}
		p.nextToken()
		arm.Result = p.ParseExpression(LOWEST)
		exp.Arms = append(exp.Arms, arm)
		if p.peekTokenIs(lexer.TOKEN_COMMA) {
			p.nextToken()
		}
		p.nextToken()
	}
	return exp
}

func (p *Parser) parsePipeExpression(left Expression) Expression {
	exp := &PipeExpression{Token: p.curToken, Left: left}
	precedence := p.curPrecedence()
	p.nextToken()
	exp.Right = p.ParseExpression(precedence)
	return exp
}

func (p *Parser) parseAssignExpression(left Expression) Expression {
	exp := &AssignExpression{Token: p.curToken, Left: left}
	precedence := p.curPrecedence()
	p.nextToken()
	exp.Value = p.ParseExpression(precedence - 1)
	return exp
}

func (p *Parser) parseArrayLiteral() Expression {
	array := &ArrayLiteral{Token: p.curToken}
	array.Elements = p.parseExpressionList(lexer.TOKEN_RBRACKET)
	return array
}

func (p *Parser) parseNullCoalesceExpression(left Expression) Expression {
	exp := &NullCoalesceExpression{Token: p.curToken, Left: left}
	precedence := p.curPrecedence()
	p.nextToken()
	exp.Right = p.ParseExpression(precedence)
	return exp
}

func (p *Parser) parseExpressionList(end lexer.TokenType) []Expression {
	list := []Expression{}
	if p.peekTokenIs(end) {
		p.nextToken()
		return list
	}
	p.nextToken()
	list = append(list, p.ParseExpression(0))
	for p.peekTokenIs(lexer.TOKEN_COMMA) {
		p.nextToken()
		p.nextToken()
		list = append(list, p.ParseExpression(0))
	}
	if !p.expectPeek(end) {
		return nil
	}
	return list
}

func (p *Parser) curTokenIs(t lexer.TokenType) bool {
	return p.curToken.Type == t
}

func (p *Parser) peekTokenIs(t lexer.TokenType) bool {
	return p.peekToken.Type == t
}

func (p *Parser) expectPeek(t lexer.TokenType) bool {
	if p.peekTokenIs(t) {
		p.nextToken()
		return true
	}
	p.peekError(t)
	return false
}

func (p *Parser) peekError(t lexer.TokenType) {
	p.errors = append(p.errors, fmt.Sprintf("expected next token to be %s, got %s instead", t, p.peekToken.Type))
}

func (p *Parser) Errors() []string {
	return p.errors
}

func (p *Parser) peekPrecedence() int {
	if prec, ok := precedences[p.peekToken.Type]; ok {
		return prec
	}
	return LOWEST
}

func (p *Parser) curPrecedence() int {
	if prec, ok := precedences[p.curToken.Type]; ok {
		return prec
	}
	return LOWEST
}
