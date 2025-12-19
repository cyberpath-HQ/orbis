#!/bin/bash

# Build the test plugin WASM module
cargo build --release --target wasm32-unknown-unknown

# Copy to plugin directory from workspace target
cp ../../target/wasm32-unknown-unknown/release/test_plugin.wasm ./test_plugin.wasm

echo "Build complete: test_plugin.wasm"
