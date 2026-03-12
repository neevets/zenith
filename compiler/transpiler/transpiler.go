package transpiler

import (
	"fmt"
	"strings"
	"github.com/neevets/zenith/compiler/lexer"
	"github.com/neevets/zenith/compiler/parser"
	"github.com/neevets/zenith/compiler/analyzer"
)

type Transpiler struct{
	lcMap *analyzer.LifeCycleMap
}

func New() *Transpiler {
	return &Transpiler{}
}

func (t *Transpiler) SetLifeCycleMap(m *analyzer.LifeCycleMap) {
	t.lcMap = m
}

func (t *Transpiler) Transpile(node parser.Node) string {
	switch n := node.(type) {
	case *parser.Program:
		var out strings.Builder
		for _, imp := range n.Imports {
			phpPath := strings.Replace(imp.Path, ".zn", ".php", 1)
			out.WriteString(fmt.Sprintf("require_once \"%s\";\n", phpPath))
		}
		if len(n.Imports) > 0 {
			out.WriteString("\n")
		}
		if n.Middleware != nil {
			out.WriteString(t.Transpile(n.Middleware))
			out.WriteString("\n")
		}
		for _, stmt := range n.Statements {
			out.WriteString(t.Transpile(stmt))
			out.WriteString("\n")
			if t.lcMap != nil {
				if vars, ok := t.lcMap.LastUses[stmt]; ok {
					for _, v := range vars {
						out.WriteString(fmt.Sprintf("unset($%s);\n", v))
					}
				}
			}
		}
		return out.String()

	case *parser.FunctionDefinition:
		var out strings.Builder
		out.WriteString("function ")
		out.WriteString(n.Name.Value)
		out.WriteString("(")
		params := []string{}
		for _, p := range n.Parameters {
			s := ""
			if p.Type != "" {
				s += p.Type + " "
			}
			if p.IsVar {
				s += "$"
			}
			s += p.Name
			params = append(params, s)
		}
		out.WriteString(strings.Join(params, ", "))
		out.WriteString(") {\n")
		out.WriteString(t.Transpile(n.Body))
		out.WriteString("}\n")
		return out.String()

	case *parser.LetStatement:
		return fmt.Sprintf("%s = %s;", t.Transpile(n.Name), t.Transpile(n.Value))

	case *parser.BlockStatement:
		var out strings.Builder
		for _, stmt := range n.Statements {
			out.WriteString("    ")
			out.WriteString(t.Transpile(stmt))
			out.WriteString("\n")
			if t.lcMap != nil {
				if vars, ok := t.lcMap.LastUses[stmt]; ok {
					for _, v := range vars {
						out.WriteString(fmt.Sprintf("    unset($%s);\n", v))
					}
				}
			}
		}
		return out.String()

	case *parser.ReturnStatement:
		return fmt.Sprintf("return %s;", t.Transpile(n.ReturnValue))

	case *parser.ExpressionStatement:
		return fmt.Sprintf("%s;", t.Transpile(n.Expression))

	case *parser.CallExpression:
		funcName := t.Transpile(n.Function)
		args := []string{}
		for _, arg := range n.Arguments {
			args = append(args, t.Transpile(arg))
		}
		
		if funcName == "print" {
			return fmt.Sprintf("echo %s", strings.Join(args, ", "))
		}
		
		return fmt.Sprintf("%s(%s)", funcName, strings.Join(args, ", "))

	case *parser.Identifier:
		return n.Value

	case *parser.Variable:
		return "$" + n.Value

	case *parser.IntegerLiteral:
		return fmt.Sprintf("%d", n.Value)

	case *parser.StringLiteral:
		val := n.Value
		if n.IsRender && n.Delimiter == '"' {
			val = t.applyXSSProtection(val)
		}
		quote := string(n.Delimiter)
		if quote == "" {
			quote = "\""
		}
		return quote + val + quote

	case *parser.MethodCallExpression:
		obj := t.Transpile(n.Object)
		method := n.Method.Value
		args := []string{}
		for _, arg := range n.Arguments {
			args = append(args, t.Transpile(arg))
		}
		
		switch method {
		case "length":
			return fmt.Sprintf("strlen(%s)", obj)
		case "push":
			return fmt.Sprintf("array_push(%s, %s)", obj, strings.Join(args, ", "))
		case "parse":
			if obj == "json" {
				return fmt.Sprintf("json_decode(%s, true)", strings.Join(args, ", "))
			}
		case "stringify":
			if obj == "json" {
				return fmt.Sprintf("json_encode(%s)", strings.Join(args, ", "))
			}
		}

		if obj == "$ctx" {
			switch method {
			case "query":
				if len(args) > 0 {
					return fmt.Sprintf("$_GET[%s]", args[0])
				}
				return "$_GET"
			case "body":
				if len(args) > 0 {
					return fmt.Sprintf("$_POST[%s]", args[0])
				}
				return "$_POST"
			}
		}
		
		return fmt.Sprintf("%s->%s(%s)", obj, method, strings.Join(args, ", "))

	case *parser.MemberExpression:
		obj := t.Transpile(n.Object)
		prop := n.Property.Value
		
		if obj == "$ctx" {
			switch prop {
			case "query":
				return "$_GET"
			case "body":
				return "$_POST"
			}
		}
		
		if obj == "$_GET" {
			return fmt.Sprintf("$_GET['%s']", prop)
		}
		if obj == "$_POST" {
			return fmt.Sprintf("$_POST['%s']", prop)
		}

		return fmt.Sprintf("%s->%s", obj, prop)

	case *parser.NullCoalesceExpression:
		left := t.Transpile(n.Left)
		right := t.Transpile(n.Right)
		
		if call, ok := n.Right.(*parser.CallExpression); ok {
			if ident, ok := call.Function.(*parser.Identifier); ok && ident.Value == "error" {
				msg := "Error"
				if len(call.Arguments) > 0 {
					msg = t.Transpile(call.Arguments[0])
				}
				return fmt.Sprintf("(%s ?? die(%s))", left, msg)
			}
		}
		
		return fmt.Sprintf("(%s ?? %s)", left, right)

	case *parser.SqlQueryExpression:
		args := []string{}
		for _, arg := range n.Args {
			args = append(args, t.Transpile(arg))
		}
		
		useClause := "$db"
		if len(args) > 0 {
			useClause = strings.Join(args, ", ") + ", $db"
		}
		
		return fmt.Sprintf("(function() use (%s) { try { $stmt = $db->prepare(\"%s\"); $stmt->execute([%s]); return $stmt->fetchAll(); } catch (Exception $e) { return null; } })()",
			useClause,
			n.Query,
			strings.Join(args, ", "),
		)

	case *parser.ArrayLiteral:
		elements := []string{}
		for _, e := range n.Elements {
			elements = append(elements, t.Transpile(e))
		}
		return "[" + strings.Join(elements, ", ") + "]"

	case *parser.IndexExpression:
		return fmt.Sprintf("%s[%s]", t.Transpile(n.Left), t.Transpile(n.Index))

	case *parser.InfixExpression:
		return fmt.Sprintf("(%s %s %s)", t.Transpile(n.Left), n.Operator, t.Transpile(n.Right))

	case *parser.IfStatement:
		var out strings.Builder
		out.WriteString(fmt.Sprintf("if (%s) {\n", t.Transpile(n.Condition)))
		out.WriteString(t.Transpile(n.Consequence))
		out.WriteString("}")
		if n.Alternative != nil {
			out.WriteString(" else {\n")
			out.WriteString(t.Transpile(n.Alternative))
			out.WriteString("}")
		}
		return out.String()

	case *parser.WhileStatement:
		var out strings.Builder
		out.WriteString(fmt.Sprintf("while (%s) {\n", t.Transpile(n.Condition)))
		out.WriteString(t.Transpile(n.Body))
		out.WriteString("}")
		return out.String()

	case *parser.AssignExpression:
		return fmt.Sprintf("%s = %s", t.Transpile(n.Left), t.Transpile(n.Value))
	}

	return ""
}

func (t *Transpiler) GetPHPHeader() string {
	return `<?php

if (!class_exists('Context')) { class Context { public $query; public $body; } }

function fetch($url) {
    $opts = [
        "http" => ["header" => "User-Agent: ZenithRuntime/1.0\r\n"]
    ];
    $context = stream_context_create($opts);
    return file_get_contents($url, false, $context);
}

`
}

func (t *Transpiler) applyXSSProtection(input string) string {
	result := input
	for {
		start := strings.Index(result, "{")
		if start == -1 {
			break
		}
		end := strings.Index(result[start:], "}")
		if end == -1 {
			break
		}
		end += start

		content := result[start+1 : end]
		
		l := lexer.New(content)
		p := parser.New(l)
		expr := p.ParseExpression(0)

		var replacement string
		if expr != nil {
			transpiledExpr := t.Transpile(expr)
			if strings.HasPrefix(content, "$") {
				replacement = "\" . htmlspecialchars(" + transpiledExpr + ") . \""
			} else {
				replacement = "\" . (" + transpiledExpr + ") . \""
			}
		} else {
			replacement = content
		}

		result = result[:start] + replacement + result[end+1:]
	}
	return result
}
