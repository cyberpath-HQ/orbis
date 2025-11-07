# Plugin Development Guide

Complete guide to creating, signing, and deploying plugins for the Plugin API system.

## Table of Contents

- [Getting Started](#getting-started)
- [Creating a Plugin](#creating-a-plugin)
- [Plugin Manifest](#plugin-manifest)
- [Building Plugins](#building-plugins)
- [Signing Plugins](#signing-plugins)
- [Testing Plugins](#testing-plugins)
- [Deployment](#deployment)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)

---

## Getting Started

### Prerequisites

- Rust 1.70 or later
- Cargo package manager
- Linux, Windows, or macOS
- `plugin-api` dependency

### Project Setup

1. **Create a new library project**:
```bash
cargo new --lib my-plugin
cd my-plugin
```

2. **Configure `Cargo.toml`**:
```toml
[package]
name = "my-plugin"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]  # Important: must be cdylib for dynamic loading

[dependencies]
plugin-api = { path = "../plugin-api" }
async-trait = "0.1"
tracing = "0.1"

# Optional dependencies
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

3. **Create `src/lib.rs`**:
```rust
use plugin_api::{make_plugin, PluginContext, PluginError};

make_plugin!({
    plugin_name: MyPlugin,
    name: "my_plugin",
    version: "1.0.0",
    author: "Your Name",
    
    init(&mut self, context: &mut PluginContext) -> Result<(), PluginError> {
        tracing::info!("MyPlugin initialized!");
        Ok(())
    }
    
    shutdown(&mut self) -> Result<(), PluginError> {
        tracing::info!("MyPlugin shutting down!");
        Ok(())
    }
    
    register_hooks(&self, hook_registry: *mut ()) -> Result<(), PluginError> {
        tracing::info!("MyPlugin hooks registered!");
        Ok(())
    }
});
```

---

## Creating a Plugin

### Using the `make_plugin!` Macro

The `make_plugin!` macro is the recommended way to create plugins. It handles all boilerplate and ensures compatibility.

#### Basic Plugin

```rust
use plugin_api::{make_plugin, PluginContext, PluginError};

make_plugin!({
    plugin_name: MyPlugin,  // Struct name
    name: "my_plugin",      // Plugin identifier (must be unique)
    version: "1.0.0",       // Semantic version
    author: "Your Name",    // Author information
    
    init(&mut self, context: &mut PluginContext) -> Result<(), PluginError> {
        // Initialization code
        Ok(())
    }
    
    shutdown(&mut self) -> Result<(), PluginError> {
        // Cleanup code
        Ok(())
    }
    
    register_hooks(&self, hook_registry: *mut ()) -> Result<(), PluginError> {
        // Hook registration
        Ok(())
    }
});
```

#### Plugin with Description

```rust
make_plugin!({
    plugin_name: DocumentationPlugin,
    name: "documentation_plugin",
    version: "1.0.0",
    author: "Your Name",
    description: "
# Documentation Plugin

This plugin provides **documentation** features.

## Features
- Feature 1
- Feature 2

## Usage
See the main documentation for usage examples.
",
    
    init(&mut self, context: &mut PluginContext) -> Result<(), PluginError> {
        Ok(())
    }
    
    shutdown(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
    
    register_hooks(&self, hook_registry: *mut ()) -> Result<(), PluginError> {
        Ok(())
    }
});
```

#### Plugin with Resource Limits

```rust
use plugin_api::{make_plugin, PluginContext, PluginError, ResourceLimits};

make_plugin!({
    plugin_name: ResourceAwarePlugin,
    name: "resource_aware_plugin",
    version: "1.0.0",
    author: "Your Name",
    
    // Declare resource limits
    resource_limits() -> ResourceLimits {
        ResourceLimits {
            max_heap_bytes: 100 * 1024 * 1024,  // 100 MB
            max_cpu_time_ms: 5000,               // 5 seconds
            max_threads: 8,                      // 8 threads
            max_file_descriptors: 50,            // 50 FDs
            max_connections: 20,                 // 20 connections
        }
    }
    
    init(&mut self, context: &mut PluginContext) -> Result<(), PluginError> {
        Ok(())
    }
    
    shutdown(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
    
    register_hooks(&self, hook_registry: *mut ()) -> Result<(), PluginError> {
        Ok(())
    }
});
```

#### Plugin with Requirements

```rust
use plugin_api::{
    make_plugin, PluginContext, PluginError, PluginRequirements,
    NetworkRequirement, NetworkTarget, FilesystemRequirement,
};

make_plugin!({
    plugin_name: NetworkPlugin,
    name: "network_plugin",
    version: "1.0.0",
    author: "Your Name",
    
    // Declare requirements
    requirements() -> PluginRequirements {
        PluginRequirements {
            network: NetworkRequirement::Restricted {
                allowed_targets: vec![
                    NetworkTarget::Domain("api.example.com".to_string()),
                    NetworkTarget::IpAddress("192.168.1.100".to_string()),
                    NetworkTarget::IpRange {
                        start: "10.0.0.0".to_string(),
                        end: "10.0.255.255".to_string(),
                    },
                ],
            },
            filesystem: FilesystemRequirement::ReadOnly {
                allowed_paths: vec![
                    "/data/readonly".into(),
                    "/config".into(),
                ],
            },
            ..Default::default()
        }
    }
    
    init(&mut self, context: &mut PluginContext) -> Result<(), PluginError> {
        Ok(())
    }
    
    shutdown(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
    
    register_hooks(&self, hook_registry: *mut ()) -> Result<(), PluginError> {
        Ok(())
    }
});
```

### Accessing Context

The plugin context provides access to shared resources:

```rust
use plugin_api::{ContextKey, PredefinedContextKey};
use sea_orm::DatabaseConnection;

init(&mut self, context: &mut PluginContext) -> Result<(), PluginError> {
    // Access database connection
    let db = context.get::<DatabaseConnection>(
        ContextKey::Predefined(PredefinedContextKey::DatabaseConnection)
    )?;
    
    // Use the database
    if db.ping().await.is_ok() {
        tracing::info!("Database connection successful!");
    }
    
    Ok(())
}
```

**Predefined Context Keys**:
- `DatabaseConnection` - Database connection pool
- `Configuration` - Application configuration
- `Logger` - Logging instance
- `HttpClient` - HTTP client
- `CacheManager` - Cache manager

**Custom Context Keys**:
```rust
let key = ContextKey::Custom("my_custom_key".to_string());
let value = context.get::<MyType>(key)?;
```

### Registering Hooks

Hooks allow plugins to respond to events:

```rust
use plugin_api::HookRegistry;

register_hooks(&self, hook_registry: *mut ()) -> Result<(), PluginError> {
    // Cast the raw pointer to HookRegistry
    let registry = unsafe { &mut *(hook_registry as *mut HookRegistry) };
    
    // Register a hook with priority
    registry.register(
        "on_request",
        200,  // Priority (200 = normal)
        Box::new(|data: Vec<u8>| {
            Box::pin(async move {
                // Process the hook
                tracing::info!("Request hook called!");
                
                // Return modified data
                Ok(data)
            })
        }),
    );
    
    Ok(())
}
```

**Hook Priorities**:
- 0-99: Critical (system-level)
- 100-199: High priority
- 200-299: Normal priority (default)
- 300-399: Low priority
- 400+: Cleanup

### Storing Plugin State

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

make_plugin!({
    plugin_name: StatefulPlugin,
    name: "stateful_plugin",
    version: "1.0.0",
    author: "Your Name",
    
    // Add state field
    state: {
        counter: Arc<RwLock<u64>>,
        config: MyConfig,
    }
    
    init(&mut self, context: &mut PluginContext) -> Result<(), PluginError> {
        // Initialize state
        self.counter = Arc::new(RwLock::new(0));
        self.config = MyConfig::load()?;
        Ok(())
    }
    
    shutdown(&mut self) -> Result<(), PluginError> {
        // Save state
        self.config.save()?;
        Ok(())
    }
    
    register_hooks(&self, hook_registry: *mut ()) -> Result<(), PluginError> {
        let counter = self.counter.clone();
        
        let registry = unsafe { &mut *(hook_registry as *mut HookRegistry) };
        registry.register(
            "on_request",
            200,
            Box::new(move |data: Vec<u8>| {
                let counter = counter.clone();
                Box::pin(async move {
                    let mut count = counter.write().await;
                    *count += 1;
                    tracing::info!("Request count: {}", *count);
                    Ok(data)
                })
            }),
        );
        
        Ok(())
    }
});
```

---

## Plugin Manifest

While not a separate file, plugin metadata is defined in the `make_plugin!` macro.

### Required Fields

- `plugin_name`: Rust struct name (CamelCase)
- `name`: Plugin identifier (snake_case, unique)
- `version`: Semantic version (e.g., "1.0.0")
- `author`: Author name or organization
- `init`: Initialization function
- `shutdown`: Shutdown function
- `register_hooks`: Hook registration function

### Optional Fields

- `description`: Markdown description
- `resource_limits`: Resource limit declaration
- `requirements`: Plugin requirements (network, filesystem, etc.)
- `state`: Plugin state fields

### Version Format

Use semantic versioning (MAJOR.MINOR.PATCH):

- **MAJOR**: Incompatible API changes
- **MINOR**: Backward-compatible functionality
- **PATCH**: Backward-compatible bug fixes

Examples:
- `1.0.0` - Initial release
- `1.0.1` - Bug fix
- `1.1.0` - New feature
- `2.0.0` - Breaking change

---

## Building Plugins

### Debug Build

```bash
cargo build
```

Output: `target/debug/libmy_plugin.so` (Linux) / `.dll` (Windows) / `.dylib` (macOS)

### Release Build

```bash
cargo build --release
```

Output: `target/release/libmy_plugin.so`

**Always use release builds for production!**

### Build Options

```bash
# Optimize for size
cargo build --release --config 'profile.release.opt-level="z"'

# Link-time optimization
cargo build --release --config 'profile.release.lto=true'

# Strip symbols
cargo build --release --config 'profile.release.strip=true'
```

### Cross-compilation

```bash
# Add target
rustup target add x86_64-unknown-linux-gnu

# Build for target
cargo build --release --target x86_64-unknown-linux-gnu
```

---

## Signing Plugins

### 1. Generate Signing Keys

**First time only**:

```bash
# Generate a key pair
./scripts/sign-plugin.sh generate-key my-plugin-key

# Output:
# - keys/my-plugin-key.key (private key - KEEP SECRET!)
# - keys/my-plugin-key.pub (public key - share this)
```

**Store private key securely**:
- Use a password manager
- Store on air-gapped machine
- Use a hardware security module (HSM)
- Never commit to version control!

Add `keys/*.key` to `.gitignore`:
```bash
echo "keys/*.key" >> .gitignore
```

### 2. Compute Plugin Hash

```bash
# Compute SHA3-512 hash
./scripts/compute-plugin-hashes.sh release
```

Output:
```
// Auto-generated trusted plugin entries
// Generated at: Mon Nov  7 10:30:00 UTC 2025
// Profile: release
// Using: SHA3-512 hashing

let hardcoded_trusted_plugins = vec![
    TrustedPluginEntry {
        hash: "a1b2c3d4e5f6...".to_string(),
        version: PluginVersion::new(1, 0, 0),
        note: Some("libmy_plugin.so".to_string()),
    },
];
```

**Save this hash** - you'll need it for the trust list.

### 3. Sign the Plugin

```bash
# Sign plugin with your private key
./scripts/sign-plugin.sh sign target/release/libmy_plugin.so keys/my-plugin-key.key

# Output:
# ✓ Plugin signed successfully
```

This embeds the signature in the plugin file.

### 4. Verify Signature

```bash
# Verify with public key
./scripts/sign-plugin.sh verify target/release/libmy_plugin.so $(cat keys/my-plugin-key.pub)

# Output:
# ✓ Signature valid
```

### 5. Add to Trust List

In your application code:

```rust
use plugin_api::{TrustedPluginEntry, PluginVersion, PluginSignature};

let trusted_plugins = vec![
    TrustedPluginEntry {
        hash: "a1b2c3d4e5f6...".to_string(),  // From step 2
        version: PluginVersion::new(1, 0, 0),
        signature: PluginSignature::from_hex("1234567890ab...")?,  // From plugin
        note: Some("My Plugin v1.0.0".to_string()),
    },
];
```

### Manual Signing (without scripts)

```bash
# 1. Generate key pair
cargo run --bin keygen -- keys/my-key.key keys/my-key.pub "My Key"

# 2. Compute hash
sha3sum -a 512 target/release/libmy_plugin.so

# 3. Sign plugin
cargo run --bin plugin-signer -- sign target/release/libmy_plugin.so keys/my-key.key

# 4. Verify signature
cargo run --bin plugin-signer -- verify target/release/libmy_plugin.so $(cat keys/my-key.pub)
```

---

## Testing Plugins

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_plugin_creation() {
        let plugin = MyPlugin::new();
        assert_eq!(plugin.name(), "my_plugin");
        assert_eq!(plugin.version(), "1.0.0");
    }
    
    #[tokio::test]
    async fn test_plugin_init() {
        let mut plugin = MyPlugin::new();
        let mut context = PluginContext::new();
        
        let result = plugin.init(&mut context).await;
        assert!(result.is_ok());
    }
}
```

### Integration Tests

```rust
// tests/integration_test.rs
use plugin_api::*;
use std::sync::Arc;

#[tokio::test]
async fn test_plugin_load() {
    let security = Arc::new(PluginSecurity::new(
        SecurityPolicy::default(),
        vec![],
        vec![],
    ));
    
    let loader = Arc::new(PluginLoader::new(security.clone()));
    let context = Arc::new(PluginContext::new());
    let hook_registry = Arc::new(RwLock::new(HookRegistry::new()));
    
    let registry = PluginRegistry::new(
        loader,
        context,
        hook_registry,
        security,
    );
    
    // Try loading the plugin
    let result = registry.load_plugin(
        "target/debug/libmy_plugin.so",
        TrustLevel::Trusted,
    ).await;
    
    assert!(result.is_ok());
}
```

### Manual Testing

```bash
# Build plugin
cargo build --release

# Run test script
./scripts/test-plugin-system.sh target/release/libmy_plugin.so
```

---

## Deployment

### 1. Prepare Plugin

```bash
# Build release version
cargo build --release

# Sign plugin
./scripts/sign-plugin.sh sign target/release/libmy_plugin.so keys/my-key.key

# Compute hash
HASH=$(sha3sum -a 512 target/release/libmy_plugin.so | awk '{print $1}')
echo "Plugin hash: $HASH"
```

### 2. Update Trust List

```rust
// In your application
let trusted_plugins = vec![
    TrustedPluginEntry {
        hash: "your_plugin_hash_here".to_string(),
        version: PluginVersion::new(1, 0, 0),
        signature: PluginSignature::from_hex("signature_here")?,
        note: Some("My Plugin v1.0.0".to_string()),
    },
];

let security = Arc::new(PluginSecurity::new(
    SecurityPolicy::default(),
    hardcoded_public_keys,
    trusted_plugins,
));

// Encrypt and save trust list
security.encrypt_and_save_trust_list(&encryption_key)?;
```

### 3. Deploy Plugin File

```bash
# Copy plugin to plugin directory
sudo cp target/release/libmy_plugin.so /opt/myapp/plugins/

# Set permissions (read-only)
sudo chmod 644 /opt/myapp/plugins/libmy_plugin.so
sudo chown root:root /opt/myapp/plugins/libmy_plugin.so
```

### 4. Deploy Application

```bash
# Deploy application with updated trust list
sudo systemctl restart myapp
```

### 5. Verify Deployment

```bash
# Check logs
sudo journalctl -u myapp -f

# Look for:
# "Successfully loaded plugin: my_plugin"
```

---

## Best Practices

### Security

1. **Never commit private keys** to version control
2. **Sign all plugins** before deployment
3. **Use offline keys** for production signing
4. **Verify signatures** after signing
5. **Rotate keys** annually
6. **Keep private keys** on air-gapped machine or HSM

### Resource Management

1. **Declare realistic limits** based on testing
2. **Test under load** before production
3. **Monitor resource usage** in production
4. **Use conservative defaults** (err on the side of caution)
5. **Handle limit violations** gracefully

### Error Handling

1. **Never panic** in plugin code (use `Result`)
2. **Log errors** appropriately
3. **Clean up resources** in `shutdown()`
4. **Validate inputs** from context
5. **Return descriptive errors**

### Code Quality

1. **Use `cargo fmt`** for consistent formatting
2. **Use `cargo clippy`** to catch common mistakes
3. **Write tests** for critical functionality
4. **Document public APIs** with doc comments
5. **Keep dependencies minimal**

### Performance

1. **Avoid blocking operations** in async code
2. **Use async I/O** where possible
3. **Pool resources** (database connections, HTTP clients)
4. **Cache expensive computations**
5. **Profile performance** under realistic load

### Compatibility

1. **Use semantic versioning** correctly
2. **Document breaking changes**
3. **Test with different host versions**
4. **Avoid unstable features**
5. **Declare minimum Rust version** in `Cargo.toml`

---

## Troubleshooting

### Plugin Won't Load

**Error**: `Failed to load plugin: Untrusted plugin`

**Solution**:
1. Compute hash: `sha3sum -a 512 plugin.so`
2. Sign plugin: `./scripts/sign-plugin.sh sign plugin.so key.key`
3. Add hash to trust list
4. Verify signature: `./scripts/sign-plugin.sh verify plugin.so pubkey`

---

**Error**: `Failed to load plugin: Symbol not found`

**Solution**:
1. Check `crate-type = ["cdylib"]` in `Cargo.toml`
2. Rebuild with `cargo build --release`
3. Verify symbols: `nm -D plugin.so | grep create_plugin`

---

**Error**: `Failed to load plugin: Invalid signature`

**Solution**:
1. Re-sign plugin with correct key
2. Verify public key matches private key
3. Check signature hasn't been corrupted

---

### Resource Limit Violations

**Error**: Plugin unloaded due to memory violation

**Solution**:
1. Increase `max_heap_bytes` in `resource_limits()`
2. Fix memory leaks in plugin code
3. Use profiler to identify allocations

---

**Error**: Plugin unloaded due to CPU time violation

**Solution**:
1. Increase `max_cpu_time_ms` in `resource_limits()`
2. Optimize hot code paths
3. Use async I/O instead of blocking operations

---

### Build Errors

**Error**: `error[E0433]: failed to resolve: use of undeclared crate or module`

**Solution**:
1. Add missing dependency to `Cargo.toml`
2. Run `cargo build` to download dependencies

---

**Error**: `error: linking with cc failed`

**Solution**:
1. Install C compiler: `sudo apt-get install build-essential`
2. Check Rust toolchain: `rustup update`

---

### Runtime Errors

**Error**: Plugin crashes on initialization

**Solution**:
1. Check logs for panic message
2. Add error handling in `init()`
3. Validate context access
4. Test in debug mode with `RUST_BACKTRACE=1`

---

**Error**: Hook not being called

**Solution**:
1. Verify hook name matches exactly
2. Check hook priority (lower = earlier)
3. Ensure `register_hooks()` completes without error
4. Check host is triggering the hook

---

## Example: Complete Plugin

Here's a complete example plugin that demonstrates all features:

```rust
use plugin_api::{
    make_plugin, PluginContext, PluginError, ResourceLimits,
    PluginRequirements, NetworkRequirement, NetworkTarget,
    ContextKey, PredefinedContextKey, HookRegistry,
};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

// Define plugin with all features
make_plugin!({
    plugin_name: CompletePlugin,
    name: "complete_plugin",
    version: "1.0.0",
    author: "Your Name <your.email@example.com>",
    description: "
# Complete Plugin Example

This is a **complete example** plugin demonstrating all features.

## Features
- Database access
- Resource limits
- Network requirements
- Hook registration
- State management
",
    
    // Declare resource limits
    resource_limits() -> ResourceLimits {
        ResourceLimits {
            max_heap_bytes: 50 * 1024 * 1024,  // 50 MB
            max_cpu_time_ms: 2000,              // 2 seconds
            max_threads: 4,
            max_file_descriptors: 32,
            max_connections: 10,
        }
    }
    
    // Declare requirements
    requirements() -> PluginRequirements {
        PluginRequirements {
            network: NetworkRequirement::Restricted {
                allowed_targets: vec![
                    NetworkTarget::Domain("api.example.com".to_string()),
                ],
            },
            ..Default::default()
        }
    }
    
    // Plugin state
    state: {
        request_count: Arc<RwLock<u64>>,
        db: Option<Arc<DatabaseConnection>>,
    }
    
    // Initialization
    init(&mut self, context: &mut PluginContext) -> Result<(), PluginError> {
        info!("Initializing CompletePlugin");
        
        // Initialize state
        self.request_count = Arc::new(RwLock::new(0));
        
        // Access database
        let db = context.get::<DatabaseConnection>(
            ContextKey::Predefined(PredefinedContextKey::DatabaseConnection)
        )?;
        
        // Test connection
        if db.ping().await.is_ok() {
            info!("Database connection successful!");
            self.db = Some(db);
        } else {
            warn!("Database connection failed!");
        }
        
        info!("CompletePlugin initialized successfully");
        Ok(())
    }
    
    // Shutdown
    shutdown(&mut self) -> Result<(), PluginError> {
        info!("Shutting down CompletePlugin");
        
        // Log final count
        let count = *self.request_count.read().await;
        info!("Processed {} requests", count);
        
        info!("CompletePlugin shut down successfully");
        Ok(())
    }
    
    // Hook registration
    register_hooks(&self, hook_registry: *mut ()) -> Result<(), PluginError> {
        info!("Registering hooks for CompletePlugin");
        
        let registry = unsafe { &mut *(hook_registry as *mut HookRegistry) };
        let counter = self.request_count.clone();
        let db = self.db.clone();
        
        // Register request hook
        registry.register(
            "on_request",
            200,  // Normal priority
            Box::new(move |mut data: Vec<u8>| {
                let counter = counter.clone();
                let db = db.clone();
                
                Box::pin(async move {
                    // Increment counter
                    {
                        let mut count = counter.write().await;
                        *count += 1;
                        info!("Request #{}", *count);
                    }
                    
                    // Process request with database
                    if let Some(db) = db {
                        // Example: query database
                        match db.ping().await {
                            Ok(_) => info!("Database query successful"),
                            Err(e) => error!("Database query failed: {}", e),
                        }
                    }
                    
                    // Modify data (example: add header)
                    data.extend_from_slice(b"X-Plugin: CompletePlugin\n");
                    
                    Ok(data)
                })
            }),
        );
        
        info!("Hooks registered successfully");
        Ok(())
    }
});

// Export signature symbols (optional but recommended)
#[no_mangle]
pub static PLUGIN_SIGNATURE: &str = "complete_plugin_v1.0.0";

#[no_mangle]
pub static PLUGIN_HASH: &str = "will_be_computed_during_build";
```

---

For more information:
- [Features Documentation](features.md)
- [Security Guide](security.md)
- [Integration Guide](integration.md)
- [API Reference](api-reference.md)

