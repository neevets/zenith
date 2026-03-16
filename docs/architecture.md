# Architecture & Security

Zenith is built for the enterprise, where safety and security are paramount.

## Quantum Shield: Static Analysis
The Quantum Shield is Zenith's static analysis engine. It runs after parsing and before transpilation, ensuring that only valid code reaches the PHP runtime.

### Key Responsibilities:
- **Type Checking**: Validates all `let` assignments and function signatures.
- **Null Safety**: Analyzes object access paths to recommend nullsafe operators where appropriate.
- **SQL Validation**: Inspects `db.query` calls to verify table and column names against a known schema.

## Security Model: The Sandboxed Runner
Zenith's runner implements a strict capability-based security model. By default, Zenith scripts cannot interact with the host system.

### Permission Flags:
- `--allow-read`: Grants read access to the file system.
- `--allow-net`: Enables network-related PHP functions (CURL, Guzzle, etc.).
- `--allow-env`: Permits access to environment variables.

```bash
zenith run my_app.zen --allow-net --allow-read
```

## Compilation & Bundling
The `zenith bundle` command utilizes PHP's PHAR capabilities to create self-contained executables. These bundles include the Zenith runtime logic and the transpiled PHP, making deployment as simple as a single binary.
