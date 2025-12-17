---
sidebar_position: 3
title: WASM Plugins
description: Building backend plugins with Rust and WebAssembly
---

# WASM Plugins

WASM (WebAssembly) plugins allow you to write backend logic in Rust that runs in a sandboxed environment.

## Overview

WASM plugins can:
- Handle API route requests
- Execute complex business logic
- Process and transform data
- Interact with the database
- Call external APIs (with permissions)

## Prerequisites

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown
```

## Project Setup

### Initialize Plugin

```bash
mkdir my-plugin
cd my-plugin

# Create Rust library
cargo init --lib
```

### Configure Cargo.toml

```toml
[package]
name = "my-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]  # Required for WASM

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[profile.release]
opt-level = "s"      # Optimize for size
lto = true           # Link-time optimization
strip = true         # Strip debug symbols
```

### Required Entry Points

Every WASM plugin must export these functions:

```rust
// src/lib.rs

/// Initialize the plugin
/// Called once when the plugin loads
/// Return 0 for success, non-zero for error
#[no_mangle]
pub extern "C" fn init() -> i32 {
    // Initialization logic here
    0
}

/// Execute plugin logic
/// Called for general plugin execution
#[no_mangle]
pub extern "C" fn execute() -> i32 {
    0
}

/// Clean up resources
/// Called when the plugin unloads
#[no_mangle]
pub extern "C" fn cleanup() -> i32 {
    0
}
```

## Route Handlers

Route handlers process API requests.

### Defining Routes

In `manifest.json`:

```json
{
  "routes": [
    {
      "path": "/api/greet",
      "method": "GET",
      "handler": "handle_greet"
    }
  ]
}
```

### Implementing Handlers

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct GreetResponse {
    message: String,
}

/// Handle GET /api/greet
#[no_mangle]
pub extern "C" fn handle_greet(input_ptr: *const u8, input_len: usize) -> *const u8 {
    // Parse input (JSON)
    let input = unsafe {
        std::slice::from_raw_parts(input_ptr, input_len)
    };
    
    // Create response
    let response = GreetResponse {
        message: "Hello from WASM!".to_string(),
    };
    
    // Serialize and return
    let json = serde_json::to_string(&response).unwrap();
    let bytes = json.into_bytes();
    let ptr = bytes.as_ptr();
    std::mem::forget(bytes);
    ptr
}
```

### Handler Input/Output

Handlers receive a JSON object with:

```json
{
  "params": { "id": "123" },    // Path parameters
  "query": { "page": "1" },     // Query string
  "body": { ... },              // Request body
  "headers": { ... },           // HTTP headers
  "user": { ... }               // Authenticated user (if any)
}
```

And return a JSON response:

```json
{
  "status": 200,
  "body": { ... },
  "headers": { ... }
}
```

## Memory Management

WASM has a linear memory model. Here's how to handle it:

### Allocating Memory

```rust
/// Allocate memory for host to write data
#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

/// Free allocated memory
#[no_mangle]
pub extern "C" fn dealloc(ptr: *mut u8, size: usize) {
    unsafe {
        let _ = Vec::from_raw_parts(ptr, 0, size);
    }
}
```

### Passing Strings

```rust
/// Get string from pointer and length
fn get_string(ptr: *const u8, len: usize) -> String {
    let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    String::from_utf8_lossy(slice).to_string()
}

/// Return string to host
fn return_string(s: String) -> *const u8 {
    let bytes = s.into_bytes();
    let ptr = bytes.as_ptr();
    std::mem::forget(bytes);
    ptr
}
```

## Host Functions

Plugins can call host functions provided by Orbis:

### Database Access

```rust
extern "C" {
    fn db_query(query_ptr: *const u8, query_len: usize) -> *const u8;
    fn db_execute(stmt_ptr: *const u8, stmt_len: usize) -> i32;
}

fn query_database(sql: &str) -> Result<Vec<Row>, Error> {
    let json = unsafe {
        let result_ptr = db_query(sql.as_ptr(), sql.len());
        // Parse result
    };
    // ...
}
```

### Logging

```rust
extern "C" {
    fn log_info(msg_ptr: *const u8, msg_len: usize);
    fn log_error(msg_ptr: *const u8, msg_len: usize);
    fn log_debug(msg_ptr: *const u8, msg_len: usize);
}

fn log(level: &str, message: &str) {
    match level {
        "info" => unsafe { log_info(message.as_ptr(), message.len()) },
        "error" => unsafe { log_error(message.as_ptr(), message.len()) },
        "debug" => unsafe { log_debug(message.as_ptr(), message.len()) },
        _ => {}
    }
}
```

### HTTP Requests

```rust
extern "C" {
    fn http_request(
        method_ptr: *const u8, method_len: usize,
        url_ptr: *const u8, url_len: usize,
        body_ptr: *const u8, body_len: usize,
    ) -> *const u8;
}

fn fetch(url: &str) -> Result<Response, Error> {
    let method = "GET";
    unsafe {
        let response_ptr = http_request(
            method.as_ptr(), method.len(),
            url.as_ptr(), url.len(),
            std::ptr::null(), 0,
        );
        // Parse response
    }
}
```

