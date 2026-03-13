package analyzer

import (
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
		lcMap:    &LifeCycleMap{
			LastUses: make(map[parser.Statement][]string),
			Errors:   []string{},
		},
		sm: NewSchemaManager("zenith.schema.json"),
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
	if program.Middleware != nil {
		for _, stmt := range program.Middleware.Statements {
			a.analyzeStatement(stmt)
		}
	}
	for _, stmt := range program.Statements {
		a.analyzeStatement(stmt)
	}
}

func (a *LifeCycleAnalyzer) analyzeStatement(stmt parser.Statement) {
	switch s := stmt.(type) {
	case *parser.LetStatement:
		a.analyzeExpression(s.Value, stmt)
	case *parser.ExpressionStatement:
		a.analyzeExpression(s.Expression, stmt)
	case *parser.ReturnStatement:
		a.analyzeExpression(s.ReturnValue, stmt)
	case *parser.FunctionDefinition:
		innerAnalyzer := New()
		innerMap := innerAnalyzer.AnalyzeBlock(s.Body)
		for innerStmt, variables := range innerMap.LastUses {
			a.lcMap.LastUses[innerStmt] = variables
		}
		a.lcMap.Errors = append(a.lcMap.Errors, innerMap.Errors...)
	case *parser.IfStatement:
		a.analyzeExpression(s.Condition, stmt)
		a.AnalyzeBlock(s.Consequence)
		if s.Alternative != nil {
			a.AnalyzeBlock(s.Alternative)
		}
	case *parser.WhileStatement:
		a.analyzeExpression(s.Condition, stmt)
		oldInLoop := a.inLoop
		a.inLoop = true
		for _, loopStmt := range s.Body.Statements {
			a.analyzeStatement(loopStmt)
		}
		a.inLoop = oldInLoop
	}
}

func (a *LifeCycleAnalyzer) AnalyzeBlock(block *parser.BlockStatement) *LifeCycleMap {
	for _, stmt := range block.Statements {
		a.analyzeStatement(stmt)
	}
	
	for varName, stmt := range a.lastUses {
		a.lcMap.LastUses[stmt] = append(a.lcMap.LastUses[stmt], varName)
	}
	
	return a.lcMap
}

func (a *LifeCycleAnalyzer) analyzeExpression(exp parser.Expression, parentStmt parser.Statement) {
	switch e := exp.(type) {
	case *parser.Variable:
		if !a.inLoop {
			a.lastUses[e.Value] = parentStmt
		}
	case *parser.CallExpression:
		for _, arg := range e.Arguments {
			a.analyzeExpression(arg, parentStmt)
		}
		a.analyzeExpression(e.Function, parentStmt)
	case *parser.MethodCallExpression:
		a.analyzeExpression(e.Object, parentStmt)
		for _, arg := range e.Arguments {
			a.analyzeExpression(arg, parentStmt)
		}
	case *parser.NullCoalesceExpression:
		a.analyzeExpression(e.Left, parentStmt)
		a.analyzeExpression(e.Right, parentStmt)
	case *parser.SqlQueryExpression:
		for _, arg := range e.Args {
			a.analyzeExpression(arg, parentStmt)
		}
		if a.sm != nil && e.Table != "" {
			errs := a.sm.ValidateQuery(e.Table, e.Columns)
			a.lcMap.Errors = append(a.lcMap.Errors, errs...)
		}
	case *parser.ArrayLiteral:
		for _, element := range e.Elements {
			a.analyzeExpression(element, parentStmt)
		}
	case *parser.MemberExpression:
		a.analyzeExpression(e.Object, parentStmt)
	case *parser.InfixExpression:
		a.analyzeExpression(e.Left, parentStmt)
		a.analyzeExpression(e.Right, parentStmt)
	case *parser.AssignExpression:
		a.analyzeExpression(e.Value, parentStmt)
		a.analyzeExpression(e.Left, parentStmt)
	}
}
