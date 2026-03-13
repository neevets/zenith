package runner

import (
	"fmt"
	"io/ioutil"
	"os"
	"os/exec"
	"path/filepath"

	"github.com/neevets/zenith/src/internal/compiler/lexer"
	"github.com/neevets/zenith/src/internal/compiler/parser"
	"github.com/neevets/zenith/src/internal/compiler/transpiler"
	"github.com/neevets/zenith/src/internal/compiler/analyzer"
)

func Bundle(filename string, output string) error {
	input, err := ioutil.ReadFile(filename)
	if err != nil {
		return fmt.Errorf("failed to read file: %w", err)
	}

	l := lexer.New(string(input))
	p := parser.New(l)
	program := p.ParseProgram()

	if len(p.Errors()) != 0 {
		return fmt.Errorf("parsing failed")
	}

	a := analyzer.New()
	lcMap := a.Analyze(program)

	t := transpiler.New()
	t.SetLifeCycleMap(lcMap)
	
	phpCode := t.GetPHPHeader() + t.Transpile(program)

	tmpDir, err := ioutil.TempDir("", "zenith-bundle-*")
	if err != nil {
		return err
	}
	defer os.RemoveAll(tmpDir)

	err = ioutil.WriteFile(filepath.Join(tmpDir, "app.php"), []byte(phpCode), 0644)
	if err != nil {
		return err
	}

	goCode := fmt.Sprintf(`
package main

import (
    _ "embed"
    "fmt"
    "io/ioutil"
    "os"
    "os/exec"
    "strings"
)

//go:embed app.php
var phpCode string

func main() {
    tmpFile, err := ioutil.TempFile("", "zenith-app-*.php")
    if err != nil {
        fmt.Fprintf(os.Stderr, "Error: %%v\n", err)
        os.Exit(1)
    }
    defer os.Remove(tmpFile.Name())

    if _, err := tmpFile.Write([]byte(phpCode)); err != nil {
        fmt.Fprintf(os.Stderr, "Error: %%v\n", err)
        os.Exit(1)
    }
    tmpFile.Close()

    phpBin := "php"
    if _, err := exec.LookPath("php"); err != nil {
        home, _ := os.UserHomeDir()
        localPhp := home + "/.zenith/bin/php"
        if _, err := os.Stat(localPhp); err == nil {
            phpBin = localPhp
        }
    }

    cmd := exec.Command(phpBin, tmpFile.Name())
    cmd.Stdout = os.Stdout
    cmd.Stderr = os.Stderr
    err = cmd.Run()
    if err != nil {
        if strings.Contains(err.Error(), "executable file not found") {
            fmt.Println("Error: PHP not found in system and no local Z-Embed binary discovered.")
        }
        os.Exit(1)
    }
}
`)

	err = ioutil.WriteFile(filepath.Join(tmpDir, "main.go"), []byte(goCode), 0644)
	if err != nil {
		return err
	}

	absOutput, _ := filepath.Abs(output)
	cmd := exec.Command("go", "build", "-o", absOutput, "main.go")
	cmd.Dir = tmpDir
	outputLog, err := cmd.CombinedOutput()
	if err != nil {
		return fmt.Errorf("go build failed: %v\n%s", err, string(outputLog))
	}

	fmt.Printf("Zenith bundle created: %s\n", output)
	return nil
}
