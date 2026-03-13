package system

import (
	"archive/tar"
	"compress/gzip"
	"fmt"
	"io"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
)

const (
	PHP_VERSION = "8.2.0"
	BASE_URL    = "https://github.com/shivammathur/php-bin/releases/download/"
)

func EnsurePHP() (string, error) {
	if path, err := exec.LookPath("php"); err == nil {
		return path, nil
	}

	home, err := os.UserHomeDir()
	if err != nil {
		return "", err
	}
	localBin := filepath.Join(home, ".zenith", "bin")
	localPhp := filepath.Join(localBin, "php")
	if runtime.GOOS == "windows" {
		localPhp += ".exe"
	}

	if _, err := os.Stat(localPhp); err == nil {
		return localPhp, nil
	}

	fmt.Printf("PHP not found in system. Downloading PHP %s for %s/%s...\n", PHP_VERSION, runtime.GOOS, runtime.GOARCH)
	if err := os.MkdirAll(localBin, 0755); err != nil {
		return "", err
	}

	url, err := getDownloadURL()
	if err != nil {
		return "", err
	}

	tmpFile := filepath.Join(os.TempDir(), "php-bundle.tar.gz")
	if err := downloadFile(url, tmpFile); err != nil {
		return "", err
	}
	defer os.Remove(tmpFile)

	fmt.Println("Extracting PHP runtime...")
	if err := extractTarGz(tmpFile, localBin); err != nil {
		return "", fmt.Errorf("failed to extract: %w", err)
	}

	if _, err := os.Stat(localPhp); err != nil {
		return "", fmt.Errorf("extraction succeeded but binary not found at %s", localPhp)
	}

	err = os.Chmod(localPhp, 0755)
	if err != nil {
		return "", err
	}

	return localPhp, nil
}

func getDownloadURL() (string, error) {
	var osName string
	switch runtime.GOOS {
	case "linux":
		osName = "linux"
	case "darwin":
		osName = "macos"
	case "windows":
		osName = "win"
	default:
		return "", fmt.Errorf("unsupported OS: %s", runtime.GOOS)
	}

	return fmt.Sprintf("%s%s/php-%s-%s-%s.tar.gz", BASE_URL, PHP_VERSION, PHP_VERSION, osName, runtime.GOARCH), nil
}

func downloadFile(url string, filepath string) error {
	resp, err := http.Get(url)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("bad status: %s", resp.Status)
	}

	out, err := os.Create(filepath)
	if err != nil {
		return err
	}
	defer out.Close()

	_, err = io.Copy(out, resp.Body)
	return err
}

func extractTarGz(gzipPath, destDir string) error {
	file, err := os.Open(gzipPath)
	if err != nil {
		return err
	}
	defer file.Close()

	gzr, err := gzip.NewReader(file)
	if err != nil {
		return err
	}
	defer gzr.Close()

	tr := tar.NewReader(gzr)

	for {
		header, err := tr.Next()
		if err == io.EOF {
			break
		}
		if err != nil {
			return err
		}

		target := filepath.Join(destDir, header.Name)

		switch header.Typeflag {
		case tar.TypeDir:
			if err := os.MkdirAll(target, 0755); err != nil {
				return err
			}
		case tar.TypeReg:
			f, err := os.OpenFile(target, os.O_CREATE|os.O_RDWR, os.FileMode(header.Mode))
			if err != nil {
				return err
			}
			if _, err := io.Copy(f, tr); err != nil {
				f.Close()
				return err
			}
			f.Close()
		}
	}
	return nil
}
