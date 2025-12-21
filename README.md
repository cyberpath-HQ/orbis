# 🚀 Orbis

**Plugin-Driven Desktop Application Platform**

Orbis is a modern, extensible desktop application platform built with Rust and React. It enables developers to create powerful applications using a declarative JSON-based UI schema system, with WASM-sandboxed plugins for secure extensibility.

### 🎯 Key Highlights

- **🔌 No React Required**: Build UI with JSON schemas - plugins don't ship React code
- **🦀 Tauri Desktop App**: Native performance with Rust backend, React frontend
- **🔒 WASM Sandboxing**: Secure plugin isolation with configurable permissions
- **📊 35+ Components**: Rich component library including forms, tables, charts, and more
- **⚡ Reactive State**: Zustand-powered state management with expression interpolation
- **🎨 shadcn/ui**: Beautiful, accessible components out of the box
- **🔄 Two Modes**: Standalone (SQLite) or Client-Server (PostgreSQL)

---

## ✨ Features

### 🔧 Core Platform Features

| Feature | Status | Description |
|---------|--------|-------------|
| **Cross-Platform Server** | ✅ Done | High-performance Rust/Axum backend that runs on Windows, Linux, and macOS |
| **Schema-Driven UI** | ✅ Done | 35+ UI components rendered from JSON schemas with shadcn/ui |
| **CLI Configuration** | ✅ Done | Full configuration via command line arguments and environment variables |
| **Multi-Database Support** | ✅ Done | PostgreSQL and SQLite backends with automatic migrations |
| **HTTPS/TLS Support** | ✅ Done | Optional TLS encryption with rustls |
| **JSON API** | ✅ Done | RESTful API for communication and integration with existing tools |

### 🔌 Plugin System

| Feature | Status | Description |
|---------|--------|-------------|
| **WASM Plugins** | ✅ Done | Secure, sandboxed WebAssembly plugins with wasmtime |
| **Plugin Routes** | ✅ Done | Plugins can define custom API endpoints |
| **Plugin Pages** | ✅ Done | Plugins define UI via JSON schemas (no React code needed) |
| **Action System** | ✅ Done | 16 action types for interactive behaviors and state management |
| **Plugin Registry** | ✅ Done | Hot-loading/unloading of plugins at runtime |

### 🎨 UI System

| Feature | Status | Description |
|---------|--------|-------------|
| **Component Library** | ✅ Done | 35+ built-in components (Button, Form, Table, Chart, etc.) |
| **State Management** | ✅ Done | Zustand + Immer for reactive state with page-level stores |
| **Expression System** | ✅ Done | Dynamic value interpolation with `{{state.field}}` syntax |
| **Error Boundaries** | ✅ Done | Plugin and page-level error isolation |
| **Lifecycle Hooks** | ✅ Done | `onMount` and `onUnmount` hooks for pages |
| **Toast Notifications** | ✅ Done | Sonner integration for user feedback |

### 🔐 Security

| Feature | Status | Description |
|---------|--------|-------------|
| **JWT Authentication** | ✅ Done | Secure token-based authentication |
| **Argon2 Password Hashing** | ✅ Done | Industry-standard password security |
| **Session Management** | ✅ Done | Secure session handling with refresh tokens |
| **WASM Sandboxing** | ✅ Done | Plugins run in secure sandboxed environment |

### 🔄 Modes

| Mode | Description |
|------|-------------|
| **Standalone** | Local database with embedded server (single user) |
| **Client-Server** | Connect to remote Orbis server (multi-user) |


## 🏗️ Architecture Overview

Orbis uses a **schema-driven architecture** where plugins define UI through JSON instead of code:

```mermaid
flowchart LR
    A["Plugin (WASM)"] --> B["JSON Schema"] --> C["SchemaRenderer (React)"] --> D["UI Components"]
```

**Key architectural decisions:**

1. **No React in Plugins**: Plugins ship JSON schemas, not React code - the core `SchemaRenderer` interprets schemas and renders shadcn/ui components
2. **WASM Sandboxing**: All plugins run in isolated wasmtime environments with configurable permissions
3. **Action-Based Interactivity**: Instead of callbacks, plugins use action definitions (`updateState`, `callApi`, etc.) that the core executes
4. **Page-Level State**: Each plugin page has its own Zustand store (no global state pollution)
5. **Expression Interpolation**: Dynamic values use `{{state.field}}` syntax for reactive updates

**Communication flow:**

```mermaid
flowchart TD
    UI["User Interaction"]
    EV["Event"]
    AE["Action Executor"]
    SU["State Update"]
    RR["Re-render"]
    TC["Tauri Commands"]
    RB["Rust Backend"]
    DB["Database"]
    PR["Plugin Runtime (WASM)"]

    UI --> EV --> AE --> SU --> RR
    EV --> TC
    TC <--> RB
    RB --> DB
    TC --> PR
```

