# Plugin API

A secure, high-performance plugin system for Rust applications with comprehensive sandboxing, resource management, and cryptographic verification.

## Overview

The Plugin API provides a complete framework for building, securing, and managing dynamic plugins in Rust. It features:

- **🔒 Enterprise-grade Security**: Ed25519 signatures, SHA3-512 hashing, encrypted trust lists
- **📦 Process Isolation**: Optional sandboxing with Linux namespaces, cgroups, and seccomp
- **⚡ Resource Management**: Configurable limits for CPU, memory, threads, and network connections
- **🔌 Hot-reload Support**: Load and unload plugins at runtime without restarting
- **📊 Real-time Monitoring**: Track resource usage and violations with automatic enforcement
- **🎯 Hook System**: Flexible event-driven architecture for plugin integration
- **🛡️ Memory Isolation**: Per-plugin memory tracking and cleanup guarantees

## Quick Links

- **[Features Documentation](docs/features.md)** - Complete list of implemented functionalities
- **[Security Guide](docs/security.md)** - Comprehensive security measures and best practices
- **[Plugin Development Guide](docs/plugin-development.md)** - How to create, sign, and deploy plugins
- **[Integration Guide](docs/integration.md)** - Integrate the plugin system into your application
- **[API Reference](docs/api-reference.md)** - Detailed API documentation
- **[Examples](../example-plugin/)** - Working example plugin implementation

## Quick Start

### Creating a Plugin

```rust
use plugin_api::{make_plugin, PluginContext, PluginError};

make_plugin!({
    plugin_name: MyPlugin,
    name: "my_plugin",
    version: "1.0.0",
    author: "Your Name",
    
    init(&mut self, context: &mut PluginContext) -> Result<(), PluginError> {
        // Initialize your plugin
        tracing::info!("Plugin initialized!");
        Ok(())
    }
    
    shutdown(&mut self) -> Result<(), PluginError> {
        // Cleanup on unload
        Ok(())
    }
    
    register_hooks(&self, hook_registry: *mut ()) -> Result<(), PluginError> {
        // Register event hooks
        Ok(())
    }
});
```

### Building the Plugin

```bash
# Build the plugin
cargo build --release

# Compute SHA3-512 hash
./scripts/compute-plugin-hashes.sh release

# Generate signing keys
./scripts/sign-plugin.sh generate-key my-key

# Sign the plugin
./scripts/sign-plugin.sh sign target/release/libmy_plugin.so keys/my-key.key
```

### Loading Plugins in Your Application

```rust
use std::sync::Arc;
use plugin_api::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize components
    let context = Arc::new(PluginContext::new());
    let hook_registry = Arc::new(RwLock::new(HookRegistry::new()));
    
    // Configure security
    let hardcoded_public_keys = vec![
        PublicKey::from_hex("your_public_key_here")?,
    ];
    
    let trusted_plugins = vec![
        TrustedPluginEntry {
            hash: "plugin_sha3_512_hash".to_string(),
            version: PluginVersion::new(1, 0, 0),
            signature: PluginSignature::from_hex("signature_here")?,
            note: Some("My Plugin".to_string()),
        },
    ];
    
    let security_policy = SecurityPolicy::default();
    let security = Arc::new(PluginSecurity::new(
        security_policy,
        hardcoded_public_keys,
        trusted_plugins,
    ));
    
    // Create loader and registry
    let loader = Arc::new(PluginLoader::new(security.clone()));
    let registry = Arc::new(PluginRegistry::new(
        loader,
        context,
        hook_registry,
        security,
    ));
    
    // Load a plugin
    registry.load_plugin("path/to/plugin.so", TrustLevel::Trusted).await?;
    
    // Start resource monitoring
    let monitor = registry.clone().start_resource_monitor(None);
    
    // Your application logic here...
    
    // Cleanup
    registry.unload_all().await?;
    monitor.abort();
    
    Ok(())
}
```

## Architecture

The plugin system is built with a layered architecture:

```
┌─────────────────────────────────────────────────────────┐
│                    Host Application                      │
│  ┌────────────┐  ┌──────────────┐  ┌────────────────┐ │
│  │  Registry  │  │   Context    │  │ Hook Registry  │ │
│  └────────────┘  └──────────────┘  └────────────────┘ │
└───────────────────────┬─────────────────────────────────┘
                        │
        ┌───────────────┼───────────────┐
        │               │               │
┌───────▼─────┐ ┌───────▼─────┐ ┌──────▼──────┐
│   Plugin A  │ │   Plugin B  │ │   Plugin C  │
│  (In-proc)  │ │ (Sandboxed) │ │ (In-proc)   │
└─────────────┘ └─────────────┘ └─────────────┘
```

### Core Components

