package analyzer

import (
	"strings"
	"github.com/neevets/zenith/src/internal/compiler/parser"
)

type LifeCycleMap struct {
	LastUses map[parser.Statement][]string
	Errors   []string
}

type LifeCycleAnalyzer struct {
	lastUses map[string]parser.Statement
	lcMap    *LifeCycleMap
	sm       *SchemaManager
	tc       *TypeChecker
	inLoop   bool
}

func New() *LifeCycleAnalyzer {
	return &LifeCycleAnalyzer{
		lastUses: make(map[string]parser.Statement),
		lcMap: &LifeCycleMap{
			LastUses: make(map[parser.Statement][]string),
			Errors:   []string{},
		},
		sm: NewSchemaManager("schema.json"),
		tc: NewTypeChecker(),
	}
}

func (a *LifeCycleAnalyzer) Analyze(program *parser.Program) *LifeCycleMap {
	a.traverseProgram(program)
	
	typeErrors := a.tc.Check(program)
	a.lcMap.Errors = append(a.lcMap.Errors, typeErrors...)
	
	for varName, stmt := range a.lastUses {
		a.lcMap.LastUses[stmt] = append(a.lcMap.LastUses[stmt], varName)
	}
	
	return a.lcMap
}

func (a *LifeCycleAnalyzer) traverseProgram(program *parser.Program) {
	for _, stmt := range program.Statements {
		a.analyzeStatementWithSecurity(stmt)
	}
}

func (a *LifeCycleAnalyzer) analyzeStatement(stmt parser.Statement) {
	switch s := stmt.(type) {
	case *parser.LetStatement:
		a.analyzeExpression(s.Value, stmt)
		a.lastUses[s.Name.Value] = stmt
	case *parser.ExpressionStatement:
		a.analyzeExpression(s.Expression, stmt)
	case *parser.ReturnStatement:
		a.analyzeExpression(s.ReturnValue, stmt)
	case *parser.BlockStatement:
		for _, bs := range s.Statements {
			a.analyzeStatement(bs)
		}
	case *parser.IfStatement:
		a.analyzeExpression(s.Condition, stmt)
		a.analyzeStatement(s.Consequence)
		if s.Alternative != nil {
			a.analyzeStatement(s.Alternative)
		}
	}
}

func (a *LifeCycleAnalyzer) analyzeExpression(exp parser.Expression, parentStmt parser.Statement) {
	if exp == nil {
		return
	}
	switch e := exp.(type) {
	case *parser.Variable:
		a.lastUses[e.Value] = parentStmt
	case *parser.CallExpression:
		a.analyzeExpression(e.Function, parentStmt)
		for _, arg := range e.Arguments {
			a.analyzeExpression(arg, parentStmt)
		}
	case *parser.InfixExpression:
		a.analyzeExpression(e.Left, parentStmt)
		a.analyzeExpression(e.Right, parentStmt)
	case *parser.AssignExpression:
		a.analyzeExpression(e.Value, parentStmt)
		a.analyzeExpression(e.Left, parentStmt)
	case *parser.IndexExpression:
		a.analyzeExpression(e.Left, parentStmt)
		a.analyzeExpression(e.Index, parentStmt)
	case *parser.MemberExpression:
		a.analyzeExpression(e.Object, parentStmt)
	case *parser.MethodCallExpression:
		a.analyzeExpression(e.Object, parentStmt)
		for _, arg := range e.Arguments {
			a.analyzeExpression(arg, parentStmt)
		}
	case *parser.ArrayLiteral:
		for _, el := range e.Elements {
			a.analyzeExpression(el, parentStmt)
		}
	case *parser.PipeExpression:
		a.analyzeExpression(e.Left, parentStmt)
		a.analyzeExpression(e.Right, parentStmt)
	}
}

func (a *LifeCycleAnalyzer) checkSecurity(exp parser.Expression) {
	switch e := exp.(type) {
	case *parser.SqlQueryExpression:
		if strings.Contains(e.Query, "$") || strings.Contains(e.Query, "{") {
			a.lcMap.Errors = append(a.lcMap.Errors, "Quantum Shield Alert: Potential SQL Injection detected. Use parameter binding instead of variable interpolation.")
		}
	case *parser.MethodCallExpression:
		if ident, ok := e.Object.(*parser.Identifier); ok && ident.Value == "file" {
			if e.Method.Value == "read" || e.Method.Value == "write" {
				if len(e.Arguments) > 0 {
					if lit, ok := e.Arguments[0].(*parser.StringLiteral); ok {
						if strings.Contains(lit.Value, "..") {
							a.lcMap.Errors = append(a.lcMap.Errors, "Quantum Shield Alert: Potential Path Traversal detected in file access.")
						}
					}
				}
			}
		}
	}
}

func (a *LifeCycleAnalyzer) analyzeStatementWithSecurity(stmt parser.Statement) {
	a.analyzeStatement(stmt)
	switch s := stmt.(type) {
	case *parser.ExpressionStatement:
		a.checkSecurity(s.Expression)
	case *parser.LetStatement:
		a.checkSecurity(s.Value)
	}
}


