---
sidebar_position: 1
title: Plugin Development Overview
description: Getting started with Orbis plugin development
---

# Plugin Development Overview

This guide covers everything you need to know to create plugins for Orbis.

## What Are Plugins?

Plugins are self-contained modules that extend Orbis functionality. They can:

- Add new pages to the application
- Define custom API routes
- Execute backend logic in WASM
- Store and retrieve data
- Communicate with other plugins

## Plugin Types

### 1. Manifest-Only Plugins

The simplest plugin type. Just a manifest file defining UI pages:

```
my-plugin/
└── manifest.json
```

**Best for:**
- Dashboard pages
- Static content
- Simple interactive UIs
- Prototyping

### 2. WASM Plugins

Full-featured plugins with backend execution:

```
my-plugin/
├── manifest.json
├── Cargo.toml
├── src/lib.rs
└── target/wasm32-unknown-unknown/release/my_plugin.wasm
```

**Best for:**
- Complex business logic
- Data processing
- External API integration
- Custom algorithms

## Quick Start

### Minimal Manifest Plugin

Create `plugins/hello/manifest.json`:

```json
{
  "name": "hello",
  "version": "1.0.0",
  "description": "Hello World plugin",
  "pages": [
    {
      "id": "main",
      "title": "Hello World",
      "route": "/hello",
      "icon": "Hand",
      "layout": {
        "type": "Container",
        "className": "p-6",
        "children": [
          {
            "type": "Heading",
            "level": 1,
            "text": "Hello, World!"
          },
          {
            "type": "Text",
            "content": "This is my first Orbis plugin."
          }
        ]
      }
    }
  ]
}
```

That's it! Place it in the `plugins/` directory and restart Orbis.

## Plugin File Structure

### Recommended Structure

```
my-plugin/
├── manifest.json          # Required: plugin metadata and config
├── README.md              # Recommended: documentation
├── CHANGELOG.md           # Recommended: version history
│
├── src/                   # WASM plugin source (if applicable)
│   └── lib.rs
├── Cargo.toml             # WASM plugin dependencies
│
├── assets/                # Static assets (optional)
│   ├── icon.svg
│   └── styles.css
│
└── tests/                 # Plugin tests (optional)
    └── integration.rs
```

## Development Workflow

### 1. Create Plugin Directory

```bash
mkdir -p plugins/my-plugin
cd plugins/my-plugin
```

### 2. Write Manifest

Create `manifest.json` with your plugin configuration.

### 3. Test Iteratively

Run Orbis in development mode for hot reload:

```bash
cd orbis
bun run tauri dev
```

Changes to manifest files are picked up automatically.

### 4. Add WASM (Optional)

If you need backend logic:

```bash
cargo init --lib
# Edit Cargo.toml for WASM target
cargo build --target wasm32-unknown-unknown --release
```

### 5. Build for Distribution

Package your plugin:

```bash
# As a ZIP
zip -r my-plugin.zip manifest.json my_plugin.wasm

# Or with embedded manifest
python3 add_custom_section.py my_plugin.wasm -s manifest < manifest.json
```

## Key Concepts

### Pages

Pages are the UI entry points. Each page has:

- A unique route in the application
- Its own state store
- A layout defined by component schemas

See [Page Definitions](./page-definitions).

### State

Each page has reactive state that drives the UI:

```json
{
  "state": {
    "count": { "type": "number", "default": 0 }
  }
}
```

State is accessed via expressions: `{{state.count}}`

### Actions

Actions respond to user events:

```json
{
  "events": {
    "onClick": [
      { "type": "updateState", "path": "count", "value": "{{state.count + 1}}" }
    ]
  }
}
```

### Components

35+ built-in components for building UIs:

- Layout: Container, Flex, Grid
- Typography: Text, Heading
- Forms: Form, Field
- Data: Table, List, Card
- Navigation: Button, Link, Tabs
- And more...

## Example Plugins

### Counter Plugin

```json
{
  "name": "counter",
  "version": "1.0.0",
  "pages": [
    {
      "id": "main",
      "title": "Counter",
      "route": "/counter",
      "icon": "Hash",
      "state": {
        "count": { "type": "number", "default": 0 }
      },
      "layout": {
        "type": "Flex",
        "direction": "column",
        "align": "center",
        "gap": "1rem",
        "className": "p-6",
        "children": [
          {
            "type": "Heading",
            "level": 1,
            "text": "Count: {{state.count}}"
          },
          {
            "type": "Flex",
            "gap": "0.5rem",
            "children": [
              {
                "type": "Button",
                "label": "-",
                "variant": "outline",
                "events": {
                  "onClick": [
                    { "type": "updateState", "path": "count", "value": "{{state.count - 1}}" }
                  ]
                }
              },
              {
                "type": "Button",
                "label": "+",
                "events": {
                  "onClick": [
                    { "type": "updateState", "path": "count", "value": "{{state.count + 1}}" }
                  ]
                }
              }
            ]
          },
          {
            "type": "Button",
            "label": "Reset",
            "variant": "ghost",
            "events": {
              "onClick": [
                { "type": "updateState", "path": "count", "value": 0 }
              ]
            }
          }
        ]
      }
    }
  ]
}
```

### Todo List Plugin

See the full example in the [repository](https://github.com/cyberpath-HQ/orbis/tree/main/plugins).

## Next Steps

- **[Manifest Reference](./manifest)** - Complete manifest documentation
- **[WASM Plugins](./wasm-plugins)** - Creating plugins with Rust
- **[Page Definitions](./page-definitions)** - Defining UI pages
- **[Building Plugins](./building-plugins)** - Build and distribution
- **[Best Practices](./best-practices)** - Guidelines and patterns
