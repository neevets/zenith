package transpiler

import (
	"fmt"
	"strings"
	"github.com/neevets/zenith/src/internal/compiler/lexer"
	"github.com/neevets/zenith/src/internal/compiler/parser"
	"github.com/neevets/zenith/src/internal/compiler/analyzer"
)

type Transpiler struct {
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
			path := imp.AbsPath
			if path == "" {
				path = strings.Replace(imp.Path, ".zen", ".php", 1)
			}
			out.WriteString(fmt.Sprintf("require_once \"%s\";\n", path))
		}
		if len(n.Imports) > 0 {
			out.WriteString("\n")
		}
		if n.Middleware != nil {
			out.WriteString(t.Transpile(n.Middleware) + "\n")
		}
		for _, stmt := range n.Statements {
			out.WriteString(t.Transpile(stmt) + "\n")
			if t.lcMap != nil {
				if vars, ok := t.lcMap.LastUses[stmt]; ok {
					for _, v := range vars {
						if v != "ctx" && !strings.HasPrefix(v, "db") && v != "file" {
							out.WriteString(fmt.Sprintf("unset($%s);\n", v))
						}
					}
				}
			}
		}
		return out.String()

	case *parser.FunctionDefinition:
		var out strings.Builder
		out.WriteString("function " + n.Name.Value + "(")
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
		out.WriteString(strings.Join(params, ", ") + ")")
		if n.ReturnType != "" {
			out.WriteString(": " + n.ReturnType)
		}
		out.WriteString(" {\n    global $file, $db, $ctx, $db_path;\n" + t.Transpile(n.Body) + "}\n")
		return out.String()

	case *parser.LetStatement:
		return fmt.Sprintf("%s = %s;", t.Transpile(n.Name), t.Transpile(n.Value))

	case *parser.BlockStatement:
		var out strings.Builder
		for _, stmt := range n.Statements {
			out.WriteString("    " + t.Transpile(stmt) + "\n")
			if t.lcMap != nil {
				if vars, ok := t.lcMap.LastUses[stmt]; ok {
					for _, v := range vars {
						if v != "ctx" {
							out.WriteString(fmt.Sprintf("    unset($%s);\n", v))
						}
					}
				}
			}
		}
		return out.String()

	case *parser.ReturnStatement:
		return fmt.Sprintf("return %s;", t.Transpile(n.ReturnValue))

	case *parser.ExpressionStatement:
		return t.Transpile(n.Expression) + ";"

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
		if obj == "file" {
			obj = "$file"
		}
		op := "->"
		if n.IsNullsafe {
			op = "?->"
		}
		return fmt.Sprintf("%s%s%s(%s)", obj, op, method, strings.Join(args, ", "))

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
		op := "->"
		if n.IsNullsafe {
			op = "?->"
		}
		return fmt.Sprintf("%s%s%s", obj, op, prop)

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
		use := "$db"
		if len(args) > 0 {
			use = strings.Join(args, ", ") + ", $db"
		}
		return fmt.Sprintf("(function() use (%s) { try { $stmt = $db->prepare(\"%s\"); $stmt->execute([%s]); return $stmt->fetchAll(); } catch (Exception $e) { return null; } })()",
			use, n.Query, strings.Join(args, ", "))

	case *parser.ArrayLiteral:
		elements := []string{}
		for _, e := range n.Elements {
			elements = append(elements, t.Transpile(e))
		}
		return "[" + strings.Join(elements, ", ") + "]"

	case *parser.IndexExpression:
		return fmt.Sprintf("%s[%s]", t.Transpile(n.Left), t.Transpile(n.Index))

	case *parser.InfixExpression:
		op := n.Operator
		if op == "+" {
			op = "."
		}
		return fmt.Sprintf("(%s %s %s)", t.Transpile(n.Left), op, t.Transpile(n.Right))

	case *parser.IfStatement:
		var out strings.Builder
		out.WriteString(fmt.Sprintf("if (%s) {\n%s}", t.Transpile(n.Condition), t.Transpile(n.Consequence)))
		if n.Alternative != nil {
			out.WriteString(fmt.Sprintf(" else {\n%s}", t.Transpile(n.Alternative)))
		}
		return out.String()

	case *parser.WhileStatement:
		return fmt.Sprintf("while (%s) {\n%s}", t.Transpile(n.Condition), t.Transpile(n.Body))

	case *parser.ForStatement:
		return fmt.Sprintf("foreach (%s as $%s) {\n%s}", t.Transpile(n.Iterable), n.Variable, t.Transpile(n.Body))

	case *parser.AssignExpression:
		return fmt.Sprintf("%s = %s", t.Transpile(n.Left), t.Transpile(n.Value))

	case *parser.PipeExpression:
		lhs := t.Transpile(n.Left)
		switch r := n.Right.(type) {
		case *parser.CallExpression:
			args := []string{lhs}
			for _, arg := range r.Arguments {
				args = append(args, t.Transpile(arg))
			}
			return fmt.Sprintf("%s(%s)", t.Transpile(r.Function), strings.Join(args, ", "))
		case *parser.Identifier:
			return fmt.Sprintf("%s(%s)", t.Transpile(r), lhs)
		default:
			return lhs
		}

	case *parser.MatchExpression:
		var out strings.Builder
		out.WriteString(fmt.Sprintf("match (%s) {\n", t.Transpile(n.Condition)))
		for _, arm := range n.Arms {
			out.WriteString("        ")
			if arm.IsDefault {
				out.WriteString("default")
			} else {
				vals := []string{}
				for _, val := range arm.Values {
					vals = append(vals, t.Transpile(val))
				}
				out.WriteString(strings.Join(vals, ", "))
			}
			out.WriteString(" => " + t.Transpile(arm.Result) + ",\n")
		}
		out.WriteString("    }")
		return out.String()

	case *parser.ArrowFunctionExpression:
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
		ret := ""
		if n.ReturnType != "" {
			ret = ": " + n.ReturnType
		}
		return fmt.Sprintf("fn(%s)%s => %s", strings.Join(params, ", "), ret, t.Transpile(n.Body))

	case *parser.SpawnExpression:
		var body string
		if block, ok := n.Body.(*parser.BlockStatement); ok {
			stmts := []string{}
			for _, s := range block.Statements {
				stmts = append(stmts, t.Transpile(s))
			}
			body = fmt.Sprintf("function() {\n            %s\n        }", strings.Join(stmts, "\n            "))
		} else {
			body = t.Transpile(n.Body)
		}
		return fmt.Sprintf("(function() { $f = new Fiber(%s); $f->start(); return $f; })()", body)

	case *parser.YieldStatement:
		val := ""
		if n.Value != nil {
			val = t.Transpile(n.Value)
		}
		return fmt.Sprintf("Fiber::suspend(%s);", val)

	case *parser.EnumStatement:
		var out strings.Builder
		out.WriteString(fmt.Sprintf("enum %s {\n", n.Name.Value))
		for _, c := range n.Cases {
			out.WriteString(fmt.Sprintf("    case %s", c.Name.Value))
			if c.Value != nil {
				out.WriteString(fmt.Sprintf(" = %s", t.Transpile(c.Value)))
			}
			out.WriteString(";\n")
		}
		out.WriteString("}\n")
		return out.String()


	case *parser.StructDefinition:
		var out strings.Builder
		out.WriteString(fmt.Sprintf("class %s {\n", n.Name.Value))
		for _, f := range n.Fields {
			mod := "public"
			if f.IsReadonly {
				mod += " readonly"
			}
			typ := ""
			if f.Type != "" {
				typ = f.Type + " "
			}
			out.WriteString(fmt.Sprintf("    %s %s$%s;\n", mod, typ, f.Name))
		}
		out.WriteString("}\n")
		return out.String()

	case *parser.MapLiteral:
		pairs := []string{}
		for _, pair := range n.Pairs {
			pairs = append(pairs, fmt.Sprintf("%s => %s", t.Transpile(pair.Key), t.Transpile(pair.Value)))
		}
		return "[" + strings.Join(pairs, ", ") + "]"
	case *parser.PrefixExpression:
		return t.transpilePrefixExpression(n)
	}
	return ""
}

