# orbis-plugin-api

Public API for developing Orbis WASM plugins. This crate contains all the types, host functions, and utilities needed to create secure, stateful plugins for the Orbis application platform.

## Features

- **WASM Runtime Interface**: Complete host function bindings for plugin development
- **Stateful by Default**: Built-in state management with JSON serialization
- **Plugin Manifest**: Define plugin metadata, dependencies, permissions, and routes
- **UI Schema**: Declarative JSON-based UI definition system for creating plugin pages
- **Type Safety**: Strongly-typed Rust definitions that serialize to JSON
- **Comprehensive Logging**: Multiple log levels (ERROR, WARN, INFO, DEBUG, TRACE)
- **Memory Management**: Automatic allocation/deallocation patterns
- **Minimal Dependencies**: Only requires `serde`, `serde_json`, `semver`, and `thiserror`

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
orbis-plugin-api = "0.1"
```

## Quick Start: WASM Plugin

### 1. Project Setup

```toml
[package]
name = "my-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
orbis-plugin-api = { path = "path/to/orbis-plugin-api" }
serde = { version = "1.0", features = ["derive"], default-features = false }
serde_json = { version = "1.0", default-features = false, features = ["alloc"] }

[profile.release]
opt-level = "z"
lto = true
strip = true
```

### 2. Plugin Implementation

```rust
#![no_std]

extern crate alloc;

use alloc::{format, string::String, vec::Vec};
use orbis_plugin_api::{LogLevel, PluginContext};
use serde::{Deserialize, Serialize};

// Import host functions
unsafe extern "C" {
    fn log(level: i32, ptr: *const u8, len: i32);
    fn state_get(key_ptr: *const u8, key_len: i32) -> *const u8;
    fn state_set(key_ptr: *const u8, key_len: i32, value_ptr: *const u8, value_len: i32) -> i32;
}

// Memory management (required)
#[unsafe(no_mangle)]
pub extern "C" fn allocate(size: i32) -> *mut u8 {
    let mut buf = Vec::with_capacity(size as usize);
    let ptr = buf.as_mut_ptr();
    core::mem::forget(buf);
    ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn deallocate(ptr: *mut u8, size: i32) {
    unsafe { let _ = Vec::from_raw_parts(ptr, 0, size as usize); }
}

// Lifecycle hooks
#[unsafe(no_mangle)]
pub extern "C" fn init() -> i32 {
    // Initialize plugin
    1 // Success
}

// Handler function
#[unsafe(no_mangle)]
pub extern "C" fn my_handler(context_ptr: i32, context_len: i32) -> i32 {
    // Process request and return response pointer
    0
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
```

### 3. Build

```bash
cargo build --target wasm32-unknown-unknown --release
```

## Host Functions Reference

### State Management

#### `state_get(key_ptr, key_len) -> ptr`

Get a value from plugin state. Returns NULL if key not found, otherwise returns pointer to length-prefixed JSON data.

#### `state_set(key_ptr, key_len, value_ptr, value_len) -> i32`

Set a value in plugin state. Returns 1 on success, 0 on failure.

#### `state_remove(key_ptr, key_len) -> i32`

Remove a value from plugin state. Returns 1 on success, 0 on failure.

### Logging

#### `log(level, ptr, len)`

Log a message at the specified level:

- `0` = ERROR
- `1` = WARN
- `2` = INFO
- `3` = DEBUG
- `4` = TRACE

See `examples/basic-plugin.rs` for complete working examples.

## Creating UI Manifests

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

## Type Reference

### Core Types

- `PluginContext`: Context passed to handler functions
- `PluginManifest`: Main manifest structure describing the plugin
- `PageDefinition`: UI page definition with routes, state, and components
- `ComponentSchema`: Individual UI component schema
- `Action`: Actions that can be triggered by UI events
- `LogLevel`: Enum for logging levels

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
