#!/usr/bin/env bash
# Dotnet Rust Release Packaging Script (Bash / CI)
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "=========================================="
echo "  Dotnet Rust Native - Release Packaging"
echo "=========================================="

echo "[1/4] Compiling release binaries..."
cargo build --release

RELEASE_DIR="$ROOT_DIR/target/x86_64-pc-windows-msvc/release"
if [ ! -f "$RELEASE_DIR/dotnet-cli.exe" ]; then
    RELEASE_DIR="$ROOT_DIR/target/release"
fi

DIST_DIR="$ROOT_DIR/dist/Dotnet-Rust-win-x64"
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

echo "[2/4] Assembling release directory..."
cp "$RELEASE_DIR/dotnet-cli.exe" "$DIST_DIR/dotnet.exe"
cp "$RELEASE_DIR/dotnet-cli.exe" "$DIST_DIR/dotnet-cli.exe"
cp "$RELEASE_DIR/dotnet-app.exe" "$DIST_DIR/dotnet-app.exe"

[ -f .env.example ] && cp .env.example "$DIST_DIR/"
[ -f README.md ] && cp README.md "$DIST_DIR/"
[ -f LICENSE ] && cp LICENSE "$DIST_DIR/"

echo "[3/4] Release distribution assembled at $DIST_DIR"
ls -la "$DIST_DIR"

echo "[4/4] Package complete!"
