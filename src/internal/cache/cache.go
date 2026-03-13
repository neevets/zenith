package cache

import (
	"crypto/sha256"
	"fmt"
	"io/ioutil"
	"net/http"
	"os"
	"path/filepath"
	"strings"
)

type Cache struct {
	BaseDir string
}

func New() (*Cache, error) {
	home, err := os.UserHomeDir()
	if err != nil {
		return nil, err
	}
	base := filepath.Join(home, ".zenith", "cache")
	if err := os.MkdirAll(base, 0755); err != nil {
		return nil, err
	}
	return &Cache{BaseDir: base}, nil
}

func (c *Cache) Get(url string) (string, error) {
	if !strings.HasPrefix(url, "http") {
		return url, nil
	}

	hash := fmt.Sprintf("%x", sha256.Sum256([]byte(url)))
	localPath := filepath.Join(c.BaseDir, hash+".zen")

	if _, err := os.Stat(localPath); err == nil {
		return localPath, nil
	}

	fmt.Printf("Downloading %s...\n", url)
	resp, err := http.Get(url)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return "", fmt.Errorf("failed to download: status %d", resp.StatusCode)
	}

	data, err := ioutil.ReadAll(resp.Body)
	if err != nil {
		return "", err
	}

	if err := ioutil.WriteFile(localPath, data, 0644); err != nil {
		return "", err
	}

	return localPath, nil
}

func (c *Cache) GetTranspiled(sourceHash string) (string, bool) {
	localPath := filepath.Join(c.BaseDir, "transpiled", sourceHash+".php")
	if _, err := os.Stat(localPath); err == nil {
		data, err := ioutil.ReadFile(localPath)
		if err == nil {
			return string(data), true
		}
	}
	return "", false
}

func (c *Cache) SaveTranspiled(sourceHash string, phpCode string) error {
	dir := filepath.Join(c.BaseDir, "transpiled")
	if err := os.MkdirAll(dir, 0755); err != nil {
		return err
	}
	localPath := filepath.Join(dir, sourceHash+".php")
	return ioutil.WriteFile(localPath, []byte(phpCode), 0644)
}
