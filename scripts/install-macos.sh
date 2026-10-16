#!/bin/sh
# Builds the release bundle and installs it to ~/Applications/Cerebro.app.
set -e
cd "$(dirname "$0")/.."
npm run tauri -- build --bundles app
pkill -x cerebro-app 2>/dev/null || true
pkill -x Cerebro 2>/dev/null || true
rm -rf "$HOME/Applications/Cerebro.app"
mkdir -p "$HOME/Applications"
cp -R target/release/bundle/macos/Cerebro.app "$HOME/Applications/"
echo "Installed: ~/Applications/Cerebro.app"
open "$HOME/Applications/Cerebro.app"
