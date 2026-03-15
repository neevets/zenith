# Contributing to Zenith

We welcome contributions from the community to help improve the language, tooling, and documentation.

---

# Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Ways to Contribute](#ways-to-contribute)
3. [Getting Started](#getting-started)
4. [Development Setup](#development-setup)
5. [Project Structure](#project-structure)
6. [Coding Guidelines](#coding-guidelines)
7. [Commit Guidelines](#commit-guidelines)
8. [Pull Request Process](#pull-request-process)

---

# Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment for everyone.

- Be respectful and constructive
- Avoid harassment or discrimination
- Help others learn and improve

---

# Ways to Contribute

You can contribute in several ways:

- Fix bugs
- Implement new language features
- Improve documentation
- Optimize the compiler or runtime
- Write examples and tutorials
- Report issues or suggest improvements

---

# Getting Started

1. Fork the repository
2. Clone your fork

```bash
git clone https://github.com/neevets/zenith.git
cd zenith
````

3. Create a new branch

```bash
git checkout -b feature/my-feature
```

---

# Development Setup

Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Build the project:

```bash
cargo build
```

Run tests:

```bash
cargo test
```

Run the CLI locally:

```bash
cargo run -- index.zen
```

---

# Commit Guidelines

Use clear commit messages.

Recommended format:

```
type(scope): description
```

Examples:

```
feat(parser): add pipe operator support
fix(transpiler): correct match expression generation
docs: improve installation guide
refactor(analyzer): simplify type inference
```

Types:

* `feat` → new feature
* `fix` → bug fix
* `docs` → documentation
* `refactor` → code restructuring
* `test` → tests
* `chore` → maintenance

---

# Pull Request Process

1. Ensure your branch is up to date with `main`
2. Run all tests
3. Ensure code formatting and lint checks pass
4. Open a Pull Request
5. Describe clearly what your PR does

A good PR should include:

* A clear description
* Related issue (if applicable)
* Tests when needed

---

# Reporting Issues

If you find a bug or want to request a feature, please open an issue and include:

* Zenith version
* OS and environment
* Steps to reproduce
* Expected vs actual behavior

---

# Thank You!

Your contributions help make **Zenith** better for everyone.