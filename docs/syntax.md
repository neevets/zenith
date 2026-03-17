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
let $x: int = "string"; // Fatal error in strict mode
```

## 2. Advanced Functional Paradigm

### Constant Folding (Compile-Time Evaluation)
The Zenith transpiler automatically evaluates constant expressions during the code generation phase.
```zenith
let $x = 10 + 20 * 2; // Transpiles directly to $x = 50;
let $s = "Hello " + "World"; // Transpiles to $s = "Hello World";
```

### Pipe Operator (`|>`) & Placeholders
The pipe operator chains expressions by injecting the left-hand value as the first argument of the right-hand call.
```zenith
$data |> process() |> format("json") |> println();
```

### Modern Closures
Short closure syntax provides concise lambda definitions with implicit scope capture.
```zenith
let $multiplier = ($n) => $n * 10;
let $typed_closure = ($x: int): int => $x ** 2;
```

### Memoization Decorator (`@memoize`)
Functions can be automatically cached using the `@memoize` attribute. This wraps the function body in a closure and utilizes an internal `static $memo_cache`.
```zenith
@memoize
function expensive_calc($n) {
    // Computed only once for each distinct $n
    return $n * 3.1415;
}
```

## 3. First-Class SQL & Data Handling

### SQL Query Blocks
SQL is a first-class citizen in Zenith. Instead of strings, use `query` blocks that are analyzed for security and syntax.
```zenith
// Global or local DB connection
db.connect("mysql:host=localhost;dbname=prod");

let $results = query {
    SELECT u.id, u.email 
    FROM users u 
    WHERE u.status == 'active' 
    LIMIT 10
};
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

## 5. Concurrency Model: Fibers & Spawn
Zenith abstracts PHP 8.5+ Fibers into high-level concurrency blocks.
```zenith
let $task = spawn {
    println("Task started");
    yield "Paused";
    println("Task resumed");
};

$task.resume(); // Output: Task started
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
let $pdo = \\PDO { "sqlite::memory:" };
let $time = \\time();
```
