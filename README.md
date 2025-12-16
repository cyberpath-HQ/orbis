# 🚀 Orbis

**NextGen Extensible Asset Management Platform**

Orbis is a modern, enterprise-grade asset management platform designed to provide comprehensive visibility and control over your IT infrastructure. Built with performance, security, and extensibility in mind.

> ⚠️ **IMPORTANT**: This project is **NOT production ready** and is under **active development**. Breaking changes may be applied at any time until production stability is reached. Use at your own risk.

---

## ✨ Features

### 🔧 Core Platform Features

| Feature | Status | Description |
|---------|--------|-------------|
| **Cross-Platform Server** | ✅ Done | High-performance Rust/Axum backend that runs on Windows, Linux, and macOS |
| **React GUI** | ✅ Done | Modern, intuitive web interface with plugin UI rendering |
| **CLI Configuration** | ✅ Done | Full configuration via command line arguments and environment variables |
| **Multi-Database Support** | ✅ Done | PostgreSQL and SQLite backends with automatic migrations |
| **HTTPS/TLS Support** | ✅ Done | Optional TLS encryption with rustls |
| **JSON API** | ✅ Done | RESTful API for communication and integration with existing tools |

### 🔌 Plugin System

| Feature | Status | Description |
|---------|--------|-------------|
| **WASM Plugins** | ✅ Done | Secure, sandboxed WebAssembly plugins with wasmtime |
| **Plugin Routes** | ✅ Done | Plugins can define custom API endpoints |
| **Plugin Pages** | ✅ Done | Plugins can define React pages via JSON UI schema |
| **Plugin Registry** | ✅ Done | Hot-loading/unloading of plugins at runtime |

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

- Rust 1.91.0+ (nightly for Edition 2024)
- Node.js 18+ (bun preferred)
- PostgreSQL 15+ (or SQLite for standalone mode)

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

Orbis supports WASM plugins in three flavors:

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

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

