package runner

import (
	"fmt"
	"io/ioutil"
	"os"
	"os/exec"
	"path/filepath"

	"github.com/neevets/zenith/src/internal/engine"
)

func Bundle(filename string, output string) error {
	e := engine.New(engine.Options{})
	phpCode, err := e.Transpile(filename)
	if err != nil {
		return err
	}

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
    "path/filepath"
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
        localPaths := []string{
            filepath.Join(home, ".zenith", "bin", "php"),
            "/usr/local/bin/php",
            "/usr/bin/php",
        }
        found := false
        for _, path := range localPaths {
            if _, err := os.Stat(path); err == nil {
                phpBin = path
                found = true
                break
            }
        }
        if !found {
            fmt.Printf("Zenith Preview Mode (PHP not found in system)\n\nGenerated PHP:\n%%s\n", phpCode)
            return
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
