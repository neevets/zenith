# Functional Programming Patterns

Zenith introduces several functional programming constructs to make code more readable and maintainable.

## The Pipe Operator (`|>`)
The pipe operator allows you to chain function calls without nesting them, improving readability significantly.

### Syntax
`lhs |> rhs(args)` is transformed by the transpiler into `rhs(lhs, args)`.

### Example
```zenith
"  zenith architecture  "
    |> trim()
    |> strtoupper()
    |> str_replace("ARCH", "ENG")
    |> print();
```

In standard PHP, this would be highly nested: `echo str_replace("ARCH", "ENG", strtoupper(trim("...")))`.

## Arrow Functions (`fn`)
Following PHP 8.0's footprint, Zenith provides concise arrow functions. They are ideal for high-order functions and short expressions.

### Characteristics
- Single-expression bodies.
- Implicit return of the expression result.
- Automatic capture of variables from the outer scope by value.

### Example
```zenith
let $users = ["Alice", "Bob", "Charlie"];
let $shout = fn($name: string): string => strtoupper($name);

// Transpiles to: fn(string $name): string => strtoupper($name)
```

## High-Order Functions
Since Zenith transpiles to PHP, any standard PHP high-order function (like `array_map`, `array_filter`) can be used seamlessly with Zenith's arrow functions.
