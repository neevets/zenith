package engine

import (
	"crypto/sha256"
	"fmt"
	"io/ioutil"
	"os"
	"os/exec"
	"strings"

	"path/filepath"

	"github.com/neevets/zenith/src/internal/compiler/lexer"
	"github.com/neevets/zenith/src/internal/compiler/parser"
	"github.com/neevets/zenith/src/internal/compiler/transpiler"
	"github.com/neevets/zenith/src/internal/compiler/analyzer"
	"github.com/neevets/zenith/src/internal/system"
	"github.com/neevets/zenith/src/internal/cache"
)

type Options struct {
	AllowRead bool
	AllowNet  bool
	AllowEnv  bool
}

type Engine struct {
	opts Options
}

func New(opts Options) *Engine {
	return &Engine{opts: opts}
}

func (e *Engine) Transpile(filename string) (string, error) {
	absPath, _ := filepath.Abs(filename)
	input, err := ioutil.ReadFile(absPath)
	if err != nil {
		return "", fmt.Errorf("failed to read file: %w", err)
	}

	cm, _ := cache.New()
	sourceHash := ""
	if cm != nil {
		sourceHash = fmt.Sprintf("%x", sha256.Sum256(input))
		if cachedPhp, ok := cm.GetTranspiled(sourceHash); ok {
			return cachedPhp, nil
		}
	}

	l := lexer.New(string(input))
	p := parser.New(l)
	program := p.ParseProgram()
	fmt.Printf("ENGINE: ParseProgram finished for %s, errors=%d\n", filename, len(p.Errors()))

	if len(p.Errors()) != 0 {
		var errMsgs []string
		for _, msg := range p.Errors() {
			errMsgs = append(errMsgs, msg)
		}
		return "", fmt.Errorf("parser errors:\n%s", strings.Join(errMsgs, "\n"))
	}

	a := analyzer.New()
	lcMap := a.Analyze(program)

	if len(lcMap.Errors) > 0 {
		var errMsgs []string
		for _, err := range lcMap.Errors {
			errMsgs = append(errMsgs, fmt.Sprintf("[!] %s", err))
		}
		return "", fmt.Errorf("Quantum Shield blocked execution:\n%s", strings.Join(errMsgs, "\n"))
	}

	t := transpiler.New()
	t.SetLifeCycleMap(lcMap)

	dir := filepath.Dir(absPath)
	for _, imp := range program.Imports {
		ext := strings.HasSuffix(imp.Path, ".zen")
		if ext {
			impAbs := imp.Path
			if !filepath.IsAbs(impAbs) {
				impAbs = filepath.Join(dir, imp.Path)
			}
			php, err := e.Transpile(impAbs)
			if err != nil {
				return "", fmt.Errorf("failed to transpile import %s: %w", imp.Path, err)
			}
			phpPath := strings.Replace(impAbs, ".zen", ".php", 1)
			imp.AbsPath = phpPath
			if err := ioutil.WriteFile(phpPath, []byte(php), 0644); err != nil {
				return "", fmt.Errorf("failed to write transpiled file %s: %w", phpPath, err)
			}
		}
	}
	
	phpCode := t.GetPHPHeader() + t.Transpile(program)

	if cm != nil && sourceHash != "" {
		cm.SaveTranspiled(sourceHash, phpCode)
	}

	return phpCode, nil
}

func (e *Engine) Execute(phpCode string) (string, error) {
	tmpFile, err := ioutil.TempFile(".", ".zenith-tmp-*.php")
	if err != nil {
		return "", fmt.Errorf("failed to create temp file: %w", err)
	}
	defer os.Remove(tmpFile.Name())

	if _, err := tmpFile.Write([]byte(phpCode)); err != nil {
		return "", fmt.Errorf("failed to write to temp file: %w", err)
	}
	tmpFile.Close()

	phpArgs := []string{}
	
	if !e.opts.AllowRead {
		phpArgs = append(phpArgs, "-d", "open_basedir=.:"+tmpFile.Name())
	}

	if !e.opts.AllowNet {
		phpArgs = append(phpArgs, "-d", "allow_url_fopen=Off")
		disabledFuncs := "curl_init,curl_exec,fsockopen,pfsockopen,stream_socket_client,socket_create"
		phpArgs = append(phpArgs, "-d", "disable_functions="+disabledFuncs)
	}

	phpArgs = append(phpArgs, tmpFile.Name())

	phpBin, err := system.EnsurePHP()
	if err != nil {
		return fmt.Sprintf("Zenith Preview Mode (PHP not found and auto-download failed: %v)\n\nGenerated PHP:\n%s\n", err, phpCode), nil
	}

	cmd := exec.Command(phpBin, phpArgs...)
	output, err := cmd.CombinedOutput()
	if err != nil {
		return "", fmt.Errorf("PHP Execution Error (%v):\n%s\n\nGenerated PHP:\n%s", err, string(output), phpCode)
	}

	return string(output), nil
}
