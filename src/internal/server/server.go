package server

import (
	"fmt"
	"io/ioutil"
	"log"
	"net/http"
	"strings"

	"github.com/neevets/zenith/src/internal/engine"
)

func transpileLibraries() {
	files, _ := ioutil.ReadDir(".")
	for _, f := range files {
		if strings.HasSuffix(f.Name(), ".zen") {
			e := engine.New(engine.Options{})
			phpCode, err := e.Transpile(f.Name())
			if err != nil {
				continue
			}
			phpPath := strings.Replace(f.Name(), ".zen", ".php", 1)
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
	e := engine.New(engine.Options{
		AllowRead: true,
		AllowNet:  true,
		AllowEnv:  true,
	})

	phpCode, err := e.Transpile(fullPath)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	ctxInit := fmt.Sprintf(`
$ctx = new Context();
$ctx->path = "%s";
$ctx->query = (object)$_GET;
$ctx->body = (object)$_POST;
$db = null;
`, path)
	phpCode = strings.Replace(phpCode, "$file = new ZenithFile();", "$file = new ZenithFile();\n"+ctxInit, 1)

	output, err := e.Execute(phpCode)
	if err != nil {
		w.Header().Set("Content-Type", "text/plain")
		fmt.Fprintf(w, "%v", err)
		return
	}

	w.Header().Set("Content-Type", "text/html")
	w.Write([]byte(output))
}

func Start(port string) {
	if !strings.HasPrefix(port, ":") {
		port = ":" + port
	}
	fmt.Printf("Zenith Server starting on http://localhost%s\n", port)
	http.HandleFunc("/", handleZenith)
	log.Fatal(http.ListenAndServe(port, nil))
}
