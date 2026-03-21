# Advanced Language Specification

This document provides a deep dive into the Zenith syntax, language semantics, and advanced compiler features.

## 1. Compiler Directives & Strict Mode
Zenith supports file-level directives to control the behavior of the `Quantum Shield` analyzer.

### `#!strict`
The `#![strict]` directive converts semantic warnings into fatal compilation errors. 
- **Undefined Symbols**: Any variable, function, or struct access not present in the symbol table will stop the build.
- **Strict Typing**: Type mismatches in assignments or function returns become fatal.

```zenith
#![strict]
$x: int = 10;
$y = 20;
$res = $x + $y;
```

## 2. Advanced Functional Paradigm

### Constant Folding (Compile-Time Evaluation)
The Zenith transpiler automatically evaluates constant expressions during the code generation phase.
```zenith
let $x: int = 10 + 20 * 2; // Transpiles directly to $x = 50;
let $s = "Hello " . "World"; // Transpiles to $s = "Hello World";
```

### Pipe Operator (`|>`) & Placeholders
The pipe operator chains expressions by injecting the left-hand value as the first argument of the right-hand call.
```zenith
$data |> process() |> format("json") |> println();
```

### Modern Closures
Short closure syntax provides concise lambda definitions with implicit scope capture.
```zenith
$multiplier = ($n) => $n * 10;
$typed_closure = ($x: int): int => $x ** 2;
```

### Memoization Decorator (`@memoize`)
Functions can be automatically cached using the `@memoize` attribute. This wraps the function body in a closure and utilizes an internal `static $memo_cache`.
```zenith
@memoize
expensive_calc($n) {
    // Computed only once for each distinct $n
    return $n * 3.1415;
}
```

## 3. First-Class SQL & Data Handling

### SQL Query Blocks
SQL is a first-class citizen in Zenith. Instead of strings, use `query` blocks that are analyzed for security and syntax.
```zenith
// Global or local DB connection
db->connect("mysql:host=localhost;dbname=prod");

$results = query {
    SELECT u.id, u.email 
    FROM users u 
    WHERE u.status == 'active' 
    LIMIT 10
}; // Checked against schema at compile time
```

### Sanitization Pipeline (`!>`)
Standardize security by using the sanitization operator before output.
```zenith
let $input = $_GET["html_content"];
println($input !> "html"); // Escapes XSS automatically
```

## 4. Metadata & Attributes
Zenith supports generic metadata on definitions. These are captured in the AST and can be used by the compiler or external tools.
```zenith
@Table("products")
@Serializable
struct Product {
    id: int,
    price: float
}
```

## 5. Concurrency Model: I/O Parallelism
Zenith abstracts PHP 8.4+ Fibers into high-level `spawn` blocks, ideal for parallel I/O tasks like fetching multiple APIs during a single web request.

```zenith
$api1 = spawn { fetch("https://api.v1.com") };
$api2 = spawn { fetch("https://api.v2.com") };

// Wait and collect results in parallel
$results = [$api1->resume(), $api2->resume()]; 
```

## 6. Type System & Structs
Zenith uses a nominal type system with support for composition and inheritance.

### Struct Inheritance
```zenith
struct Entity {
    id: int,
    created_at: string
}

struct User : Entity {
    username: string
}
```

## 7. Native Interoperability
Access native PHP functions or classes using the double-backslash `\\` prefix.
```zenith
$pdo = \\PDO { "sqlite::memory:" };
$time = \\time();
```

## 8. Typed ORM (Active Record)
Zenith turns simple `struct`s into powerful database models using the `@Table` attribute. This provides a zero-overhead, type-safe ORM experience.

### Model Definition
```zenith
@Table("users")
struct User {
    id: int,
    name: string,
    email: string,
    active: bool
}
```

### Usage
```zenith
// Static retrieval
$user = User::find(1);

// Static where-clause (transpiles to optimized SQL)
$active_users = User::where("active", true)->get();

// Method-based persistence
$user->name = "New Name";
$user->save();
```
Zenith handles the mapping between your struct fields and database columns automatically at compile-time when using `query` blocks or ORM methods.
