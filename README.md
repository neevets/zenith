<div align="center">

<img src="assets/zen-logo.png" width="200" alt="Zenith Logo" />

<h1>
  <a href="https://zenithlang.xyz">Zenith</a>
</h1>

<p>
  <img src="https://github.com/neevets/zenith/actions/workflows/tests.yml/badge.svg" alt="Tests">
  <img src="https://img.shields.io/github/license/neevets/zenith" alt="License">
  <br>
  <img src="https://img.shields.io/github/stars/neevets/zenith" alt="Stars">
  <img src="https://img.shields.io/github/forks/neevets/zenith" alt="Forks">
</p>

> **Zenith is an experimental research project.** It is NOT production-ready. The security features and type analysis are in prototype/development stage, and should not be relied upon for critical applications.

</div>

# Table of Contents

1. [Introduction](#introduction)
2. [Why Zenith?](#why-zenith)
3. [Architecture](#architecture)
4. [Managed Execution Context](#managed-execution-context)
5. [Installation](#installation)
6. [Documentation](#documentation)
7. [License](#license)

---

# Introduction

**Zenith** is a programming language that transpiles to **PHP 8.4+**. It is designed to explore modern functional syntax, safe-by-default database queries, and simplified concurrency on top of the PHP ecosystem.

---

# Why Zenith?

Zenith aims to solve specific ergonomic and safety pain points through syntax-level integration:

- **Simplified Concurrency**: Native `spawn` blocks that handle PHP Fibers automatically.
- **Safe-by-default SQL**: Inline SQL queries that are validated and parameterized at compile-time.
- **Functional Ergonomics**: Native pipe operator `|>` and built-in memoization support.
- **Static Analysis**: Built-in Zenith Analyzer for detecting common pitfalls like path traversal and SQL injection during development.

---

# Installation

## Quick Install (Experimental Only)

```bash
# Linux/MacOS
curl -fsSL https://neevets.github.io/zenith/install.sh | sh -s -- -y

# Windows (PowerShell)
iwr -useb https://neevets.github.io/zenith/install.ps1 | iex
```

> To protect users, Zenith will NO LONGER download managed PHP binaries automatically. You must have PHP 8.4 installed on your system or explicitly set `ZENITH_AUTO_INSTALL_RUNTIME=1`.

See full installer usage in [docs/installation.md](docs/installation.md).

# Documentation

- [Syntax & Types](docs/syntax.md)
- [Functional Programming](docs/functional.md)
- [Modern PHP Features](docs/modern_php.md)
- [Architecture & Security](docs/architecture.md)
- [Installation](docs/installation.md)

---

# License

This project is licensed under the **GNU GPL v3 License**.

---

# Contributing

Contributions are welcome! Please read the [CONTRIBUTING.md](CONTRIBUTING.md) file for more information.
