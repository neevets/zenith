<div align="center">
  <img src="assets/logo.png" width="200" alt="Zenith Logo">
  <p><a href="https://zenithlang.vercel.app">zenithlang.vercel.app</a></p>
  <h1>Zenith</h1>
  <p><strong>Strict Types | Functional Patterns | High-Performance Core</strong></p>
</div>

---

## Zenith V2.6 (High-Performance Edition)
Zenith is a high-performance programming language that transpiles to PHP 8.1+. It combines the ubiquity of PHP with the safety and ergonomics of modern functional languages, now powered by an optimized engine for maximum efficiency.

## Core Pillars
- **Optimized Engine**: High-performance parser, analyzer, and transpiler core.
- **Modern Transpilation**: Targets PHP 8.1+ features like `match`, `fn`, and `?->`.
- **Quantum Shield**: Strict static type system that eliminates runtime type errors.
- **Functional Ergonomics**: Native pipe operator `|>` and high-order functions.
- **Secure by Design**: Sandboxed environment with explicit permissions (FS, Net, Env).
- **Concurrent Blocks**: Native support for parallelism via `spawn` blocks and fibers.

## Installation

### Quick Install (Linux & macOS)
```bash
curl -fsS https://dl.zenith.vercel.app/install.sh | sh
```

### Build from Source
```bash
# Clone and build
git clone https://github.com/neevets/zenith
cd zenith
cargo build --release

# Move to path
sudo mv target/release/zenith /usr/local/bin/
```

## Advanced Syntax at a Glance

### Concurrency & Performance
```zenith
spawn {
    println("Running in background...");
};

let $data = load_large_dataset() 
    |> filter(fn($x) => $x > 10)
    |> map(fn($x) => $x * 2);
```

### Modern Features
```zenith
let $status = match($code) {
    200 => "OK",
    404 => {
        log_error("Not found");
        "Error: 404"
    },
    default => "Unknown"
};
```

## Documentation
- [Syntax & Types](docs/syntax.md)
- [Functional Programming](docs/functional.md)
- [Modern PHP Features](docs/modern_php.md)
- [Architecture & Security](docs/architecture.md)


## IDE Support (.zen)
Zenith files use the `.zen` extension.

- **VS Code**: open the workspace and install the recommended extension (`neevets.zenith-vscode`).
- **Cursor**: reads the same `.vscode` workspace config, so `.zen` and `.zenith` are associated automatically.
- **Antigravity**: if it is VS Code-compatible, it will also pick up `.vscode/settings.json` and `.vscode/extensions.json` from this repository.

If your IDE does not auto-load workspace settings, add this association manually:

```json
{
  "files.associations": {
    "*.zen": "zenith",
    "*.zenith": "zenith"
  }
}
```
