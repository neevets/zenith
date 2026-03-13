package analyzer

import (
	"fmt"
	"github.com/neevets/zenith/src/internal/compiler/parser"
)

type TypeChecker struct {
	symbols map[string]string
	errors  []string
}

func NewTypeChecker() *TypeChecker {
	return &TypeChecker{
		symbols: make(map[string]string),
		errors:  []string{},
	}
}

func (tc *TypeChecker) Check(program *parser.Program) []string {
	for _, stmt := range program.Statements {
		tc.checkStatement(stmt)
	}
	return tc.errors
}

func (tc *TypeChecker) checkStatement(stmt parser.Statement) {
	switch s := stmt.(type) {
	case *parser.LetStatement:
		valType := tc.inferType(s.Value)
		if s.Type != "" && s.Type != "any" {
			if !tc.isCompatible(s.Type, valType) {
				tc.errors = append(tc.errors, fmt.Sprintf("Type mismatch: cannot assign %s to variable %s of type %s", valType, s.Name.Value, s.Type))
			}
		}
		tc.symbols[s.Name.Value] = valType
	case *parser.FunctionDefinition:
		// Register function parameters in a local scope
		// Simplified for now: just check body
		for _, stmt := range s.Body.Statements {
			tc.checkStatement(stmt)
		}
	case *parser.IfStatement:
		tc.checkStatement(s.Consequence)
		if s.Alternative != nil {
			tc.checkStatement(s.Alternative)
		}
	case *parser.WhileStatement:
		for _, stmt := range s.Body.Statements {
			tc.checkStatement(stmt)
		}
	}
}

func (tc *TypeChecker) inferType(exp parser.Expression) string {
	switch e := exp.(type) {
	case *parser.PipeExpression:
		return tc.inferType(e.Right)
	case *parser.IntegerLiteral:

		return "int"
	case *parser.StringLiteral:
		return "string"
	case *parser.Variable:
		if t, ok := tc.symbols[e.Value]; ok {
			return t
		}
		return "any"
	case *parser.InfixExpression:
		// Primitive inference for math
		left := tc.inferType(e.Left)
		right := tc.inferType(e.Right)
		if left == "int" && right == "int" {
			return "int"
		}
		return "any"
	case *parser.ArrayLiteral:
		return "array"
	case *parser.CallExpression:
		// We'd need a function symbol table for better inference
		return "any"
	default:
		return "any"
	}
}

func (tc *TypeChecker) isCompatible(target, actual string) bool {
	if target == actual || target == "any" || actual == "any" {
		return true
	}
	return false
}
