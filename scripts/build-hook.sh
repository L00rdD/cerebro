#!/bin/sh
# Builds cerebro-hook and places it where Tauri expects sidecars:
# src-tauri/binaries/cerebro-hook-<target-triple>[.exe]
# Usage: build-hook.sh [target-triple]   (defaults to the host triple)
set -e
cd "$(dirname "$0")/.."
if [ -n "$1" ]; then
  triple="$1"
  cargo build -p cerebro-hook --release --target "$triple"
  out="target/$triple/release"
else
  triple=$(rustc -vV | sed -n 's/^host: //p')
  cargo build -p cerebro-hook --release
  out="target/release"
fi
ext=""
case "$triple" in *windows*) ext=".exe" ;; esac
mkdir -p src-tauri/binaries
cp "$out/cerebro-hook$ext" "src-tauri/binaries/cerebro-hook-$triple$ext"
echo "sidecar: src-tauri/binaries/cerebro-hook-$triple$ext"
