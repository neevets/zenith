package runner

import (
	"fmt"
	"io/ioutil"
	"strings"

	"github.com/neevets/zenith/src/internal/engine"
)

type Options struct {
	AllowRead bool
	AllowNet  bool
	AllowEnv  bool
}

func Run(filename string, opts Options) error {
	e := engine.New(engine.Options{
		AllowRead: opts.AllowRead,
		AllowNet:  opts.AllowNet,
		AllowEnv:  opts.AllowEnv,
	})

	phpCode, err := e.Transpile(filename)
	if err != nil {
		return err
	}

	output, err := e.Execute(phpCode)
	if err != nil {
		return err
	}

	fmt.Print(output)
	return nil
}

func transpileToPhp(znPath string) error {
	e := engine.New(engine.Options{})
	phpCode, err := e.Transpile(znPath)
	if err != nil {
		return err
	}
	phpPath := strings.Replace(znPath, ".zen", ".php", 1)
	return ioutil.WriteFile(phpPath, []byte(phpCode), 0644)
}
