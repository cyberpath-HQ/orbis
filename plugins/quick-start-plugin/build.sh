#!/bin/bash
set -euo pipefail

# cargo build --target wasm32-unknown-unknown --release -p quick-start-plugin
cargo build --target wasm32-unknown-unknown -p quick-start-plugin
# cp ../../target/wasm32-unknown-unknown/release/quick_start_plugin.wasm ./quick_start_plugin.wasm
cp ../../target/wasm32-unknown-unknown/debug/quick_start_plugin.wasm ./quick_start_plugin.wasm