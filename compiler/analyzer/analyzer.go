package analyzer

import (
	"fmt"
	"github.com/neevets/zenith/compiler/lexer"
	"github.com/neevets/zenith/compiler/parser"
	"strings"
)

type LifeCycleMap struct {
	LastUses map[parser.Statement][]string
	Errors   []string
}

type LifeCycleAnalyzer struct {
	lastUses map[string]parser.Statement
	lcMap    *LifeCycleMap
	sm       *SchemaManager
	inLoop   bool
	varTypes map[string]string
	funcSigs map[string]funcSignature
}

type funcSignature struct {
	ParamTypes []string
	ReturnType string
}

func New() *LifeCycleAnalyzer {
	return &LifeCycleAnalyzer{
		lastUses: make(map[string]parser.Statement),
		lcMap: &LifeCycleMap{
			LastUses: make(map[parser.Statement][]string),
			Errors:   []string{},
		},
		sm:       NewSchemaManager("zenith.schema.json"),
		varTypes: make(map[string]string),
		funcSigs: make(map[string]funcSignature),
	}
}

func (a *LifeCycleAnalyzer) Analyze(program *parser.Program) *LifeCycleMap {
	a.traverseProgram(program)

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
		if s.Type != "" {
			a.varTypes[s.Name.Value] = s.Type
		}
		a.analyzeExpression(s.Value, stmt)
		if got := a.inferType(s.Value); s.Type != "" && !a.typeCompatible(s.Type, got) {
			a.lcMap.Errors = append(a.lcMap.Errors, a.errorAtToken(s.Token, "type mismatch in let $"+s.Name.Value+": expected "+s.Type+", got "+got))
		}
		if s.Type == "" {
			a.varTypes[s.Name.Value] = a.inferType(s.Value)
		}
	case *parser.ExpressionStatement:
		a.analyzeExpression(s.Expression, stmt)
	case *parser.ReturnStatement:
		a.analyzeExpression(s.ReturnValue, stmt)
	case *parser.FunctionDefinition:
		paramTypes := []string{}
		for _, p := range s.Parameters {
			if p.Type != "" {
				paramTypes = append(paramTypes, p.Type)
			} else {
				paramTypes = append(paramTypes, "any")
			}
		}
		a.funcSigs[s.Name.Value] = funcSignature{ParamTypes: paramTypes, ReturnType: emptyAsAny(s.ReturnType)}
		innerAnalyzer := New()
		innerAnalyzer.funcSigs = a.funcSigs
		for _, p := range s.Parameters {
			if p.IsVar {
				innerAnalyzer.varTypes[p.Name] = emptyAsAny(p.Type)
			}
		}
		innerMap := innerAnalyzer.AnalyzeBlock(s.Body)
		for innerStmt, variables := range innerMap.LastUses {
			a.lcMap.LastUses[innerStmt] = variables
		}
		a.lcMap.Errors = append(a.lcMap.Errors, innerMap.Errors...)
		if s.ReturnType != "" {
			for _, st := range s.Body.Statements {
				if ret, ok := st.(*parser.ReturnStatement); ok {
					got := innerAnalyzer.inferType(ret.ReturnValue)
					if !a.typeCompatible(s.ReturnType, got) {
						a.lcMap.Errors = append(a.lcMap.Errors, a.errorAtToken(ret.Token, "return type mismatch in function "+s.Name.Value+": expected "+s.ReturnType+", got "+got))
					}
				}
			}
		}
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
	case *parser.ForStatement:
		if s.Init != nil {
			a.analyzeStatement(s.Init)
		}
		if s.Condition != nil {
			a.analyzeExpression(s.Condition, stmt)
		}
		if s.Post != nil {
			a.analyzeExpression(s.Post, stmt)
		}
		for _, loopStmt := range s.Body.Statements {
			a.analyzeStatement(loopStmt)
		}
	case *parser.ForeachStatement:
		a.analyzeExpression(s.Iterable, stmt)
		a.varTypes[s.Item.Value] = "any"
		for _, loopStmt := range s.Body.Statements {
			a.analyzeStatement(loopStmt)
		}
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
		if _, ok := a.varTypes[e.Value]; !ok {
			a.lcMap.Errors = append(a.lcMap.Errors, a.errorAtToken(e.Token, "undefined variable $"+e.Value))
		}
		if !a.inLoop {
			a.lastUses[e.Value] = parentStmt
		}
	case *parser.CallExpression:
		for _, arg := range e.Arguments {
			a.analyzeExpression(arg, parentStmt)
		}
		a.analyzeExpression(e.Function, parentStmt)
		if ident, ok := e.Function.(*parser.Identifier); ok {
			if sig, ok := a.funcSigs[ident.Value]; ok {
				for i, arg := range e.Arguments {
					if i >= len(sig.ParamTypes) {
						break
					}
					got := a.inferType(arg)
					if !a.typeCompatible(sig.ParamTypes[i], got) {
						a.lcMap.Errors = append(a.lcMap.Errors, a.errorAtToken(e.Token, fmt.Sprintf("argument %d of %s expects %s, got %s", i+1, ident.Value, sig.ParamTypes[i], got)))
					}
				}
			}
		}
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
		if v, ok := e.Left.(*parser.Variable); ok {
			declared := a.varTypes[v.Value]
			got := a.inferType(e.Value)
			if declared != "" && !a.typeCompatible(declared, got) {
				a.lcMap.Errors = append(a.lcMap.Errors, a.errorAtToken(e.Token, "assignment type mismatch for $"+v.Value+": expected "+declared+", got "+got))
			}
		}
	case *parser.MatchExpression:
		a.analyzeExpression(e.Target, parentStmt)
		for _, arm := range e.Arms {
			a.analyzeExpression(arm.Value, parentStmt)
		}
		if e.Default != nil {
			a.analyzeExpression(e.Default, parentStmt)
		}
	}
}

func (a *LifeCycleAnalyzer) inferType(exp parser.Expression) string {
	switch e := exp.(type) {
	case *parser.IntegerLiteral:
		return "int"
	case *parser.StringLiteral:
		return "string"
	case *parser.ArrayLiteral:
		return "array"
	case *parser.Variable:
		if t, ok := a.varTypes[e.Value]; ok {
			return t
		}
		return "any"
	case *parser.InfixExpression:
		if e.Operator == "+" || e.Operator == "-" {
			return "int"
		}
		return "bool"
	case *parser.CallExpression:
		if ident, ok := e.Function.(*parser.Identifier); ok {
			if sig, ok := a.funcSigs[ident.Value]; ok {
				return emptyAsAny(sig.ReturnType)
			}
		}
	}
	return "any"
}

func (a *LifeCycleAnalyzer) typeCompatible(expected, got string) bool {
	if expected == "" || expected == "any" || got == "any" {
		return true
	}
	for _, typ := range strings.Split(expected, "|") {
		if strings.TrimSpace(typ) == got {
			return true
		}
	}
	return false
}

func (a *LifeCycleAnalyzer) errorAtL(line, col int, msg string) string {
	if line <= 0 {
		return msg
	}
	return fmt.Sprintf("line %d, col %d: %s", line, col, msg)
}

func (a *LifeCycleAnalyzer) errorAtToken(tok lexer.Token, msg string) string {
	return a.errorAtL(tok.Line, tok.Column, msg)
}

func emptyAsAny(v string) string {
	if v == "" {
		return "any"
	}
	return v
}
