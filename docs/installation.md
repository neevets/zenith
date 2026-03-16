# Installation

## One-line installer

```bash
curl -fsSL https://neevets.github.io/zenith/install.sh | sh -s -- -y
```

Alternative with `wget`:

```bash
wget -qO- https://neevets.github.io/zenith/install.sh | sh -s -- -y
```

## Installer options

```text
-q, --quiet         Reduce output
-y                  Skip confirmation prompt
    --to <DIR>      Install directory
-h, --help          Show help
```

## Build from source

```bash
git clone https://github.com/neevets/zenith
cd zenith
cargo build --release
sudo mv target/release/zenith /usr/local/bin/
```
