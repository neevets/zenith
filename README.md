# Zenith: The Modern PHP Runtime
**Official Site**: [zenithlang.vercel.app](https://zenithlang.vercel.app)

Zenith is a reimagined PHP experience. It's fast, secure, and built for the modern cloud.

## Features
- **Quantum Shield**: Predictive type safety that validates SQL queries at compile-time.
- **Mareas Memory Management**: Automated, deterministic memory handles via static analysis.
- **Z-Server**: Zero-config development server with real-time transpilation.
- **Z-Permissions**: Secure-by-default sandbox (no file/net access without flags).
- **Z-Bundle**: Compile your Zenith apps into self-contained binaries.

## Quick Start

### Installation
```bash
go build -o zenith ./cmd/zenith/main.go
sudo mv zenith /usr/local/bin/
```

### Run a Script
```bash
zenith run examples/benchmark.zen --allow-read
```

### Start Dev Server
```bash
zenith serve 8080
```

## Performance
- **Transpilation**: >350 lines/ms.
- **Latency**: <30ms total E2E for standard APIs.
