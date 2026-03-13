package runner

import (
	"fmt"
	"io/ioutil"
	"os"
	"os/exec"
	"strings"

	"github.com/neevets/zenith/src/internal/compiler/lexer"
	"github.com/neevets/zenith/src/internal/compiler/parser"
	"github.com/neevets/zenith/src/internal/compiler/transpiler"
	"github.com/neevets/zenith/src/internal/compiler/analyzer"
	"github.com/neevets/zenith/src/internal/cache"
)

type Options struct {
	AllowRead bool
	AllowNet  bool
	AllowEnv  bool
}

func Run(filename string, opts Options) error {
	input, err := ioutil.ReadFile(filename)
	if err != nil {
		return fmt.Errorf("failed to read file: %w", err)
	}

	l := lexer.New(string(input))
	p := parser.New(l)
	program := p.ParseProgram()

	if len(p.Errors()) != 0 {
		fmt.Printf("Parser errors in %s:\n", filename)
		for _, msg := range p.Errors() {
			fmt.Printf("\t%s\n", msg)
		}
		return fmt.Errorf("parsing failed")
	}

	cm, err := cache.New()
	if err == nil {
		for _, imp := range program.Imports {
			if strings.HasPrefix(imp.Path, "http") {
				localPath, err := cm.Get(imp.Path)
				if err == nil {
					err = transpileToPhp(localPath)
					if err == nil {
						imp.Path = localPath
					}
				}
			}
		}
	}

	a := analyzer.New()
	lcMap := a.Analyze(program)

	if len(lcMap.Errors) > 0 {
		fmt.Println("Quantum Shield Warnings:")
		for _, err := range lcMap.Errors {
			fmt.Printf("  [!] %s\n", err)
		}
		return fmt.Errorf("Quantum Shield blocked execution due to schema mismatches")
	}

	t := transpiler.New()
	t.SetLifeCycleMap(lcMap)
	
	phpCode := t.GetPHPHeader() + t.Transpile(program)

	tmpFile, err := ioutil.TempFile(".", ".zenith-tmp-*.php")
	if err != nil {
		return fmt.Errorf("failed to create temp file: %w", err)
	}
	defer os.Remove(tmpFile.Name())

	if _, err := tmpFile.Write([]byte(phpCode)); err != nil {
		return fmt.Errorf("failed to write to temp file: %w", err)
	}
	tmpFile.Close()

	phpArgs := []string{}
	
	if !opts.AllowRead {
		phpArgs = append(phpArgs, "-d", "open_basedir="+tmpFile.Name())
	}

	if !opts.AllowNet {
		disabledFuncs := "curl_init,curl_exec,file_get_contents,fsockopen,pfsockopen,stream_socket_client,socket_create"
		phpArgs = append(phpArgs, "-d", "disable_functions="+disabledFuncs)
	}

	phpArgs = append(phpArgs, tmpFile.Name())

	phpBin := "php"
	if _, err := exec.LookPath("php"); err != nil {
		home, _ := os.UserHomeDir()
		localPhp := home + "/.zenith/bin/php"
		if _, err := os.Stat(localPhp); err == nil {
			phpBin = localPhp
		}
	}

	cmd := exec.Command(phpBin, phpArgs...)
	output, err := cmd.CombinedOutput()
	if err != nil {
		if strings.Contains(err.Error(), "executable file not found") {
			fmt.Printf("Zenith Preview Mode (PHP not found in system)\n\nGenerated PHP:\n%s\n", phpCode)
			return nil
		}
		fmt.Printf("PHP Execution Error (%v):\n%s\n\nGenerated PHP:\n%s\n", err, string(output), phpCode)
		return err
	}

	fmt.Print(string(output))
	return nil
}

func transpileToPhp(znPath string) error {
	input, err := ioutil.ReadFile(znPath)
	if err != nil {
		return err
	}
	l := lexer.New(string(input))
	p := parser.New(l)
	prog := p.ParseProgram()
	t := transpiler.New()
	phpCode := t.GetPHPHeader() + t.Transpile(prog)
	phpPath := strings.Replace(znPath, ".zen", ".php", 1)
	return ioutil.WriteFile(phpPath, []byte(phpCode), 0644)
}
