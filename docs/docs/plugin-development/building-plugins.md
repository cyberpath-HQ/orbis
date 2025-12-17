---
sidebar_position: 5
title: Building Plugins
description: Building, packaging, and distributing plugins
---

# Building Plugins

Learn how to build, package, and distribute your Orbis plugins.

## Build Process Overview

```mermaid
flowchart LR
    A[Rust Source] --> B[cargo build]
    B --> C[WASM Binary]
    C --> D[Add Manifest]
    D --> E[Final Plugin]
```

## Project Structure

```
my-plugin/
├── Cargo.toml
├── manifest.json
├── build.sh
├── add_custom_section.py
├── src/
│   └── lib.rs
└── target/
    └── wasm32-unknown-unknown/
        └── release/
            └── my_plugin.wasm
```

## Cargo Configuration

### Cargo.toml

```toml
[package]
name = "my-plugin"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[profile.release]
opt-level = "s"           # Optimize for size
lto = true                # Link-time optimization
strip = true              # Strip symbols
codegen-units = 1         # Single codegen unit
panic = "abort"           # Abort on panic (smaller)
```

### Optimization Tips

| Option | Effect | Trade-off |
|--------|--------|-----------|
| `opt-level = "s"` | Small binary | Slightly slower |
| `opt-level = "z"` | Smallest binary | Slower |
| `lto = true` | Smaller, faster | Longer compile |
| `strip = true` | Remove debug symbols | No stack traces |

## Build Script

### Basic Build (build.sh)

```bash
#!/bin/bash
set -e

echo "Building plugin..."

# Build the WASM binary
cargo build --target wasm32-unknown-unknown --release

# Get the output path
WASM_FILE="target/wasm32-unknown-unknown/release/my_plugin.wasm"

# Embed manifest as custom section
python3 add_custom_section.py "$WASM_FILE" manifest.json

echo "Build complete: $WASM_FILE"
echo "Size: $(du -h $WASM_FILE | cut -f1)"
```

### Manifest Embedding Script (add_custom_section.py)

```python
#!/usr/bin/env python3
"""Embed manifest.json as a custom section in WASM binary."""

import sys
import json
import struct

def add_custom_section(wasm_path: str, manifest_path: str):
    # Read manifest
    with open(manifest_path, 'r') as f:
        manifest = json.load(f)
    manifest_bytes = json.dumps(manifest, separators=(',', ':')).encode('utf-8')
    
    # Read WASM
    with open(wasm_path, 'rb') as f:
        wasm = bytearray(f.read())
    
    # Validate WASM magic
    if wasm[:4] != b'\x00asm':
        raise ValueError("Invalid WASM file")
    
    # Create custom section
    section_name = b'manifest'
    name_len = len(section_name)
    content = bytes([name_len]) + section_name + manifest_bytes
    
    # Section header: type (0) + length (LEB128)
    section = bytearray([0])  # Custom section type
    section.extend(encode_leb128(len(content)))
    section.extend(content)
    
    # Insert after WASM header (8 bytes)
    result = wasm[:8] + section + wasm[8:]
    
    # Write result
    with open(wasm_path, 'wb') as f:
        f.write(result)
    
    print(f"Added manifest section ({len(manifest_bytes)} bytes)")

def encode_leb128(value: int) -> bytes:
    """Encode unsigned integer as LEB128."""
    result = []
    while True:
        byte = value & 0x7f
        value >>= 7
        if value:
            byte |= 0x80
        result.append(byte)
        if not value:
            break
    return bytes(result)

if __name__ == '__main__':
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <wasm_file> <manifest.json>")
        sys.exit(1)
    add_custom_section(sys.argv[1], sys.argv[2])
```

## Alternative: External Manifest

Instead of embedding, you can use an external manifest file:

```
my-plugin/
├── my_plugin.wasm
└── manifest.json
```

Orbis searches for manifests in this order:
1. Custom section in WASM binary
2. `manifest.json` next to WASM file
3. `<plugin_name>.json` next to WASM file

## Development Workflow

### Watch Mode

For rapid iteration, use cargo-watch:

```bash
# Install cargo-watch
cargo install cargo-watch

# Auto-rebuild on changes
cargo watch -x 'build --target wasm32-unknown-unknown --release'
```

### Debug Builds

For development with better error messages:

```toml
[profile.dev]
opt-level = 0
debug = true
panic = "unwind"  # Better stack traces
```

```bash
cargo build --target wasm32-unknown-unknown
# Output: target/wasm32-unknown-unknown/debug/my_plugin.wasm
```

### Validation

Before deploying, validate your plugin:

