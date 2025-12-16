# Sample Plugins

This directory contains sample plugins for testing the Orbis plugin system.

## Hello Plugin

A simple "Hello World" plugin demonstrating all three plugin flavors.

### Building

```bash
cd hello-plugin
./build.sh
```

**Prerequisites:**

- Rust toolchain with `wasm32-unknown-unknown` target
- Python 3 (for adding custom sections to WASM files)

Install prerequisites:

```bash
rustup target add wasm32-unknown-unknown
```

### Plugin Variants

After building, the following variants are created:

1. **Unpacked with External Manifest** (`unpacked-external/`)
   - Directory containing `hello_plugin.wasm` and `manifest.json`
   - Manifest is read from the JSON file

2. **Unpacked with Embedded Manifest** (`unpacked-embedded/`)
   - Directory containing only `hello_plugin.wasm`
   - Manifest is embedded in WASM custom section

3. **Standalone** (`standalone.wasm`)
   - Single WASM file with embedded manifest
   - No additional files needed

4. **Packed with External Manifest** (`packed-external.zip`)
   - ZIP archive containing WASM and manifest.json
   - Manifest is extracted from JSON file in archive

5. **Packed with Embedded Manifest** (`packed-embedded.zip`)
   - ZIP archive containing only WASM
   - Manifest is embedded in WASM custom section

## Testing

Load these plugins using the Orbis plugin manager:

```rust
use orbis_plugin::{PluginManager, PluginSource};

// Unpacked with external manifest
let source = PluginSource::from_path(&PathBuf::from("sample-plugins/hello-plugin/unpacked-external"))?;

// Unpacked with embedded manifest
let source = PluginSource::from_path(&PathBuf::from("sample-plugins/hello-plugin/unpacked-embedded"))?;

// Standalone
let source = PluginSource::from_path(&PathBuf::from("sample-plugins/hello-plugin/standalone.wasm"))?;

// Packed with external manifest
let source = PluginSource::from_path(&PathBuf::from("sample-plugins/hello-plugin/packed-external.zip"))?;

// Packed with embedded manifest
let source = PluginSource::from_path(&PathBuf::from("sample-plugins/hello-plugin/packed-embedded.zip"))?;
```

All variants should load successfully and provide the same manifest information.