func (t *Transpiler) transpilePrefixExpression(pe *parser.PrefixExpression) string {
	return "(" + pe.Operator + t.Transpile(pe.Right) + ")"
}

func (t *Transpiler) GetPHPHeader() string {
	return `<?php

if (!class_exists('Context')) { class Context { public $path; public $query; public $body; } }

if (!function_exists('fetch')) {
function fetch($url) {
    $opts = ["http" => ["header" => "User-Agent: ZenithRuntime/1.0\r\n"]];
    return file_get_contents($url, false, stream_context_create($opts));
}
}

if (!function_exists('json')) {
function json($data) {
    return is_string($data) ? json_decode($data, true) : json_encode($data);
}
}

if (!function_exists('env')) {
function env($key) {
    return getenv($key);
}
}

if (!function_exists('println')) {
function println($data) {
    echo $data . "\n";
}
}

if (!function_exists('redirect')) {
function redirect($url) {
    header("Location: " . $url);
    exit;
}
}

if (!function_exists('z_assert')) {
function z_assert($condition, $message = "Assertion failed") {
    if ($condition) {
        echo "  [OK] Pass: " . $message . "\n";
    } else {
        echo "  [FAIL] FAIL: " . $message . "\n";
        exit(1);
    }
}
}

if (!class_exists('ZenithFile')) {
class ZenithFile {
    public function read($path) { return file_get_contents($path); }
    public function write($path, $data) { return file_put_contents($path, $data); }
}
}
$file = new ZenithFile();
$ctx = new Context();
$ctx->path = parse_url($_SERVER['REQUEST_URI'] ?? '/', PHP_URL_PATH);
$ctx->query = $_GET ?? [];
$ctx->body = $_POST ?? [];

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
		expr := parser.New(lexer.New(content)).ParseExpression(0)
		var repl string
		if expr != nil {
			tr := t.Transpile(expr)
			if strings.HasPrefix(content, "$") {
				repl = "\" . htmlspecialchars(" + tr + ") . \""
			} else {
				repl = "\" . (" + tr + ") . \""
			}
		} else {
			repl = content
		}
		result = result[:start] + repl + result[end+1:]
	}
	return result
}