```bash
# Check WASM validity
wasm-validate target/wasm32-unknown-unknown/release/my_plugin.wasm

# Verify exports
wasm2wat target/wasm32-unknown-unknown/release/my_plugin.wasm | grep '(export'

# Check size
ls -lh target/wasm32-unknown-unknown/release/my_plugin.wasm
```

Expected exports:
```wat
(export "init" (func $init))
(export "execute" (func $execute))
(export "cleanup" (func $cleanup))
(export "memory" (memory 0))
(export "alloc" (func $alloc))
(export "dealloc" (func $dealloc))
```

## Packaging

### Single-File Distribution

With embedded manifest, the WASM file is self-contained:

```bash
cp target/wasm32-unknown-unknown/release/my_plugin.wasm dist/
```

### Bundle with Assets

If your plugin has assets:

```
my-plugin-bundle/
├── my_plugin.wasm
├── manifest.json    # Optional if embedded
└── assets/
    ├── icon.svg
    └── readme.md
```

Create a zip:
```bash
zip -r my-plugin-v1.0.0.zip my-plugin-bundle/
```

## Installation

### Plugin Directory

Plugins are loaded from `ORBIS_PLUGINS_DIR`:

```bash
# Default location
~/.orbis/plugins/

# Or set custom location
ORBIS_PLUGINS_DIR=/path/to/plugins
```

### Installing a Plugin

```bash
# Copy WASM file
cp my_plugin.wasm ~/.orbis/plugins/

# Or copy bundle
unzip my-plugin-v1.0.0.zip -d ~/.orbis/plugins/
```

### Hot Reload

Orbis watches the plugin directory. New or updated plugins are automatically loaded without restart.

To force reload:
1. Touch the WASM file: `touch my_plugin.wasm`
2. Restart Orbis

## Size Optimization

### Minimize Dependencies

```toml
# ✅ Feature-minimal
serde = { version = "1.0", default-features = false, features = ["derive", "alloc"] }

# ❌ Avoid full features
serde = { version = "1.0", features = ["derive"] }  # Pulls in std
```

### Use wasm-opt

```bash
# Install binaryen
apt install binaryen

# Optimize WASM
wasm-opt -Os -o optimized.wasm my_plugin.wasm

# Aggressive optimization
wasm-opt -Oz -o smallest.wasm my_plugin.wasm
```

### Size Comparison

| Build | Typical Size |
|-------|-------------|
| Debug | 500KB - 2MB |
| Release | 100KB - 500KB |
| Release + LTO | 50KB - 200KB |
| Release + wasm-opt | 30KB - 100KB |

## Troubleshooting

### "Missing export: init"

Your plugin must export the required functions:

```rust
#[no_mangle]
pub extern "C" fn init() -> i32 { 0 }

#[no_mangle]
pub extern "C" fn execute(ptr: i32, len: i32) -> i32 { 0 }

#[no_mangle]
pub extern "C" fn cleanup() { }
```

### "Invalid WASM file"

Check the binary is valid:
```bash
wasm-validate my_plugin.wasm
```

Common causes:
- Corrupted download
- Built for wrong target (not wasm32-unknown-unknown)
- Manifest embedding script error

### "Manifest not found"

Ensure manifest is either:
1. Embedded as custom section (run `add_custom_section.py`)
2. Present as `manifest.json` next to WASM file

### Large File Size

1. Enable release optimizations in `Cargo.toml`
2. Run `wasm-opt -Os` on the output
3. Remove unnecessary dependencies
4. Check for debug symbols: `wasm-opt --strip-debug`

### Memory Issues

If your plugin crashes with memory errors:
1. Check `alloc`/`dealloc` implementations
2. Ensure strings are properly null-terminated
3. Use Rust's allocator correctly

## CI/CD Example

### GitHub Actions

```yaml
name: Build Plugin

on:
  push:
    branches: [main]
  release:
    types: [created]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          targets: wasm32-unknown-unknown
      
      - name: Install binaryen
        run: sudo apt-get install -y binaryen
      
      - name: Build
        run: cargo build --target wasm32-unknown-unknown --release
      
      - name: Optimize
        run: wasm-opt -Os -o optimized.wasm target/wasm32-unknown-unknown/release/my_plugin.wasm
      
      - name: Embed manifest
        run: python3 add_custom_section.py optimized.wasm manifest.json
      
      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: my-plugin
          path: optimized.wasm
```

## Next Steps

- **[Testing Plugins](./testing-plugins)** - Test your plugins
- **[Best Practices](./best-practices)** - Production-ready plugins
- **[Components](../components/overview)** - UI component reference