1. **PluginRegistry**: Central management for all plugins
2. **PluginLoader**: Dynamic library loading with security checks
3. **PluginSecurity**: Signature verification and trust management
4. **PluginContext**: Safe data sharing between host and plugins
5. **HookRegistry**: Event-driven communication system
6. **ResourceMonitor**: Real-time resource tracking and enforcement
7. **ProcessManager**: Optional sandboxing with Linux namespaces (Linux only)

## Key Features

### Security Features

- **Cryptographic Verification**: Ed25519 signature validation
- **Hash Verification**: SHA3-512 integrity checks
- **Encrypted Trust Lists**: XChaCha20-Poly1305 encrypted plugin whitelist
- **Public Key Pinning**: Hardcoded trusted public keys
- **Signature Requirements**: All plugins must be signed
- **Trust Level Enforcement**: Only trusted plugins can be loaded

### Isolation Features

- **Memory Isolation**: Per-plugin memory tracking and cleanup
- **Resource Limits**: Configurable CPU, memory, thread, and network limits
- **Process Sandboxing** (Linux): Separate processes with namespace isolation
- **Network Restrictions**: Configurable network access controls
- **Filesystem Restrictions**: Sandboxed filesystem access
- **Capability Dropping**: Minimal privilege execution

### Management Features

- **Hot Reload**: Load/unload plugins without restart
- **Status Tracking**: Real-time plugin status monitoring
- **Violation Tracking**: Automatic violation detection and enforcement
- **Auto-unmount**: Automatic plugin unloading on policy violations
- **Graceful Shutdown**: Proper cleanup on plugin unload
- **Discovery**: Automatic plugin scanning and registration

## Platform Support

| Feature | Linux | Windows | macOS |
|---------|-------|---------|-------|
| Core Plugin System | ✅ | ✅ | ✅ |
| Dynamic Loading | ✅ | ✅ | ✅ |
| Signature Verification | ✅ | ✅ | ✅ |
| Resource Monitoring | ✅ | ⚠️ Limited | ⚠️ Limited |
| Process Sandboxing | ✅ | ❌ | ❌ |
| Namespace Isolation | ✅ | ❌ | ❌ |
| Cgroups | ✅ | ❌ | ❌ |
| Seccomp Filtering | ✅ | ❌ | ❌ |

## Documentation

### Getting Started
- [Features Overview](docs/features.md) - All implemented features
- [Plugin Development Guide](docs/plugin-development.md) - Create your first plugin
- [Integration Guide](docs/integration.md) - Add plugins to your app

### Security
- [Security Guide](docs/security.md) - Security architecture and best practices
- [Signing Process](docs/plugin-development.md#signing-plugins) - How to sign plugins
- [Trust Management](docs/security.md#trust-management) - Managing trusted plugins

### Advanced Topics
- [API Reference](docs/api-reference.md) - Complete API documentation
- [Resource Management](docs/features.md#resource-management) - Configure limits
- [Sandboxing](docs/features.md#sandboxing-linux-only) - Process isolation setup
- [Monitoring](docs/features.md#monitoring-system) - Resource monitoring

## Requirements

### Core Requirements
- Rust 1.70+ (for `async fn` in traits)
- `libloading` for dynamic library loading
- `async-trait` for async plugin trait

### Security Requirements
- `ed25519-dalek` for signature verification
- `sha3` for hash computation
- `chacha20poly1305` for trust list encryption

### Optional Requirements
- Linux kernel 4.8+ (for sandboxing features)
- `procfs` access (for resource monitoring on Linux)
- `cap-std` (for capability-based filesystem access)

## Performance

The plugin system is designed for high performance:

- **Minimal Overhead**: Direct function calls through vtables
- **Zero-copy IPC**: Optional shared memory for sandboxed plugins
- **Lazy Loading**: Plugins loaded only when needed
- **Efficient Monitoring**: Configurable check intervals (default: 10s)
- **Lock-free Reads**: RwLock for concurrent plugin access

## Examples

See the [example-plugin](../example-plugin/) directory for a complete working example, including:

- Plugin implementation using the `make_plugin!` macro
- Database access through shared context
- Hook registration
- Resource limit declaration

## Contributing

Contributions are welcome! Please ensure:

1. All tests pass: `cargo test`
2. Code is formatted: `cargo fmt`
3. No clippy warnings: `cargo clippy`
4. Documentation is updated

## License

See the root LICENSE file for license information.

## Changelog

### Version 0.2.0 (Current)
- Added resource monitoring and enforcement
- Added violation tracking and auto-unmount
- Added process sandboxing for Linux
- Improved security with signature requirements
- Enhanced documentation

### Version 0.1.0
- Initial release
- Core plugin system
- Basic security features
- Hook registry

## Support

For issues, questions, or contributions:
- Open an issue on the repository
- Check existing documentation
- Review example implementations

---

**⚠️ Security Notice**: This plugin system is designed for trusted environments. Always verify plugin sources, use signature verification, and configure appropriate resource limits for your use case.

