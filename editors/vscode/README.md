# Orbis DSL VS Code Extension

Language support for [Orbis DSL](https://github.com/ebalo/orbis-assets) - a domain-specific language for defining UI pages.

## Features

- **Syntax Highlighting**: Rich TextMate grammar for `.orbis` files
- **Semantic Highlighting**: LSP-powered highlighting based on semantic analysis
- **IntelliSense**: Context-aware completions for:
  - Keywords (page, state, hooks, template, fragment, interface, styles)
  - All 32 built-in components with attributes and events
  - Control flow (if, for, when)
  - State variables and expressions
  - Actions (api, toast, router, console)
- **Hover Documentation**: Detailed docs for components, keywords, and symbols
- **Go to Definition**: Navigate to state variables, fragments, and interfaces
- **Find References**: Locate all usages of symbols
- **Rename**: Safely rename symbols across the file
- **Diagnostics**: Real-time error detection with suggestions
- **Document Outline**: See page structure in the outline view
- **Folding**: Collapse code blocks
- **Code Actions**: Quick fixes for common issues
- **Snippets**: Pre-built templates for common patterns

## Installation

### From Marketplace (Recommended)

1. Open VS Code
2. Go to Extensions (Ctrl+Shift+X)
3. Search for "Orbis DSL"
4. Click Install

### From VSIX

1. Download the `.vsix` file from [releases](https://github.com/ebalo/orbis-assets/releases)
2. Run `code --install-extension orbis-dsl-*.vsix`

### For Development

```bash
# Clone the repository
git clone https://github.com/ebalo/orbis-assets.git
cd orbis-assets/editors/vscode

# Install dependencies
npm install

# Compile
npm run compile

# Open in VS Code
code .

# Press F5 to launch Extension Development Host
```

## Requirements

### Language Server (orbis-lsp)

The extension requires the Orbis Language Server for full functionality. Install it via:

```bash
# From crates.io (when published)
cargo install orbis-lsp

# Or build from source
cd orbis-assets
cargo build --release -p orbis-lsp
```

If `orbis-lsp` is not found in your PATH, configure the path in settings:

```json
{
    "orbis.lsp.path": "/path/to/orbis-lsp"
}
```

## Configuration

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `orbis.lsp.path` | string | `""` | Path to orbis-lsp executable |
| `orbis.lsp.args` | array | `[]` | Additional arguments for LSP |
| `orbis.trace.server` | enum | `"off"` | LSP communication trace level |
| `orbis.validation.enabled` | boolean | `true` | Enable validation |
| `orbis.completion.enabled` | boolean | `true` | Enable completions |
| `orbis.hover.enabled` | boolean | `true` | Enable hover documentation |

## Snippets

| Prefix | Description |
|--------|-------------|
| `page` | Page block with metadata |
| `state` | State block |
| `hooks` | Hooks block with @mount |
| `template` | Template block |
| `fragment` | Fragment definition |
| `interface` | Interface definition |
| `if`, `ifelse` | Conditionals |
| `for`, `fori` | Loops |
| `when` | Pattern matching |
| `Button`, `Field`, `Card`, etc. | Component snippets |
| `orbis-page` | Complete page template |
| `orbis-fetch` | Data fetching pattern |
| `orbis-form` | Form with validation |

## Commands

| Command | Description |
|---------|-------------|
| `Orbis: Restart Language Server` | Restart the LSP server |
| `Orbis: Show Output Channel` | View LSP logs |

## Troubleshooting

### Language Server Not Starting

1. Check that `orbis-lsp` is installed: `which orbis-lsp`
2. Verify the path in settings if using a custom location
3. Check the output channel: `View → Output → Orbis LSP`

### No Completions or Diagnostics

1. Ensure the file has `.orbis` extension
2. Check the LSP is running (look for "Orbis Language Server started" in output)
3. Try restarting the server: `Orbis: Restart Language Server`

### Performance Issues

1. For large files, try disabling semantic highlighting
2. Increase trace level to debug: `"orbis.trace.server": "verbose"`

## Contributing

Contributions are welcome! Please see the [main repository](https://github.com/ebalo/orbis-assets) for guidelines.

## License

MIT License - see [LICENSE](../../LICENSE) for details.
