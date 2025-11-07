# Integration Guide

Complete guide to integrating the Plugin API into your Rust application.

## Table of Contents

- [Overview](#overview)
- [Dependencies](#dependencies)
- [Basic Integration](#basic-integration)
- [Advanced Configuration](#advanced-configuration)
- [Feature Flags](#feature-flags)
- [Security Setup](#security-setup)
- [Sandboxing Setup (Linux)](#sandboxing-setup-linux)
- [Context Management](#context-management)
- [Hook Management](#hook-management)
- [Monitoring Integration](#monitoring-integration)
- [Error Handling](#error-handling)
- [Production Deployment](#production-deployment)
- [Complete Example](#complete-example)

---

## Overview

The Plugin API can be integrated into any Rust application to provide secure, dynamic plugin loading capabilities. This guide covers everything from basic setup to advanced production configurations.

### Architecture Overview

```
Your Application
├── Core Application Code
├── Plugin System Integration
│   ├── PluginRegistry (manages plugins)
│   ├── PluginLoader (loads .so files)
│   ├── PluginSecurity (verifies signatures)
│   ├── PluginContext (shares data)
│   └── HookRegistry (event system)
└── Plugins (loaded dynamically)
    ├── Plugin A
    ├── Plugin B
    └── Plugin C
```

---

## Dependencies

### Required Dependencies

Add to your `Cargo.toml`:

```toml
[dependencies]
plugin-api = { path = "../plugin-api", features = ["server"] }
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

### Optional Dependencies

For database integration:
```toml
sea-orm = { version = "0.12", features = ["runtime-tokio-rustls", "sqlx-postgres"] }
```

For HTTP server:
```toml
axum = "0.7"
tower = "0.4"
```

For configuration:
```toml
serde = { version = "1", features = ["derive"] }
toml = "0.8"
```

### Feature Flags

```toml
[dependencies.plugin-api]
path = "../plugin-api"
features = [
    "server",      # Server-side features (required)
    "loader",      # Plugin loading (required)
    "security",    # Security features (required)
    "registry",    # Plugin registry (required)
    "sandboxing",  # Linux sandboxing (optional, Linux only)
    "macros",      # Macro support (optional, for plugins)
]
```

---

## Basic Integration

### Step 1: Create Plugin Context

The context is used to share data between your application and plugins:

```rust
use plugin_api::PluginContext;
use std::sync::Arc;

// Create context
let context = Arc::new(PluginContext::new());
```

### Step 2: Create Hook Registry

The hook registry manages event handlers:

```rust
use plugin_api::HookRegistry;
use std::sync::RwLock;

// Create hook registry
let hook_registry = Arc::new(RwLock::new(HookRegistry::new()));
```

### Step 3: Configure Security

Set up security with public keys and trusted plugins:

```rust
use plugin_api::{
    PluginSecurity, SecurityPolicy, PublicKey,
    TrustedPluginEntry, PluginVersion, PluginSignature,
};

// Load public keys (hardcoded for security)
let hardcoded_public_keys = vec![
    PublicKey::from_hex("your_public_key_hex_here")?,
];

// Define trusted plugins
let trusted_plugins = vec![
    TrustedPluginEntry {
        hash: "sha3_512_hash_of_plugin".to_string(),
        version: PluginVersion::new(1, 0, 0),
        signature: PluginSignature::from_hex("plugin_signature_hex")?,
        note: Some("Example Plugin".to_string()),
    },
];

// Create security policy
let security_policy = SecurityPolicy {
    only_trusted: true,
    trust_list_path: Some("data/plugin_trust_list.enc".into()),
};

// Create security manager
let security = Arc::new(PluginSecurity::new(
    security_policy,
    hardcoded_public_keys,
    trusted_plugins,
));
```

### Step 4: Create Plugin Loader

```rust
use plugin_api::PluginLoader;

// Create loader
let loader = Arc::new(PluginLoader::new(security.clone()));
```

### Step 5: Create Plugin Registry

```rust
use plugin_api::PluginRegistry;

// Create registry
let registry = Arc::new(PluginRegistry::new(
    loader,
    context.clone(),
    hook_registry.clone(),
    security,
));
```

### Step 6: Load Plugins

```rust
use plugin_api::TrustLevel;

// Load a single plugin
let plugin_name = registry.load_plugin(
    "/path/to/plugin.so",
    TrustLevel::Trusted,
).await?;

println!("Loaded plugin: {}", plugin_name);

// Or scan a directory
registry.scan_directory("/path/to/plugins")?;
let available = registry.get_available_plugins()?;

for plugin_info in available {
    println!("Found plugin: {} v{}", plugin_info.name, plugin_info.version);
    
    // Load if trusted
    if plugin_info.trust_level == TrustLevel::Trusted {
        registry.load_plugin_by_name(&plugin_info.name).await?;
    }
}
```

### Step 7: Clean Up on Shutdown

```rust
// Unload all plugins gracefully
registry.unload_all().await?;
```

---

## Advanced Configuration

### Using Environment Variables

```rust
use std::env;

// Load encryption key from environment
let encryption_key = env::var("PLUGIN_TRUST_KEY")
    .expect("PLUGIN_TRUST_KEY not set");

// Load/decrypt trust list
let trusted_plugins = security.load_and_decrypt_trust_list(&encryption_key)?;
```

### Configuration File

Create `config/plugins.toml`:

```toml
[plugins]
directory = "/opt/myapp/plugins"
auto_load = true
scan_on_start = true

[security]
only_trusted = true
trust_list_path = "data/plugin_trust_list.enc"

[monitoring]
enabled = true
check_interval_secs = 10
auto_unmount = true
violation_threshold = 10

[sandboxing]
enabled = true  # Linux only
mode = "strict"

[sandboxing.cgroups]
memory_limit_mb = 100
cpu_quota_percent = 50
max_pids = 50
```

Load configuration:

```rust
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Config {
    plugins: PluginConfig,
    security: SecurityConfig,
    monitoring: MonitoringConfig,
    sandboxing: SandboxConfig,
}

#[derive(Deserialize)]
struct PluginConfig {
    directory: String,
    auto_load: bool,
    scan_on_start: bool,
}

// ... other config structs

// Load config
let config_str = fs::read_to_string("config/plugins.toml")?;
let config: Config = toml::from_str(&config_str)?;
```

### Dynamic Plugin Loading

```rust
use axum::{Router, routing::post, Json};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct LoadPluginRequest {
    plugin_name: String,
}

#[derive(Serialize)]
struct LoadPluginResponse {
    success: bool,
    message: String,
}

async fn load_plugin_endpoint(
    registry: Arc<PluginRegistry>,
    Json(req): Json<LoadPluginRequest>,
) -> Json<LoadPluginResponse> {
    match registry.load_plugin_by_name(&req.plugin_name).await {
        Ok(_) => Json(LoadPluginResponse {
            success: true,
            message: format!("Plugin '{}' loaded successfully", req.plugin_name),
        }),
        Err(e) => Json(LoadPluginResponse {
            success: false,
            message: format!("Failed to load plugin: {}", e),
        }),
    }
}

// Add to router
let app = Router::new()
    .route("/api/plugins/load", post(load_plugin_endpoint));
```

---

## Feature Flags

### Available Features

| Feature | Description | Required |
|---------|-------------|----------|
| `server` | Server-side functionality | Yes |
| `loader` | Plugin loading | Yes |
| `security` | Security features | Yes |
| `registry` | Plugin registry | Yes |
| `sandboxing` | Linux sandboxing | No |
| `macros` | Plugin macros | No* |
| `context-proxy` | IPC context proxy | No* |

\* Required for plugin development, not for server integration

### Conditional Compilation

```rust
// Only compile sandboxing code on Linux
#[cfg(all(feature = "sandboxing", target_os = "linux"))]
fn setup_sandboxing() -> SandboxConfig {
    SandboxConfig {
        enable_pid_namespace: true,
        enable_network_namespace: true,
        enable_mount_namespace: true,
        // ... more config
    }
}

#[cfg(not(all(feature = "sandboxing", target_os = "linux")))]
fn setup_sandboxing() -> SandboxConfig {
    // Dummy config for non-Linux platforms
    SandboxConfig::default()
}
```

---

## Security Setup

### Generate Keys

```bash
# Generate signing keys
./scripts/sign-plugin.sh generate-key production

# Output:
# keys/production.key (keep secret!)
# keys/production.pub
```

### Add Public Keys to Application

```rust
// Hardcode public keys (don't load from files at runtime!)
let hardcoded_public_keys = vec![
    // Production key
    PublicKey::from_hex(
        "ed01907e5e1b3f7c2d4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e"
    )?,
    
    // Backup key
    PublicKey::from_hex(
        "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
    )?,
];
```

### Encrypt Trust List

```rust
// First time: create and encrypt trust list
let encryption_key = generate_random_key(); // 32 bytes, hex-encoded
security.encrypt_and_save_trust_list(&encryption_key)?;

// Store key securely (environment variable, secrets manager, etc.)
println!("Save this key securely: {}", encryption_key);
```

### Load Trust List

```rust
// On application start
let encryption_key = env::var("PLUGIN_TRUST_KEY")?;

// Load and decrypt trust list
let trusted_plugins = security.load_and_decrypt_trust_list(&encryption_key)?;
```

### Runtime Trust Management

```rust
// Add a new trusted plugin at runtime
let new_plugin = TrustedPluginEntry {
    hash: compute_hash("/path/to/new_plugin.so")?,
    version: PluginVersion::new(1, 0, 0),
    signature: read_signature("/path/to/new_plugin.so")?,
    note: Some("New Plugin".to_string()),
};

security.add_trusted_plugin(new_plugin)?;

// Save updated trust list
security.encrypt_and_save_trust_list(&encryption_key)?;
```

---

## Sandboxing Setup (Linux)

### Prerequisites

```bash
# Install required tools
sudo apt-get install -y libcap-dev libseccomp-dev

# Add Rust dependencies
cargo add caps seccompiler
```

### Create Sandbox Configuration

```rust
use plugin_api::sandbox::{SandboxConfig, SeccompMode};

let sandbox_config = SandboxConfig {
    // Namespace isolation
    enable_pid_namespace: true,
    enable_network_namespace: true,
    enable_mount_namespace: true,
    enable_ipc_namespace: true,
    enable_uts_namespace: true,
    enable_user_namespace: false, // Usually requires root
    
    // Cgroups
    cgroup_config: Some(CgroupConfig {
        memory_limit_bytes: Some(100 * 1024 * 1024), // 100 MB
        cpu_quota_us: Some(100_000), // 100ms per 100ms
        cpu_period_us: 100_000,
        pids_limit: Some(50),
        io_weight: Some(100),
    }),
    
    // Seccomp filtering
    seccomp_mode: SeccompMode::Strict,
    
    // Capabilities
    capabilities_config: CapabilitiesConfig {
        drop_all: true,
        allowed_caps: vec![],
        no_new_privs: true,
    },
    
    // Filesystem
    filesystem_config: FilesystemConfig {
        root_path: "/var/lib/myapp/plugins/root".into(),
        read_only_paths: vec![
            "/usr".into(),
            "/lib".into(),
            "/lib64".into(),
        ],
        writable_paths: vec![
            "/tmp".into(),
        ],
        bind_mounts: vec![],
    },
    
    // Network
    network_config: NetworkConfig {
        mode: NetworkMode::Restricted,
        allowed_targets: vec![
            NetworkTarget::Domain("api.example.com".to_string()),
        ],
    },
};
```

### Create Process Manager

```rust
#[cfg(target_os = "linux")]
use plugin_api::process::PluginProcessManager;

#[cfg(target_os = "linux")]
let process_manager = Arc::new(
    PluginProcessManager::new(sandbox_config)
);

// Create sandboxed registry
#[cfg(target_os = "linux")]
let registry = Arc::new(PluginRegistry::new_sandboxed(
    loader,
    context,
    hook_registry,
    security,
    process_manager,
));
```

### Permissions

Sandboxing may require elevated privileges:

```bash
# Option 1: Run as root (not recommended)
sudo ./myapp

# Option 2: Grant capabilities
sudo setcap cap_sys_admin,cap_setuid,cap_setgid=ep ./myapp

# Option 3: Use unprivileged user namespaces (recommended)
# Enable unprivileged user namespaces
echo 1 | sudo tee /proc/sys/kernel/unprivileged_userns_clone

# Allow more subordinate IDs
echo "username:100000:65536" | sudo tee -a /etc/subuid
echo "username:100000:65536" | sudo tee -a /etc/subgid
```

---

## Context Management

### Adding Context Data

```rust
use plugin_api::{ContextKey, PredefinedContextKey};
use sea_orm::DatabaseConnection;

// Add database connection
let db: DatabaseConnection = Database::connect(options).await?;
context.set(
    ContextKey::Predefined(PredefinedContextKey::DatabaseConnection),
    Arc::new(db),
)?;

// Add custom data
#[derive(Clone)]
struct AppConfig {
    api_key: String,
    base_url: String,
}

let config = AppConfig {
    api_key: "secret".to_string(),
    base_url: "https://api.example.com".to_string(),
};

context.set(
    ContextKey::Custom("app_config".to_string()),
    Arc::new(config),
)?;
```

### Context with Thread-Local Storage

```rust
use std::sync::Arc;
use tokio::task_local;

// Define thread-local context
task_local! {
    pub static PLUGIN_CONTEXT: Arc<PluginContext>;
}

// Use in async context
async fn my_handler() {
    PLUGIN_CONTEXT.with(|ctx| {
        // Use context
        let db = ctx.get::<DatabaseConnection>(
            ContextKey::Predefined(PredefinedContextKey::DatabaseConnection)
        ).unwrap();
    });
}
```

---

## Hook Management

### Define Hook Points

```rust
// Define hook names as constants
pub mod hooks {
    pub const ON_REQUEST: &str = "on_request";
    pub const ON_RESPONSE: &str = "on_response";
    pub const ON_ERROR: &str = "on_error";
    pub const ON_STARTUP: &str = "on_startup";
    pub const ON_SHUTDOWN: &str = "on_shutdown";
}
```

### Execute Hooks

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct RequestData {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

// Execute hook
async fn handle_request(
    hook_registry: Arc<RwLock<HookRegistry>>,
    request_data: RequestData,
) -> Result<RequestData, PluginError> {
    // Serialize data
    let data = bincode::serialize(&request_data)?;
    
    // Execute hooks
    let registry = hook_registry.read().unwrap();
    let result_data = registry.execute(hooks::ON_REQUEST, data).await?;
    
    // Deserialize result
    let modified_request: RequestData = bincode::deserialize(&result_data)?;
    
    Ok(modified_request)
}
```

### Hook Execution Policies

```rust
// Execute all handlers (default)
let result = registry.execute(hook_name, data).await?;

// Execute until first error
let result = registry.execute_until_error(hook_name, data).await?;

// Execute and collect results
let results = registry.execute_all(hook_name, data).await?;
```

---

## Monitoring Integration

### Start Resource Monitor

```rust
use std::time::Duration;

// Start monitoring with custom interval
let monitor_handle = registry.clone().start_resource_monitor(
    Some(Duration::from_secs(10))
);

// Monitor runs in background, checking every 10 seconds
```

### Custom Violation Handling

```rust
use plugin_api::ViolationType;

// Check for violations manually
let violations = registry.check_plugin_resources("my_plugin")?;

for violation in violations {
    match violation {
        ViolationType::HeapMemory { used, limit } => {
            tracing::warn!(
                "Plugin exceeded memory: {} MB / {} MB",
                used / 1024 / 1024,
                limit / 1024 / 1024
            );
        }
        ViolationType::CpuTime { used_ms, limit_ms } => {
            tracing::warn!(
                "Plugin exceeded CPU time: {} ms / {} ms",
                used_ms, limit_ms
            );
        }
        // ... handle other violations
        _ => {}
    }
}
```

### Configure Unmount Behavior

```rust
use plugin_api::UnmountBehavior;

// Configure for specific plugin
registry.set_unmount_behavior(
    "my_plugin",
    UnmountBehavior {
        auto_unmount: true,
        log_violations: true,
    },
)?;
```

### Metrics Collection

```rust
// Get metrics for a plugin
let metrics = registry.get_plugin_metrics("my_plugin")?;

println!("Plugin Metrics:");
println!("  Heap: {} MB", metrics.heap_bytes / 1024 / 1024);
println!("  CPU Time: {} ms", metrics.cpu_time_ms);
println!("  Threads: {}", metrics.thread_count);
println!("  FDs: {}", metrics.fd_count);
println!("  Connections: {}", metrics.connection_count);
println!("  Violations: {}", metrics.violation_count);
```

---

## Error Handling

### Plugin Errors

```rust
use plugin_api::PluginError;

match registry.load_plugin("/path/to/plugin.so", TrustLevel::Trusted).await {
    Ok(name) => {
        tracing::info!("Loaded plugin: {}", name);
    }
    Err(PluginError::UntrustedPlugin) => {
        tracing::error!("Plugin is not in trust list");
    }
    Err(PluginError::LoadError(msg)) => {
        tracing::error!("Failed to load plugin: {}", msg);
    }
    Err(PluginError::InitializationError(msg)) => {
        tracing::error!("Plugin initialization failed: {}", msg);
    }
    Err(e) => {
        tracing::error!("Plugin error: {}", e);
    }
}
```

### Graceful Degradation

```rust
// Try to load plugins, but continue if they fail
for plugin_path in plugin_paths {
    match registry.load_plugin(&plugin_path, TrustLevel::Trusted).await {
        Ok(name) => {
            tracing::info!("Loaded plugin: {}", name);
        }
        Err(e) => {
            tracing::warn!("Failed to load plugin {}: {}", plugin_path, e);
            // Continue without this plugin
        }
    }
}

// Application continues even if no plugins loaded
tracing::info!("Loaded {} plugins", registry.plugin_count());
```

---

## Production Deployment

### Systemd Service

Create `/etc/systemd/system/myapp.service`:

```ini
[Unit]
Description=My Application with Plugins
After=network.target

[Service]
Type=simple
User=myapp
Group=myapp
WorkingDirectory=/opt/myapp
Environment="PLUGIN_TRUST_KEY=your_key_here"
Environment="RUST_LOG=info"
ExecStart=/opt/myapp/bin/myapp
Restart=on-failure
RestartSec=5s

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/myapp/data

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096

[Install]
WantedBy=multi-user.target
```

### Environment Variables

```bash
# Production environment
export PLUGIN_TRUST_KEY="your_32_byte_hex_key"
export RUST_LOG="info,myapp=debug"
export PLUGIN_DIR="/opt/myapp/plugins"
```

### Logging

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Configure logging
tracing_subscriber::registry()
    .with(
        tracing_subscriber::EnvFilter::try_from_env("RUST_LOG")
            .unwrap_or_else(|_| "info,myapp=debug".into()),
    )
    .with(tracing_subscriber::fmt::layer())
    .init();
```

### Health Checks

```rust
use axum::{Router, routing::get, Json};
use serde::Serialize;

#[derive(Serialize)]
struct HealthStatus {
    status: String,
    plugins_loaded: usize,
    plugins_total: usize,
}

async fn health_check(
    registry: Arc<PluginRegistry>,
) -> Json<HealthStatus> {
    Json(HealthStatus {
        status: "ok".to_string(),
        plugins_loaded: registry.plugin_count(),
        plugins_total: registry.total_plugin_count(),
    })
}

let app = Router::new()
    .route("/health", get(health_check));
```

---

## Complete Example

Here's a complete, production-ready integration example:

```rust
use std::sync::Arc;
use std::env;
use tokio::signal;
use tracing::{info, error};
use plugin_api::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_env("RUST_LOG")
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    info!("Starting application with plugin system");
    
    // Load configuration
    let plugin_dir = env::var("PLUGIN_DIR")
        .unwrap_or_else(|_| "/opt/myapp/plugins".to_string());
    let trust_key = env::var("PLUGIN_TRUST_KEY")
        .expect("PLUGIN_TRUST_KEY must be set");
    
    // Create core components
    let context = Arc::new(PluginContext::new());
    let hook_registry = Arc::new(std::sync::RwLock::new(HookRegistry::new()));
    
    // Configure security
    let hardcoded_public_keys = vec![
        PublicKey::from_hex(
            "ed01907e5e1b3f7c2d4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e"
        )?,
    ];
    
    let trusted_plugins = vec![
        TrustedPluginEntry {
            hash: "plugin_hash_here".to_string(),
            version: PluginVersion::new(1, 0, 0),
            signature: PluginSignature::from_hex("signature_here")?,
            note: Some("Example Plugin".to_string()),
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
        context.clone(),
        hook_registry.clone(),
        security,
    ));
    
    // Scan and load plugins
    info!("Scanning plugin directory: {}", plugin_dir);
    match registry.scan_directory(&plugin_dir) {
        Ok(count) => info!("Discovered {} plugins", count),
        Err(e) => error!("Failed to scan plugin directory: {}", e),
    }
    
    // Load available plugins
    let available = registry.get_available_plugins()?;
    info!("Loading {} available plugins", available.len());
    
    for plugin_info in available {
        info!("Loading plugin: {} v{}", plugin_info.name, plugin_info.version);
        match registry.load_plugin_by_name(&plugin_info.name).await {
            Ok(_) => info!("Successfully loaded: {}", plugin_info.name),
            Err(e) => error!("Failed to load {}: {}", plugin_info.name, e),
        }
    }
    
    info!("Loaded {} plugins", registry.plugin_count());
    
    // Start resource monitoring
    let monitor = registry.clone().start_resource_monitor(None);
    info!("Resource monitoring started");
    
    // Run application
    info!("Application running");
    
    // Wait for shutdown signal
    signal::ctrl_c().await?;
    info!("Shutdown signal received");
    
    // Cleanup
    info!("Unloading plugins");
    registry.unload_all().await?;
    
    info!("Stopping resource monitor");
    monitor.abort();
    
    info!("Application shutdown complete");
    Ok(())
}
```

---

For more information:
- [Features Documentation](features.md)
- [Security Guide](security.md)
- [Plugin Development Guide](plugin-development.md)
- [API Reference](api-reference.md)

