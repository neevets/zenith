#!/bin/sh
has_local() {
    local _x
}

has_local 2>/dev/null || alias local=typeset

is_zsh() {
    [ -n "${ZSH_VERSION-}" ]
}

set -eu

ZENITH_REPO="${ZENITH_REPO:-neevets/zenith}"
ZENITH_BIN="${ZENITH_BIN:-zenith}"
ZENITH_QUIET=no
ZENITH_YES=no
ZENITH_INSTALL_DIR="${ZENITH_INSTALL_DIR:-}"
ZENITH_TMP_DIR=""
RETVAL=""

cleanup_tmp_dir() {
    if [ -n "${ZENITH_TMP_DIR:-}" ] && [ -d "$ZENITH_TMP_DIR" ]; then
        rm -rf "$ZENITH_TMP_DIR"
    fi
}

usage() {
    cat <<USAGE
Zenith installer

Usage: install.sh [OPTIONS]

Options:
  -q, --quiet         Reduce output
  -y                  Skip confirmation prompt
      --to <DIR>      Install directory
  -h, --help          Show this help
USAGE
}

print_line() {
    printf '%s\n' "$1" >&2
}

say() {
    if [ "$ZENITH_QUIET" = "no" ]; then
        print_line "$1"
    fi
}

warn() {
    print_line "warn: $1"
}

err() {
    print_line "error: $1"
}

check_cmd() {
    command -v "$1" >/dev/null 2>&1
}

need_cmd() {
    if ! check_cmd "$1"; then
        err "need '$1'"
        exit 1
    fi
}

ensure() {
    if ! "$@"; then
        err "command failed: $*"
        exit 1
    fi
}

normalize_arch() {
    local arch
    arch="$1"
    case "$arch" in
        x86_64|x86-64|amd64|x64)
            RETVAL="x86_64"
            ;;
        aarch64|arm64)
            RETVAL="arm64"
            ;;
        *)
            err "unsupported architecture: $arch"
            exit 1
            ;;
    esac
}

normalize_os() {
    local os
    os="$1"
    case "$os" in
        Linux)
            RETVAL="linux"
            ;;
        Darwin)
            RETVAL="macos"
            ;;
        MINGW*|MSYS*|CYGWIN*|Windows_NT)
            RETVAL="windows"
            ;;
        *)
            err "unsupported OS: $os"
            exit 1
            ;;
    esac
}

detect_target() {
    local os
    local arch
    os="$(uname -s)"
    arch="$(uname -m)"

    if [ "$os" = "Darwin" ] && [ "$arch" = "x86_64" ] && check_cmd sysctl; then
        if (sysctl hw.optional.arm64 2>/dev/null || true) | grep -q ': 1'; then
            arch="arm64"
        fi
    fi

    normalize_os "$os"
    local platform
    platform="$RETVAL"
    normalize_arch "$arch"
    local cpu
    cpu="$RETVAL"

    RETVAL="$platform-$cpu"
}

downloader_impl() {
    if check_cmd curl; then
        RETVAL="curl"
    elif check_cmd wget; then
        RETVAL="wget"
    else
        err "need curl or wget"
        exit 1
    fi
}

downloader() {
    local url
    local out
    local target
    url="$1"
    out="$2"
    target="$3"
    downloader_impl
    local d
    d="$RETVAL"
    if [ "$d" = "curl" ]; then
        if check_cmd grep && curl --help all 2>/dev/null | grep -q -- '--proto'; then
            curl --proto '=https' --tlsv1.2 --silent --show-error --fail --location "$url" --output "$out"
        else
            curl --silent --show-error --fail --location "$url" --output "$out"
        fi
    else
        wget "$url" -O "$out"
    fi
    local st=$?
    if [ $st -ne 0 ]; then
        err "artifact not found for $target"
    fi
    return $st
}

parse_args() {
    while [ $# -gt 0 ]; do
        case "$1" in
            -q|--quiet)
                ZENITH_QUIET=yes
                ;;
            -y)
                ZENITH_YES=yes
                ;;
            --to)
                shift
                if [ $# -eq 0 ]; then
                    err "--to requires a directory"
                    exit 1
                fi
                ZENITH_INSTALL_DIR="$1"
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            *)
                err "unknown option: $1"
                usage
                exit 1
                ;;
        esac
        shift
    done
}

