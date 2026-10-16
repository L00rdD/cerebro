#!/bin/sh
# Runs a command with a real Rust toolchain first in PATH.
# Needed because a `cargo` on PATH may be a wrapper that is not Cargo
# (e.g. ~/.local/bin/cargo -> Hermes), and the Tauri CLI calls `cargo` from PATH.
set -e
for dir in "${CARGO_HOME:-$HOME/.cargo}/bin" /opt/homebrew/opt/rustup/bin /usr/local/opt/rustup/bin; do
  if [ -x "$dir/cargo" ] && "$dir/cargo" --version 2>/dev/null | grep -q '^cargo '; then
    PATH="$dir:$PATH"
    export PATH
    exec "$@"
  fi
done
if command -v cargo >/dev/null && cargo --version 2>/dev/null | grep -q '^cargo '; then
  exec "$@"
fi
echo "with-rust: no Rust toolchain found (install rustup)" >&2
exit 1
