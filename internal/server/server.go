package server

import (
	"fmt"
	"io/ioutil"
	"log"
	"net/http"
	"os/exec"
	"strings"

	"github.com/neevets/zenith/compiler/lexer"
	"github.com/neevets/zenith/compiler/parser"
	"github.com/neevets/zenith/compiler/transpiler"
	"github.com/neevets/zenith/compiler/analyzer"
)

func transpileLibraries() {
	files, _ := ioutil.ReadDir(".")
	for _, f := range files {
		if strings.HasSuffix(f.Name(), ".zn") {
			input, err := ioutil.ReadFile(f.Name())
			if err != nil {
				continue
			}
			l := lexer.New(string(input))
			p := parser.New(l)
			program := p.ParseProgram()
			a := analyzer.New()
			lcMap := a.Analyze(program)
			t := transpiler.New()
			t.SetLifeCycleMap(lcMap)
			phpCode := t.GetPHPHeader() + t.Transpile(program)
			phpPath := strings.Replace(f.Name(), ".zn", ".php", 1)
			ioutil.WriteFile(phpPath, []byte(phpCode), 0644)
		}
	}
}

func handleZenith(w http.ResponseWriter, r *http.Request) {
	path := r.URL.Path
	if path == "/" {
		path = "/index.zen"
	}

	if !strings.HasSuffix(path, ".zen") {
		http.ServeFile(w, r, "."+path)
		return
	}

	transpileLibraries()

	fullPath := "." + path
	input, err := ioutil.ReadFile(fullPath)
	if err != nil {
		http.Error(w, fmt.Sprintf("File not found: %s", fullPath), http.StatusNotFound)
		return
	}

	l := lexer.New(string(input))
	p := parser.New(l)
	program := p.ParseProgram()

	if len(p.Errors()) != 0 {
		w.Header().Set("Content-Type", "text/plain")
		fmt.Fprintf(w, "Parser errors in %s:\n", fullPath)
		for _, msg := range p.Errors() {
			fmt.Fprintf(w, "\t%s\n", msg)
		}
		return
	}

	a := analyzer.New()
	lcMap := a.Analyze(program)

	t := transpiler.New()
	t.SetLifeCycleMap(lcMap)
	
	ctxInit := `
$ctx = new Context();
$ctx->query = (object)$_GET;
$ctx->body = (object)$_POST;
$db = null;
`
	phpCode := t.GetPHPHeader() + ctxInit + t.Transpile(program)

	tmpFile, err := ioutil.TempFile(".", ".zenith-tmp-*.php")
	if err != nil {
		http.Error(w, "Failed to create temp file", http.StatusInternalServerError)
		return
	}

	if _, err := tmpFile.Write([]byte(phpCode)); err != nil {
		http.Error(w, "Failed to write to temp file", http.StatusInternalServerError)
		return
	}
	tmpFile.Close()

	cmd := exec.Command("php", tmpFile.Name())
	output, err := cmd.CombinedOutput()
	if err != nil {
		w.Header().Set("Content-Type", "text/plain")
		if strings.Contains(err.Error(), "executable file not found") {
			fmt.Fprintf(w, "Zenith Preview Mode (PHP not found in system)\n\nGenerated PHP:\n%s", phpCode)
		} else {
			fmt.Fprintf(w, "PHP Execution Error (%v):\n%s\n\nGenerated PHP:\n%s", err, string(output), phpCode)
		}
		return
	}

	w.Header().Set("Content-Type", "text/html")
	w.Write(output)
}

func Start(port string) {
	if !strings.HasPrefix(port, ":") {
		port = ":" + port
	}
	fmt.Printf("Zenith Dev Server starting on http://localhost%s\n", port)
	http.HandleFunc("/", handleZenith)
	log.Fatal(http.ListenAndServe(port, nil))
}
