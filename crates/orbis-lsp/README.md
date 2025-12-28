# Orbis LSP - Language Server for Orbis DSL

A complete Language Server Protocol (LSP) implementation for the Orbis DSL, providing rich IDE features for development.

## Features

- **Syntax Highlighting**: Rich semantic tokens for syntax coloring
- **Completions**: Context-aware autocomplete for components, attributes, events, state variables
- **Hover Documentation**: Detailed documentation on hover for all DSL elements
- **Diagnostics**: Real-time syntax and semantic error reporting with helpful suggestions
- **Go to Definition**: Jump to state declarations, fragments, and imports
- **Find References**: Find all usages of state variables, fragments
- **Document Symbols**: Outline view with state, fragments, hooks, template
- **Semantic Tokens**: Rich highlighting for keywords, components, expressions

## Installation

### VS Code Extension

The simplest way to use the Orbis LSP is through the VS Code extension:

1. Build the LSP server:
   ```bash
   cargo build --release -p orbis-lsp
   ```

2. Install the VS Code extension:
   ```bash
   cd editors/vscode
   npm install
   npm run package
   code --install-extension orbis-lang-*.vsix
   ```

### Manual Installation

You can also use the LSP server directly with any LSP-compatible editor:

```bash
# Build the server
cargo build --release -p orbis-lsp

# The binary will be at:
# target/release/orbis-lsp (Linux/macOS)
# target/release/orbis-lsp.exe (Windows)
```

Configure your editor to use `orbis-lsp` as the language server for `.orbis` files.

## Usage

The LSP server communicates over stdio. Configure your editor with:

- **Command**: `orbis-lsp`
- **Language ID**: `orbis`
- **File Extensions**: `.orbis`

### Neovim (with nvim-lspconfig)

```lua
local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

if not configs.orbis then
  configs.orbis = {
    default_config = {
      cmd = { 'orbis-lsp' },
      filetypes = { 'orbis' },
      root_dir = lspconfig.util.root_pattern('.git', 'Cargo.toml', 'package.json'),
      settings = {},
    },
  }
end

lspconfig.orbis.setup {}
```

### Helix

Add to `~/.config/helix/languages.toml`:

```toml
[[language]]
name = "orbis"
scope = "source.orbis"
injection-regex = "orbis"
file-types = ["orbis"]
roots = ["Cargo.toml", "package.json"]
language-servers = ["orbis-lsp"]

[language-server.orbis-lsp]
command = "orbis-lsp"
```

## Development

### Running Tests

```bash
cargo test -p orbis-lsp
```

### Running with Logging

```bash
RUST_LOG=debug orbis-lsp
```

## Architecture

```
orbis-lsp/
├── src/
│   ├── main.rs           # Entry point, server setup
│   ├── backend.rs        # LSP Backend implementation
│   ├── capabilities.rs   # Server capabilities
│   ├── diagnostics.rs    # Error reporting and suggestions
│   ├── completion.rs     # Autocompletion providers
│   ├── hover.rs          # Hover documentation
│   ├── definition.rs     # Go-to-definition
│   ├── references.rs     # Find references
│   ├── symbols.rs        # Document symbols
│   ├── semantic_tokens.rs # Semantic highlighting
│   ├── document.rs       # Document management
│   └── analysis.rs       # Semantic analysis cache
```

## License

Same as Orbis main project.
