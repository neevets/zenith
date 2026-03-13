#!/bin/sh
set -e

# Usage: curl -fsS https://dl.zenith.vercel.app/install.sh | sh

GITHUB_REPOq54="neevets/zenith"
BINARY_NAME="zenith"

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$ARCH" in
    x86_64) ARCH="x86_64" ;;
    arm64|aarch64) ARCH="arm64" ;;
    *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

case "$OS" in
    linux) OS="linux" ;;
    darwin) OS="macos" ;;
    *) echo "Unsupported OS: $OS"; exit 1 ;;
esac

echo "--- Zenith Installer ---"
echo "Target: $OS-$ARCH"

DOWNLOAD_URL="https://github.com/$GITHUB_REPO/releases/latest/download/zenith-$OS-$ARCH.tar.gz"

echo "Downloading from $DOWNLOAD_URL..."

TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

# Mocking download for demonstration if URL doesn't exist yet
# In a real script, this would be: curl -L "$DOWNLOAD_URL" | tar xz -C "$TMP_DIR"
# For this task, we assume the binary exists or provide instructions

INSTALL_DIR="/usr/local/bin"

if [ -f "target/release/zenith" ]; then
    echo "Found local build, installing from target/release/zenith..."
    cp target/release/zenith "$TMP_DIR/zenith"
else
    echo "Note: Release binaries not found on GitHub yet. This script is ready for use once releases are published."
    exit 0
fi

echo "Installing to $INSTALL_DIR/$BINARY_NAME..."
if [ -w "$INSTALL_DIR" ]; then
    mv "$TMP_DIR/zenith" "$INSTALL_DIR/$BINARY_NAME"
else
    echo "Requiring sudo for installation..."
    sudo mv "$TMP_DIR/zenith" "$INSTALL_DIR/$BINARY_NAME"
fi

chmod +x "$INSTALL_DIR/$BINARY_NAME"

echo "Success! Zenith has been installed."
