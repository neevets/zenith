<div align="center">

<img src="assets/zen-logo.png" width="200" alt="Zenith Logo" />

<a href="https://zenithlang.vercel.app"><h1>Zenith</h1></a>

</div>

# Table of Contents

1. [Introduction](#introduction)
2. [Core Pillars](#core-pillars)
3. [Installation](#installation)
4. [Documentation](#documentation)
5. [License](#license)
6. [Contributing](#contributing)

---

# Introduction

**Zenith** is a high-performance programming language that transpiles to **PHP 8.1+**. It combines the ubiquity of PHP with the safety and ergonomics of modern functional languages, powered by an optimized engine designed for maximum efficiency.

---

# Core Pillars

- **Optimized Engine**  
  High-performance parser, analyzer, and transpiler core.

- **Modern Transpilation**  
  Targets modern PHP 8.1+ features like `match`, `function`, and `?->`.

- **Quantum Shield**  
  Strict static type system that eliminates runtime type errors.

- **Functional Ergonomics**  
  Native pipe operator `|>` and high-order functions.

- **Secure by Design**  
  Sandboxed environment with explicit permissions (FS, Net, Env).

- **Concurrent Blocks**  
  Native support for parallelism via `spawn` blocks and fibers.

---

# Installation

## Quick Install (Linux & macOS)

```bash
curl -fsS https://dl.zenithlang.xyz/install.sh | sh
```

## Build from Source

```bash
# Clone and build
git clone https://github.com/neevets/zenith
cd zenith
cargo build --release

# Move to path
sudo mv target/release/zenith /usr/local/bin/
```

---

# Documentation

- [Syntax & Types](docs/syntax.md)
- [Functional Programming](docs/functional.md)
- [Modern PHP Features](docs/modern_php.md)
- [Architecture & Security](docs/architecture.md)

---

# License

This project is licensed under the **GNU GPL v3 License**.

---

# Contributing

Contributions are welcome! Please read the [CONTRIBUTING.md](CONTRIBUTING.md) file for more information.
