# Modern PHP (8.5+)

Zenith is not just a language; it's a compiler designed to maximize the potential of modern PHP runtimes.

## Match Expressions
Match expressions in Zenith map directly to PHP 8.0 `match(...) { ... }`. Unlike traditional `switch` statements, match is an expression (it returns a value) and enforces strict identity checks.

### Example
```zenith
$res = match($code) {
    200, 201 => "Success",
    404 => "Not Found",
    500 => "Server Error",
    default => "Unknown Status"
};
```

## Nullsafe Operator (`?->`)
Zenith provides the `?->` operator for safe property and method access on nullable objects, transpiling to PHP 8.0's nullsafe operator.

### Example
```zenith
// If $user is null, $name will be null instead of throwing an error.
$name = $user?->profile?->getName();
```

## Constructor Property Promotion
(Internal Implementation) Zenith's class system (forthcoming) will leverage PHP 8.0 property promotion to minimize boilerplate.

## Fiber Support (Experimental)
Zenith is exploring native asynchronous abstractions using PHP 8.5 Fibers, allowing for concurrent "green thread" execution without the complexity of traditional async/await.
