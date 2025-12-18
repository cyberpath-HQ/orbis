# orbis-plugin-api

Public API for developing Orbis plugins. This crate contains all the types and traits needed to create plugins for the Orbis application platform.

## Features

- **Plugin Manifest**: Define plugin metadata, dependencies, permissions, and routes
- **UI Schema**: Declarative JSON-based UI definition system for creating plugin pages
- **Type Safety**: Strongly-typed Rust definitions that serialize to JSON
- **Minimal Dependencies**: Only requires `serde`, `serde_json`, `semver`, and `thiserror`

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
orbis-plugin-api = "0.1"
```

### Creating a Plugin Manifest

```rust
use orbis_plugin_api::{PluginManifest, PluginPermission, PageDefinition};

let manifest = PluginManifest {
    name: "my-plugin".to_string(),
    version: "0.1.0".to_string(),
    description: "My awesome plugin".to_string(),
    permissions: vec![PluginPermission::DatabaseRead],
    pages: vec![
        PageDefinition {
            route: "/dashboard".to_string(),
            title: "Dashboard".to_string(),
            // ... more configuration
        }
    ],
    // ... more fields
};

// Validate the manifest
manifest.validate()?;
```

### Defining UI Pages

```rust
use orbis_plugin_api::{PageDefinition, ComponentSchema, StateFieldDefinition, StateFieldType};
use std::collections::HashMap;

let page = PageDefinition {
    route: "/users".to_string(),
    title: "User Management".to_string(),
    state: {
        let mut state = HashMap::new();
        state.insert("users".to_string(), StateFieldDefinition {
            field_type: StateFieldType::Array,
            default: Some(serde_json::json!([])),
            nullable: false,
            description: Some("List of users".to_string()),
        });
        state
    },
    sections: vec![
        ComponentSchema::new("Container")
            .with_id("main")
            .with_child(
                ComponentSchema::new("Table")
                    .with_prop("dataSource", serde_json::json!("state:users"))
            )
    ],
    // ... more configuration
};
```

### Building WASM Plugins

Your plugin should be compiled to WebAssembly:

```toml
[lib]
crate-type = ["cdylib"]

[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Link time optimization
strip = true        # Strip symbols
```

Build with:

```bash
cargo build --release --target wasm32-unknown-unknown
```

## Type Reference

### Core Types

- `PluginManifest`: Main manifest structure describing the plugin
- `PageDefinition`: UI page definition with routes, state, and components
- `ComponentSchema`: Individual UI component schema
- `Action`: Actions that can be triggered by UI events

### Permissions

Plugins can request various permissions:
- `DatabaseRead` / `DatabaseWrite`
- `FileRead` / `FileWrite`
- `Network`
- `System`
- `Shell` (dangerous - requires explicit user approval)
- `Environment`

### State Management

Pages can define typed state fields:
- `String`
- `Number`
- `Boolean`
- `Object`
- `Array`

## Error Handling

All validation methods return `orbis_plugin_api::Result<T>`:

```rust
use orbis_plugin_api::{Result, Error};

fn validate_my_manifest(manifest: &PluginManifest) -> Result<()> {
    manifest.validate()?;
    // Additional validation
    Ok(())
}
```