## 📦 Crate Structure

```
crates/
├── orbis-core/        # Shared types, errors, and utilities
├── orbis-config/      # CLI and environment configuration
├── orbis-db/          # Database layer (SQLx, migrations)
├── orbis-auth/        # Authentication (JWT, Argon2, sessions)
├── orbis-plugin/      # Plugin system (WASM, manifest, UI schema)
└── orbis-server/      # Axum HTTP/HTTPS server

orbis/
├── src/               # React frontend
└── src-tauri/         # Tauri desktop application
```

## 🚀 Quick Start

### Prerequisites

- **Rust** 1.91.0+ (nightly for Edition 2024)
- **Node.js** 18+ (bun recommended)
- **PostgreSQL** 15+ (optional, for client-server mode)

### Development Setup

```bash
# Clone the repository
git clone https://github.com/cyberpath-HQ/orbis
cd orbis

# Install frontend dependencies
cd orbis && bun install

# Run the Tauri desktop app in dev mode
bun run tauri dev
```

The app will start with hot reload enabled for both frontend and backend changes.

### Build for Production

```bash
cd orbis
bun run tauri build
```

Outputs platform-specific installers in `src-tauri/target/release/bundle/`.

### Configuration

All configuration can be set via CLI arguments or environment variables:

```bash
# Run in standalone mode with SQLite
ORBIS_MODE=standalone \
ORBIS_DATABASE_BACKEND=sqlite \
ORBIS_DATABASE_PATH=./orbis.db \
cargo run

# Run as server with PostgreSQL
ORBIS_MODE=client-server \
ORBIS_RUN_MODE=server \
ORBIS_DATABASE_URL=postgres://user:pass@localhost/orbis \
ORBIS_JWT_SECRET=your-secret-key \
ORBIS_SERVER_HOST=0.0.0.0 \
ORBIS_SERVER_PORT=8080 \
cargo run

# Enable HTTPS
ORBIS_TLS_ENABLED=true \
ORBIS_TLS_CERT_PATH=./cert.pem \
ORBIS_TLS_KEY_PATH=./key.pem \
cargo run
```

### CLI Options

```bash
orbis --help

# Examples:
orbis serve --mode standalone --db-backend sqlite --db-path ./data.db
orbis serve --mode client-server --run-mode server --db-url postgres://...
orbis profile list
orbis profile switch production
orbis db migrate
orbis plugin list
```

## 🔌 Plugin Development

Plugins in Orbis are WASM modules that define their UI through **JSON schemas** instead of shipping React code. This enables secure sandboxing while providing a rich component library.

### Quick Example

A simple plugin page definition:

```json
{
  "name": "hello-plugin",
  "version": "1.0.0",
  "pages": [
    {
      "route": "/hello",
      "title": "Hello Page",
      "show_in_menu": true,
      "state": {
        "count": { "type": "number", "default": 0 }
      },
      "sections": [
        {
          "type": "Container",
          "children": [
            {
              "type": "Heading",
              "level": 1,
              "text": "Counter: {{state.count}}"
            },
            {
              "type": "Button",
              "label": "Increment",
              "events": {
                "on_click": [
                  {
                    "type": "update_state",
                    "path": "count",
                    "value": "{{state.count + 1}}"
                  }
                ]
              }
            }
          ]
        }
      ]
    }
  ]
}
```

### Building Plugins

```bash
# Navigate to your plugin directory
cd plugins/my-plugin

# Build the WASM module
./build.sh

# Plugin is ready to load in Orbis
```

### Plugin Flavors

1. **Packed** (`.zip` archive)
   - Contains WASM file, `manifest.json`, and assets
   - Manifest can be external or embedded in WASM
   - Best for plugins with UI assets (images, styles, etc.)

2. **Unpacked** (folder)
   - Directory containing WASM file, `manifest.json`, and assets
   - Manifest can be external or embedded in WASM
   - Best for development and testing

3. **Standalone** (single `.wasm` file)
   - Manifest must be embedded in WASM custom section
   - No external files, completely self-contained
   - Best for simple plugins without assets

### Available Components

Plugins have access to 35+ built-in components:

- **Layout**: Container, Flex, Grid, Tabs, Accordion
- **Forms**: Input, Select, Checkbox, Radio, DatePicker, FileUpload
- **Data**: Table, DataGrid, Tree, Timeline
- **Charts**: LineChart, BarChart, PieChart, AreaChart
- **Display**: Text, Heading, Badge, Avatar, Separator
- **Interactive**: Button, Link, Modal, Drawer, Tooltip
- **Feedback**: Alert, Toast, Progress, Skeleton
- **And more...**

### Action Types

16 action types enable rich interactivity:

- `updateState`, `navigate`, `callApi`, `showToast`
- `openModal`, `closeModal`, `submitForm`, `resetForm`
- `emit`, `log`, `conditional`, `sequence`
- `parallel`, `debounce`, `throttle`, `custom`

