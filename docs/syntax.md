# Zenith Language Specification: Syntax & Types

Zenith provides a robust, C-style syntax optimized for modern PHP transpilation.

## Variables & Scoping
Variables in Zenith are denoted with the `$` prefix and are block-scoped.

```zenith
let $message: string = "Hello, Zenith";
let $count: int = 42;
```

## Type System
Zenith features a strict static type system called **Quantum Shield**. While Zenith remains familiar, it enforces type correctness at compile-time.

### Basic Types
- `int`: 64-bit integers.
- `string`: UTF-8 string literals.
- `bool`: Boolean values (`true`, `false`).
- `float`: Floating point numbers.
- `void`: Used exclusively for function return types.
- `any`: Opt-out of static analysis for a specific variable.

### Union Types (Advanced)
Zenith leverages PHP 8's union types during transpilation.
```zenith
function getValue(id: int): string|null { ... }
```

## Control Structures

### While Loop
Standard C-style while loop.
```zenith
while ($count > 0) {
    print("Count: " + $count);
    let $count = $count - 1;
}
```

### If/Else
Standard conditional branches.
```zenith
if ($count == 0) {
    print("Blast off!");
} else {
    print("Waiting...");
}
```

## Functions
Functions support strict parameter typing and return type annotations.

```zenith
function add(a: int, b: int): int {
    return $a + $b;
}
```

### Parameters
Parameters can be defined with or without explicit types. If a type is provided, the transpiler enforces it via PHP type hints.
