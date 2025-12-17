---
sidebar_position: 1
title: Installation
description: Set up your Orbis development environment
---

# Installation

This guide will help you set up your development environment for building Orbis applications and plugins.

## Prerequisites

Before you begin, ensure you have the following installed:

### Required Tools

| Tool | Version | Purpose |
|------|---------|---------|
| **Rust** | 1.94.0 nightly or newer | Backend development |
| **Node.js** | 20+ | Frontend tooling |
| **Bun** | 1.0+ | Package manager (recommended) |
| **Git** | Latest | Version control |

### Platform-Specific Requirements

<Tabs>
<TabItem value="linux" label="Linux">

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install -y build-essential libwebkit2gtk-4.1-dev \
  libssl-dev libgtk-3-dev libayatana-appindicator3-dev \
  librsvg2-dev curl wget

# Fedora
sudo dnf install webkit2gtk4.1-devel openssl-devel gtk3-devel \
  libappindicator-gtk3-devel librsvg2-devel

# Arch Linux
sudo pacman -S webkit2gtk-4.1 openssl gtk3 libappindicator-gtk3 librsvg
```

</TabItem>
<TabItem value="macos" label="macOS">

```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install Homebrew dependencies
brew install openssl
```

</TabItem>
<TabItem value="windows" label="Windows">

1. Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
2. Select "Desktop development with C++"
3. Install [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)

</TabItem>
</Tabs>

## Install Rust

If you don't have Rust installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Add the WASM target for plugin development:

```bash
rustup target add wasm32-unknown-unknown
```

## Install Node.js and Bun

We recommend using Bun for faster package management:

```bash
# Install Bun
curl -fsSL https://bun.sh/install | bash

# Or use npm if preferred
npm install -g bun
```

## Clone the Repository

```bash
git clone https://github.com/cyberpath-HQ/orbis.git
cd orbis
```

## Install Dependencies

### Backend Dependencies

```bash
# Install Rust dependencies (handled by Cargo automatically)
cargo check --workspace
```

### Frontend Dependencies

```bash
cd orbis
bun install
```

## Verify Installation

Run the following commands to verify everything is set up correctly:

```bash
# Check Rust toolchain
rustc --version
cargo --version

# Check WASM target
rustup target list | grep wasm32

# Check Node.js and Bun
node --version
bun --version

# Build the project
cd orbis
bun run tauri build --debug
```

## Development Setup

### Environment Variables

Create a `.env` file in the project root:

```bash
# Database configuration (optional, defaults to SQLite)
ORBIS_DATABASE_URL=sqlite://./orbis.db

# Logging
RUST_LOG=info,orbis=debug

# Plugin directory
ORBIS_PLUGINS_DIR=./plugins

# Server mode (standalone or client-server)
ORBIS_MODE=standalone
```

### IDE Setup

#### VS Code (Recommended)

Install the recommended extensions:

- **rust-analyzer** - Rust language support
- **Tauri** - Tauri development tools
- **ESLint** - JavaScript/TypeScript linting

#### JetBrains IDEs

- Install the Rust plugin
- Enable TypeScript support

## Next Steps

You're all set! Continue to:

- **[Quickstart](./quickstart)** - Build your first plugin
- **[Project Structure](./project-structure)** - Understand the codebase

## Troubleshooting

### Common Issues

#### "cannot find crate for `core`"

This usually means the WASM target isn't installed:

```bash
rustup target add wasm32-unknown-unknown
```

#### Webkit2gtk not found (Linux)

Install the GTK WebKit bindings:

```bash
# Ubuntu/Debian
sudo apt install libwebkit2gtk-4.1-dev
```

#### Tauri build fails on macOS

Ensure Xcode Command Line Tools are installed:

```bash
xcode-select --install
```

For more help, check our [GitHub Issues](https://github.com/cyberpath-HQ/orbis/issues) or join our [Discord](https://discord.gg/orbis).
