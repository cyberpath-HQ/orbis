#!/bin/bash
set -e

echo "Building Hello Plugin..."

# Build the WASM module
cargo build --target wasm32-unknown-unknown --release

# Copy WASM file
cp target/wasm32-unknown-unknown/release/hello_plugin.wasm hello_plugin.wasm

echo "Creating plugin variants..."

# 1. Unpacked with external manifest
echo "  - unpacked-external/"
mkdir -p unpacked-external
cp hello_plugin.wasm unpacked-external/
cp manifest.json unpacked-external/

# 2. Unpacked with embedded manifest (no manifest.json)
echo "  - unpacked-embedded/"
mkdir -p unpacked-embedded
cat manifest.json | python3 add_custom_section.py hello_plugin.wasm -s manifest -o unpacked-embedded/hello_plugin.wasm

# 3. Standalone WASM with embedded manifest
echo "  - standalone.wasm"
cat manifest.json | python3 add_custom_section.py hello_plugin.wasm -s manifest -o standalone.wasm

# 4. Packed ZIP with external manifest
echo "  - packed-external.zip"
cd unpacked-external
zip -q ../packed-external.zip hello_plugin.wasm manifest.json
cd ..

# 5. Packed ZIP with embedded manifest
echo "  - packed-embedded.zip"
cd unpacked-embedded
zip -q ../packed-embedded.zip hello_plugin.wasm
cd ..

echo ""
echo "Build complete! Created:"
echo "  - unpacked-external/        (folder with WASM + manifest.json)"
echo "  - unpacked-embedded/        (folder with WASM containing embedded manifest)"
echo "  - standalone.wasm           (single WASM with embedded manifest)"
echo "  - packed-external.zip       (ZIP with WASM + manifest.json)"
echo "  - packed-embedded.zip       (ZIP with WASM containing embedded manifest)"
