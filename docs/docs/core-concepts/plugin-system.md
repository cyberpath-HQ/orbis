---
sidebar_position: 2
title: Plugin System
description: How Orbis plugins work
---

# Plugin System

Orbis uses a plugin architecture where all application functionality is delivered through plugins. This guide explains how the plugin system works.

## Overview

Plugins in Orbis are self-contained modules that can:

- Define UI pages with JSON schemas
- Expose API routes
- Execute backend logic in WASM sandboxes
- Store and retrieve data
- Communicate with other plugins

## Plugin Types

### Manifest-Only Plugins

Simple plugins that define UI without backend logic:

```
my-plugin/
└── manifest.json
```

Perfect for:
- Dashboard pages
- Static content
- UI-only features

### WASM Plugins

Full-featured plugins with backend execution:

```
my-plugin/
├── manifest.json
├── src/lib.rs
├── Cargo.toml
└── plugin.wasm
```

Perfect for:
- Business logic
- Data processing
- External integrations

## Plugin Manifest

The manifest is the heart of every plugin. It defines metadata, pages, routes, and permissions.

```json
{
  "name": "my-awesome-plugin",
  "version": "1.0.0",
  "description": "Does awesome things",
  "author": "Your Name",
  "homepage": "https://example.com",
  "license": "MIT",
  
  "min_orbis_version": "1.0.0",
  "dependencies": [],
  "permissions": [],
  
  "routes": [],
  "pages": [],
  
  "wasm_entry": "plugin.wasm",
  "config": {}
}
```

### Manifest Fields

| Field | Required | Description |
|-------|----------|-------------|
| `name` | ✅ | Unique plugin identifier (alphanumeric, hyphens, underscores) |
| `version` | ✅ | Semantic version (e.g., "1.0.0") |
| `description` | ❌ | Human-readable description |
| `author` | ❌ | Plugin author |
| `homepage` | ❌ | Plugin homepage URL |
| `license` | ❌ | License identifier |
| `min_orbis_version` | ❌ | Minimum required Orbis version |
| `dependencies` | ❌ | Other required plugins |
| `permissions` | ❌ | Requested capabilities |
| `routes` | ❌ | API route definitions |
| `pages` | ❌ | UI page definitions |
| `wasm_entry` | ❌ | Path to WASM binary |
| `config` | ❌ | Custom configuration |

## Page Definitions

Pages are the primary way plugins provide UI.

```json
{
  "pages": [
    {
      "id": "dashboard",
      "title": "Dashboard",
      "route": "/my-plugin",
      "icon": "LayoutDashboard",
      "state": {
        "items": {
          "type": "array",
          "default": []
        }
      },
      "layout": {
        "type": "Container",
        "children": [
          {
            "type": "Heading",
            "text": "My Dashboard"
          }
        ]
      },
      "onMount": [],
      "onUnmount": []
    }
  ]
}
```

### Page Fields

| Field | Description |
|-------|-------------|
| `id` | Unique page identifier within the plugin |
| `title` | Display title in navigation |
| `route` | URL path for the page |
| `icon` | Icon name from lucide-react |
| `state` | State definition for the page |
| `layout` | Root component schema |
| `onMount` | Actions to run when page loads |
| `onUnmount` | Actions to run when page unloads |

## Plugin Routes

Plugins can define HTTP-like routes for API functionality:

```json
{
  "routes": [
    {
      "path": "/api/items",
      "method": "GET",
      "handler": "get_items"
    },
    {
      "path": "/api/items",
      "method": "POST",
      "handler": "create_item"
    },
    {
      "path": "/api/items/:id",
      "method": "DELETE",
      "handler": "delete_item"
    }
  ]
}
```

Routes are called from UI via the `callApi` action:

```json
{
  "type": "callApi",
  "api": "my-plugin.get_items",
  "onSuccess": [
    {
      "type": "updateState",
      "path": "items",
      "value": "$response.data"
    }
  ]
}
```

## Permissions

Plugins declare required permissions in the manifest:

```json
{
  "permissions": [
    {
      "type": "network",
      "allowed_hosts": ["api.example.com", "*.myapp.io"]
    },
    {
      "type": "storage",
      "scope": "plugin-data"
    },
    {
      "type": "ipc",
      "allowed_plugins": ["other-plugin"]
    }
  ]
}
```

### Permission Types

