package parser

import (
	"github.com/neevets/zenith/src/internal/compiler/lexer"
)

type Node interface {
	TokenLiteral() string
}

type Statement interface {
	Node
	statementNode()
}

type Expression interface {
	Node
	expressionNode()
}

type Program struct {
	Imports    []*ImportStatement
	Middleware *BlockStatement
	Statements []Statement
}

func (p *Program) TokenLiteral() string {
	if len(p.Statements) > 0 {
		return p.Statements[0].TokenLiteral()
	}
	return ""
}

type FunctionDefinition struct {
	Token      lexer.Token
	IsRender   bool
	Name       *Identifier
	Parameters []*Parameter
	ReturnType string
	Body       *BlockStatement
}

func (f *FunctionDefinition) statementNode()       {}
func (f *FunctionDefinition) TokenLiteral() string { return f.Token.Literal }

type ImportStatement struct {
	Token lexer.Token
	Path  string
}

func (is *ImportStatement) statementNode()       {}
func (is *ImportStatement) TokenLiteral() string { return is.Token.Literal }

type LetStatement struct {
	Token lexer.Token
	Name  *Variable
	Type  string
	Value Expression
}

func (ls *LetStatement) statementNode()       {}
func (ls *LetStatement) TokenLiteral() string { return ls.Token.Literal }

type Parameter struct {
	Type  string
	Name  string
	IsVar bool
}

type BlockStatement struct {
	Token      lexer.Token
	Statements []Statement
}

func (bs *BlockStatement) statementNode()       {}
func (bs *BlockStatement) TokenLiteral() string { return bs.Token.Literal }

type ReturnStatement struct {
	Token       lexer.Token
	ReturnValue Expression
}

func (rs *ReturnStatement) statementNode()       {}
func (rs *ReturnStatement) TokenLiteral() string { return rs.Token.Literal }

type ExpressionStatement struct {
	Token      lexer.Token
	Expression Expression
}

func (es *ExpressionStatement) statementNode()       {}
func (es *ExpressionStatement) TokenLiteral() string { return es.Token.Literal }

type Identifier struct {
	Token lexer.Token
	Value string
}

func (i *Identifier) expressionNode()      {}
func (i *Identifier) TokenLiteral() string { return i.Token.Literal }

type Variable struct {
	Token lexer.Token
	Value string
}

func (v *Variable) expressionNode()      {}
func (v *Variable) TokenLiteral() string { return v.Token.Literal }

type StringLiteral struct {
	Token     lexer.Token
	Value     string
	IsRender  bool
	Delimiter byte
}

func (sl *StringLiteral) expressionNode()      {}
func (sl *StringLiteral) TokenLiteral() string { return sl.Token.Literal }

type IntegerLiteral struct {
	Token lexer.Token
	Value int64
}

func (il *IntegerLiteral) expressionNode()      {}
func (il *IntegerLiteral) TokenLiteral() string { return il.Token.Literal }

type CallExpression struct {
	Token     lexer.Token
	Function  Expression
	Arguments []Expression
}

func (ce *CallExpression) expressionNode()      {}
func (ce *CallExpression) TokenLiteral() string { return ce.Token.Literal }

type MethodCallExpression struct {
	Token      lexer.Token
	Object     Expression
	Method     *Identifier
	Arguments  []Expression
	IsNullsafe bool
}

func (mce *MethodCallExpression) expressionNode()      {}
func (mce *MethodCallExpression) TokenLiteral() string { return mce.Token.Literal }

type NullCoalesceExpression struct {
	Token lexer.Token
	Left  Expression
	Right Expression
}

func (nce *NullCoalesceExpression) expressionNode()      {}
func (nce *NullCoalesceExpression) TokenLiteral() string { return nce.Token.Literal }

type SqlQueryExpression struct {
	Token   lexer.Token
	Query   string
	Table   string
	Columns []string
	Args    []Expression
}

func (sqe *SqlQueryExpression) expressionNode()      {}
func (sqe *SqlQueryExpression) TokenLiteral() string { return sqe.Token.Literal }

type ArrayLiteral struct {
	Token    lexer.Token
	Elements []Expression
}