### Plugin Manifest

External `manifest.json`:

```json
{
  "name": "my-plugin",
  "version": "1.0.0",
  "description": "My awesome plugin",
  "author": "Your Name",
  "wasm_entry": "plugin.wasm",
  "permissions": ["network", "database_read"],
  "routes": [
    {
      "path": "/my-endpoint",
      "method": "GET",
      "handler": "handle_my_endpoint",
      "requires_auth": true
    }
  ],
  "pages": [
    {
      "route": "/my-page",
      "title": "My Page",
      "show_in_menu": true,
      "layout": {
        "type": "Container",
        "children": [
          {
            "type": "Heading",
            "level": 1,
            "text": "Welcome to My Plugin"
          }
        ]
      }
    }
  ]
}
```

### Embedding Manifest in WASM

For standalone plugins or to eliminate external manifest files, you can embed the manifest in a WASM custom section:

```rust
// In your Rust WASM plugin project
const MANIFEST: &str = r#"{
  "name": "my-plugin",
  "version": "1.0.0",
  "description": "Standalone plugin with embedded manifest"
}"#;

// When building with wasm-pack or cargo, add this to your Cargo.toml:
// [package.metadata.wasm-pack.profile.release]
// wasm-opt = ['-O4', '--strip-debug']

// Then use wasm-tools to add custom section:
// wasm-tools custom plugin.wasm manifest manifest.json -o plugin.wasm
```

Or use a build script to embed the manifest automatically:

```bash
# After building your WASM module:
wasm-tools custom plugin.wasm manifest manifest.json -o plugin.wasm
```

The manifest will be stored in a WASM custom section named `"manifest"` and automatically extracted by Orbis when loading the plugin.

### Plugin Structure Examples

**Unpacked Plugin:**

```text
plugins/
  my-plugin/
    manifest.json      # Plugin metadata
    plugin.wasm        # WASM binary
    icon.png          # Optional assets
    styles.css
```

**Packed Plugin:**

```text
plugins/
  my-plugin.zip
    ├── manifest.json  # Or embedded in WASM
    ├── plugin.wasm
    └── assets/
        ├── icon.png
        └── styles.css
```

**Standalone Plugin:**

```text
plugins/
  my-plugin.wasm     # Single file with embedded manifest
```

## 📋 Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `ORBIS_MODE` | `standalone` | `standalone` or `client-server` |
| `ORBIS_RUN_MODE` | `client` | `server` or `client` (for client-server mode) |
| `ORBIS_SERVER_HOST` | `127.0.0.1` | Server bind address |
| `ORBIS_SERVER_PORT` | `8080` | Server port |
| `ORBIS_DATABASE_BACKEND` | `postgres` | `postgres` or `sqlite` |
| `ORBIS_DATABASE_URL` | - | PostgreSQL connection URL |
| `ORBIS_DATABASE_PATH` | - | SQLite database file path |
| `ORBIS_DATABASE_RUN_MIGRATIONS` | `true` | Auto-run migrations on startup |
| `ORBIS_JWT_SECRET` | - | JWT signing secret (required for client-server) |
| `ORBIS_JWT_EXPIRY_SECONDS` | `3600` | JWT token expiry |
| `ORBIS_TLS_ENABLED` | `false` | Enable HTTPS |
| `ORBIS_TLS_CERT_PATH` | - | TLS certificate path |
| `ORBIS_TLS_KEY_PATH` | - | TLS private key path |
| `ORBIS_PLUGINS_DIR` | `./plugins` | Plugins directory |
| `ORBIS_LOG_LEVEL` | `info` | Log level (trace, debug, info, warn, error) |
| `ORBIS_LOG_JSON` | `false` | Output logs as JSON |

## 🛠️ Development

```bash
# Install dependencies
cd orbis && bun install

# Run development server
bun run tauri dev

# Build for production
bun run tauri build
```

## � Documentation

For comprehensive guides, API reference, and tutorials, visit the [full documentation](./docs).

**Key sections:**
- [Getting Started](./docs/src/docs/getting-started/) - Installation and quickstart guides
- [Core Concepts](./docs/src/docs/core-concepts/) - Architecture and plugin system
- [Components](./docs/src/docs/components/) - Complete component library reference
- [Actions](./docs/src/docs/actions/) - Action types and event handling
- [API Reference](./docs/src/docs/api-reference/) - Complete API documentation

## 🤝 Community

- 🐛 [Report Issues](https://github.com/cyberpath-HQ/orbis/issues)
- 📖 [Contributing Guide](https://github.com/cyberpath-HQ/orbis/blob/main/CONTRIBUTING.md)
- ⭐ [Star us on GitHub](https://github.com/cyberpath-HQ/orbis)

## �📄 License

MIT License - see [LICENSE](LICENSE) for details.