| Type | Description |
|------|-------------|
| `network` | HTTP/HTTPS requests to specified hosts |
| `storage` | Database storage access |
| `filesystem` | Local file access (with path restrictions) |
| `ipc` | Inter-plugin communication |
| `notification` | System notifications |

## Plugin Dependencies

Plugins can depend on other plugins:

```json
{
  "dependencies": [
    {
      "name": "auth-plugin",
      "version": ">=1.0.0"
    },
    {
      "name": "data-plugin",
      "version": "^2.0.0"
    }
  ]
}
```

Orbis ensures dependencies are loaded before the dependent plugin.

## Plugin Loading

### Loading Formats

Orbis supports multiple plugin formats:

1. **Unpacked Directory** (development)
   ```
   plugins/my-plugin/
   ├── manifest.json
   └── plugin.wasm
   ```

2. **Packed ZIP** (distribution)
   ```
   plugins/my-plugin.zip
   ```

3. **Standalone WASM** (with embedded manifest)
   ```
   plugins/my-plugin.wasm
   ```

### Loading Order

1. Scan plugin directories
2. Parse manifests
3. Validate dependencies
4. Load in dependency order
5. Initialize WASM runtimes
6. Register routes and pages

### Hot Reload

In development mode, Orbis watches for plugin changes:

```bash
# Plugins in this directory are hot-reloaded
ORBIS_PLUGINS_DIR=./plugins
```

When a plugin file changes:
1. Unload affected plugin
2. Reload manifest
3. Re-initialize if WASM changed
4. Re-render affected pages

## WASM Plugin Development

### Setting Up a WASM Plugin

```bash
cargo new --lib my-plugin
cd my-plugin
```

**Cargo.toml:**

```toml
[package]
name = "my-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### Plugin Entry Points

WASM plugins must export these functions:

```rust
/// Initialize the plugin (called once at load)
#[no_mangle]
pub extern "C" fn init() -> i32 {
    // Return 0 for success, non-zero for error
    0
}

/// Execute plugin logic
#[no_mangle]
pub extern "C" fn execute() -> i32 {
    // Main plugin execution
    0
}

/// Clean up resources (called at unload)
#[no_mangle]
pub extern "C" fn cleanup() -> i32 {
    // Release resources
    0
}
```

### Building WASM

```bash
# Add WASM target
rustup target add wasm32-unknown-unknown

# Build
cargo build --target wasm32-unknown-unknown --release

# Output at: target/wasm32-unknown-unknown/release/my_plugin.wasm
```

### Embedding the Manifest

You can embed the manifest in the WASM binary:

```bash
# Using the provided Python script
cat manifest.json | python3 add_custom_section.py \
  my_plugin.wasm \
  -s manifest \
  -o my_plugin_with_manifest.wasm
```

## Plugin Registry

The plugin registry tracks all loaded plugins:

```rust
// Internal API
registry.register(plugin)?;
registry.get("my-plugin")?;
registry.list_plugins()?;
registry.unload("my-plugin")?;
```

From the frontend, plugins are accessed via:

```typescript
// Get all plugins
const plugins = await invoke('list_plugins');

// Get plugin pages
const pages = await invoke('get_plugin_pages', { pluginId: 'my-plugin' });
```

## Plugin Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Discovered: Scan directory
    Discovered --> Validated: Parse manifest
    Validated --> Loading: Validate deps
    Loading --> Initializing: Load WASM
    Initializing --> Active: Call init()
    Active --> Unloading: Hot reload / shutdown
    Unloading --> [*]: Call cleanup()
    
    Validated --> Error: Invalid manifest
    Loading --> Error: Dependency missing
    Initializing --> Error: init() fails
```

## Best Practices

### Do's

✅ Use semantic versioning for plugins  
✅ Declare all required permissions  
✅ Handle errors gracefully  
✅ Validate input in WASM handlers  
✅ Clean up resources in `cleanup()`  

### Don'ts

❌ Don't request unnecessary permissions  
❌ Don't store secrets in manifest  
❌ Don't block the main thread in handlers  
❌ Don't assume network availability  

## Examples

Check these example plugins:

- **[hello-plugin](https://github.com/cyberpath-HQ/orbis/tree/main/plugins/hello-plugin)** - Simple hello world
- More examples coming soon

## Next Steps

- **[Manifest Reference](../plugin-development/manifest)** - Complete manifest documentation
- **[WASM Plugins](../plugin-development/wasm-plugins)** - In-depth WASM guide
- **[Page Definitions](../plugin-development/page-definitions)** - UI page configuration
