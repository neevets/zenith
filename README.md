<div align="center">
  <img src="assets/logo.png" width="200" alt="Zenith Logo">
  <p><a href="https://zenithlang.vercel.app">zenithlang.vercel.app</a></p>
  <h1>Zenith</h1>
  <p><strong>Strict Types | Functional Patterns | PHP 8.1+ Target</strong></p>
</div>

---

## Zenith V2.6
Zenith is a high-performance programming language that transpiles to ultra-modern PHP 8.1+. It combines the ubiquity of PHP with the safety and ergonomics of modern functional languages.

## Core Pillars
- **Ultra-Modern Transpilation**: Targets PHP 8.1+ features like `match`, `fn`, and `?->`.
- **Quantum Shield**: Strict static type system that eliminates runtime type errors.
- **Functional Ergonomics**: Native pipe operator `|>` and high-order functions.
- **Secure by Design**: Sandboxed environment with explicit permissions (FS, Net, Env).
- **Zero-Config Dev**: Built-in server and bundler for a seamless DX.

## Installation
```bash
# Clone and build
git clone https://github.com/neevets/zenith
cd zenith
./scripts/build.sh

# Move to path
sudo mv zenith /usr/local/bin/
```

## Advanced Syntax at a Glance

### Pipe Operator & Arrow Functions
```zenith
let $add = fn($a: int, $b: int): int => $a + $b;

"  hello zenith  " 
    |> trim() 
    |> strtoupper() 
    |> print();
```

### Modern PHP 8.1 Constructs
```zenith
let $user = getUser(1);
print($user?->profile?->name);

let $status = match($code) {
    200 => "OK",
    404 => "Not Found",
    default => "Unknown"
};
```

## Documentation
- [Syntax & Types](docs/syntax.md)
- [Functional Programming](docs/functional.md)
- [Modern PHP Features](docs/modern_php.md)
- [Architecture & Security](docs/architecture.md)
