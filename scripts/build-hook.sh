#!/bin/sh
# Builds cerebro-hook and places it where Tauri expects sidecars:
# src-tauri/binaries/cerebro-hook-<target-triple>[.exe]
set -e
cd "$(dirname "$0")/.."
cargo build -p cerebro-hook --release
triple=$(rustc -vV | sed -n 's/^host: //p')
ext=""
case "$triple" in *windows*) ext=".exe" ;; esac
mkdir -p src-tauri/binaries
cp "target/release/cerebro-hook$ext" "src-tauri/binaries/cerebro-hook-$triple$ext"
echo "sidecar: src-tauri/binaries/cerebro-hook-$triple$ext"