main() {
    parse_args "$@"
    need_cmd uname
    need_cmd mktemp
    need_cmd tar
    need_cmd chmod
    need_cmd mkdir
    need_cmd rm

    detect_target
    local target
    target="$RETVAL"
    local platform
    local arch
    platform="${target%-*}"
    arch="${target#*-}"

    local binary_file
    if [ "$platform" = "windows" ]; then
        binary_file="${ZENITH_BIN}.exe"
    else
        binary_file="$ZENITH_BIN"
    fi

    local candidates
    if [ "$platform" = "windows" ]; then
        candidates="zenith-windows-${arch}.tar.gz zenith-windows-amd64.tar.gz"
    elif [ "$platform" = "linux" ]; then
        candidates="zenith-linux-${arch}.tar.gz zenith-linux-amd64.tar.gz"
    else
        candidates="zenith-macos-${arch}.tar.gz zenith-macos-arm64.tar.gz"
    fi

    ZENITH_TMP_DIR="$(mktemp -d)"
    trap 'cleanup_tmp_dir' EXIT INT TERM

    local selected
    selected=""
    local artifact
    for artifact in $candidates; do
        local url
        url="https://github.com/${ZENITH_REPO}/releases/latest/download/${artifact}"
        if downloader "$url" "$ZENITH_TMP_DIR/zenith.tar.gz" "$target" >/dev/null 2>&1; then
            selected="$artifact"
            break
        fi
    done

    if [ -z "$selected" ]; then
        err "no release artifact available for $target"
        exit 1
    fi

    local final_url
    final_url="https://github.com/${ZENITH_REPO}/releases/latest/download/${selected}"
    say "target: $target"
    say "downloading: $final_url"
    ensure downloader "$final_url" "$ZENITH_TMP_DIR/zenith.tar.gz" "$target"
    ensure tar -xzf "$ZENITH_TMP_DIR/zenith.tar.gz" -C "$ZENITH_TMP_DIR"

    local source_bin
    source_bin="$ZENITH_TMP_DIR/$binary_file"
    if [ ! -f "$source_bin" ]; then
        source_bin="$(find "$ZENITH_TMP_DIR" -type f \( -name "zenith" -o -name "zenith.exe" \) | head -n 1)"
    fi

    if [ -z "$source_bin" ] || [ ! -f "$source_bin" ]; then
        err "binary not found in archive"
        exit 1
    fi

    local install_dir
    if [ -n "$ZENITH_INSTALL_DIR" ]; then
        install_dir="$ZENITH_INSTALL_DIR"
    elif [ "$platform" = "windows" ]; then
        install_dir="$HOME/.local/bin"
    elif [ -w "/usr/local/bin" ]; then
        install_dir="/usr/local/bin"
    else
        install_dir="$HOME/.local/bin"
    fi

    if [ "$ZENITH_YES" = "no" ] && [ -t 0 ]; then
        printf 'Install %s to %s? [y/N] ' "$ZENITH_BIN" "$install_dir" >&2
        read -r answer || true
        case "${answer:-}" in
            y|Y|yes|YES)
                ;;
            *)
                err "aborted"
                exit 1
                ;;
        esac
    fi

    ensure mkdir -p "$install_dir"
    local dest
    dest="$install_dir/$ZENITH_BIN"

    if [ -f "$dest" ]; then
        say "existing installation detected at: $dest"
    elif check_cmd "$ZENITH_BIN"; then
        local existing_cmd
        existing_cmd="$(command -v "$ZENITH_BIN")"
        say "existing $ZENITH_BIN found in PATH at: $existing_cmd"
    fi

    if [ -w "$install_dir" ]; then
        ensure cp "$source_bin" "$dest"
        ensure chmod +x "$dest"
    else
        if check_cmd sudo; then
            ensure sudo cp "$source_bin" "$dest"
            ensure sudo chmod +x "$dest"
        else
            err "no write access to $install_dir and sudo not available"
            exit 1
        fi
    fi

    say "installed: $dest"
    say "run: $ZENITH_BIN --help"
}

main "$@"