## Building

### Development Build

```bash
cargo build --target wasm32-unknown-unknown
```

Output: `target/wasm32-unknown-unknown/debug/my_plugin.wasm`

### Release Build

```bash
cargo build --target wasm32-unknown-unknown --release
```

Output: `target/wasm32-unknown-unknown/release/my_plugin.wasm`

### Optimization

For smaller WASM files:

```bash
# Install wasm-opt (from binaryen)
brew install binaryen  # macOS
apt install binaryen   # Linux

# Optimize
wasm-opt -Os -o optimized.wasm my_plugin.wasm
```

## Embedding the Manifest

You can embed the manifest directly in the WASM file:

```bash
# Using the provided script
cat manifest.json | python3 add_custom_section.py \
  my_plugin.wasm \
  -s manifest \
  -o my_plugin_with_manifest.wasm
```

This creates a self-contained WASM file that doesn't need a separate manifest.json.

## Complete Example

### Project Structure

```
my-plugin/
├── manifest.json
├── Cargo.toml
├── src/
│   └── lib.rs
└── build.sh
```

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
opt-level = "s"
lto = true
```

### src/lib.rs

```rust
use serde::{Deserialize, Serialize};

// Entry points

#[no_mangle]
pub extern "C" fn init() -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn execute() -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn cleanup() -> i32 {
    0
}

// Data structures

#[derive(Deserialize)]
struct CreateItemRequest {
    name: String,
    description: Option<String>,
}

#[derive(Serialize)]
struct Item {
    id: u64,
    name: String,
    description: String,
}

#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

// Handlers

#[no_mangle]
pub extern "C" fn list_items(_input_ptr: *const u8, _input_len: usize) -> *const u8 {
    let items = vec![
        Item { id: 1, name: "Item 1".into(), description: "First item".into() },
        Item { id: 2, name: "Item 2".into(), description: "Second item".into() },
    ];
    
    let response = ApiResponse {
        success: true,
        data: Some(items),
        error: None,
    };
    
    let json = serde_json::to_string(&response).unwrap();
    let bytes = json.into_bytes();
    let ptr = bytes.as_ptr();
    std::mem::forget(bytes);
    ptr
}

#[no_mangle]
pub extern "C" fn create_item(input_ptr: *const u8, input_len: usize) -> *const u8 {
    // Parse input
    let input = unsafe {
        std::slice::from_raw_parts(input_ptr, input_len)
    };
    
    let request: CreateItemRequest = match serde_json::from_slice(input) {
        Ok(req) => req,
        Err(e) => {
            let response = ApiResponse::<Item> {
                success: false,
                data: None,
                error: Some(format!("Invalid request: {}", e)),
            };
            let json = serde_json::to_string(&response).unwrap();
            let bytes = json.into_bytes();
            let ptr = bytes.as_ptr();
            std::mem::forget(bytes);
            return ptr;
        }
    };
    
    // Create item
    let item = Item {
        id: 1, // In reality, generate ID
        name: request.name,
        description: request.description.unwrap_or_default(),
    };
    
    let response = ApiResponse {
        success: true,
        data: Some(item),
        error: None,
    };
    
    let json = serde_json::to_string(&response).unwrap();
    let bytes = json.into_bytes();
    let ptr = bytes.as_ptr();
    std::mem::forget(bytes);
    ptr
}
```

### manifest.json

```json
{
  "name": "my-plugin",
  "version": "1.0.0",
  "routes": [
    { "path": "/api/items", "method": "GET", "handler": "list_items" },
    { "path": "/api/items", "method": "POST", "handler": "create_item" }
  ],
  "pages": [...],
  "wasm_entry": "my_plugin.wasm"
}
```

### build.sh

```bash
#!/bin/bash
set -e

echo "Building plugin..."
cargo build --target wasm32-unknown-unknown --release

echo "Copying WASM..."
cp target/wasm32-unknown-unknown/release/my_plugin.wasm ./

echo "Done!"
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_item() {
        let request = CreateItemRequest {
            name: "Test".into(),
            description: Some("Description".into()),
        };
        
        // Test your logic
    }
}
```

Run with:
```bash
cargo test
```

### Integration Tests

See [Testing Plugins](./testing-plugins) for integration testing strategies.

## Debugging

### Logging

Add debug output:

```rust
extern "C" {
    fn log_debug(msg_ptr: *const u8, msg_len: usize);
}

fn debug(msg: &str) {
    unsafe {
        log_debug(msg.as_ptr(), msg.len());
    }
}
```

### WASM Debugging Tools

```bash
# Inspect WASM file
wasm-objdump -x my_plugin.wasm

# Validate WASM
wasm-validate my_plugin.wasm
```

## Next Steps

- **[Building Plugins](./building-plugins)** - Build scripts and distribution
- **[Testing Plugins](./testing-plugins)** - Testing strategies
- **[Best Practices](./best-practices)** - Guidelines