func (al *ArrayLiteral) expressionNode()      {}
func (al *ArrayLiteral) TokenLiteral() string { return al.Token.Literal }

type MemberExpression struct {
	Token      lexer.Token
	Object     Expression
	Property   *Identifier
	IsNullsafe bool
}

func (me *MemberExpression) expressionNode()      {}
func (me *MemberExpression) TokenLiteral() string { return me.Token.Literal }

type IndexExpression struct {
	Token lexer.Token
	Left  Expression
	Index Expression
}

func (ie *IndexExpression) expressionNode()      {}
func (ie *IndexExpression) TokenLiteral() string { return ie.Token.Literal }

type InfixExpression struct {
	Token    lexer.Token
	Left     Expression
	Operator string
	Right    Expression
}

func (ie *InfixExpression) expressionNode()      {}
func (ie *InfixExpression) TokenLiteral() string { return ie.Token.Literal }

type IfStatement struct {
	Token       lexer.Token
	Condition   Expression
	Consequence *BlockStatement
	Alternative *BlockStatement
}

func (is *IfStatement) statementNode()       {}
func (is *IfStatement) TokenLiteral() string { return is.Token.Literal }

type WhileStatement struct {
	Token     lexer.Token
	Condition Expression
	Body      *BlockStatement
}

func (ws *WhileStatement) statementNode()       {}
func (ws *WhileStatement) TokenLiteral() string { return ws.Token.Literal }

type ForStatement struct {
	Token    lexer.Token
	Variable string
	Iterable Expression
	Body     *BlockStatement
}

func (fs *ForStatement) statementNode()       {}
func (fs *ForStatement) TokenLiteral() string { return fs.Token.Literal }

type AssignExpression struct {
	Token lexer.Token
	Left  Expression
	Value Expression
}

func (ae *AssignExpression) expressionNode()      {}
func (ae *AssignExpression) TokenLiteral() string { return ae.Token.Literal }

type PipeExpression struct {
	Token lexer.Token
	Left  Expression
	Right Expression
}

func (pe *PipeExpression) expressionNode()      {}
func (pe *PipeExpression) TokenLiteral() string { return pe.Token.Literal }

type MatchExpression struct {
	Token     lexer.Token
	Condition Expression
	Arms      []*MatchArm
}

func (me *MatchExpression) expressionNode()      {}
func (me *MatchExpression) TokenLiteral() string { return me.Token.Literal }

type MatchArm struct {
	Token     lexer.Token
	Values    []Expression
	IsDefault bool
	Result    Expression
}

type ArrowFunctionExpression struct {
	Token      lexer.Token
	Parameters []*Parameter
	ReturnType string
	Body       Expression
}

func (afe *ArrowFunctionExpression) expressionNode()      {}
func (afe *ArrowFunctionExpression) TokenLiteral() string { return afe.Token.Literal }

type SpawnExpression struct {
	Token lexer.Token
	Body  Node
}

func (se *SpawnExpression) expressionNode()      {}
func (se *SpawnExpression) TokenLiteral() string { return se.Token.Literal }

type YieldStatement struct {
	Token lexer.Token
	Value Expression
}

func (ys *YieldStatement) statementNode()       {}
func (ys *YieldStatement) TokenLiteral() string { return ys.Token.Literal }

type EnumStatement struct {
	Token lexer.Token
	Name  *Identifier
	Cases []*EnumCase
}

func (es *EnumStatement) statementNode()       {}
func (es *EnumStatement) TokenLiteral() string { return es.Token.Literal }

type EnumCase struct {
	Token lexer.Token
	Name  *Identifier
	Value Expression
}

func (ec *EnumCase) expressionNode()      {}
func (ec *EnumCase) TokenLiteral() string { return ec.Token.Literal }

type StructDefinition struct {
	Token  lexer.Token
	Name   *Identifier
	Fields []*StructField
}

func (sd *StructDefinition) statementNode()       {}
func (sd *StructDefinition) TokenLiteral() string { return sd.Token.Literal }

type StructField struct {
	Token      lexer.Token
	Name       string
	Type       string
	IsReadonly bool
}
