# Plugin API Features

This document provides a comprehensive overview of all implemented features in the Plugin API system.

## Table of Contents

- [Core Plugin System](#core-plugin-system)
- [Security System](#security-system)
- [Resource Management](#resource-management)
- [Monitoring System](#monitoring-system)
- [Sandboxing (Linux Only)](#sandboxing-linux-only)
- [IPC System](#ipc-system)
- [Hook System](#hook-system)
- [Context Management](#context-management)
- [Registry Management](#registry-management)

---

## Core Plugin System

### Dynamic Plugin Loading

**Status**: ✅ Fully Implemented

- **Dynamic Library Loading**: Load `.so` (Linux), `.dll` (Windows), `.dylib` (macOS) files at runtime
- **Symbol Resolution**: Automatic resolution of plugin constructor functions
- **Hot Reload**: Load and unload plugins without application restart
- **Version Management**: Track and validate plugin versions
- **Dependency Management**: Ensure plugins have access to required dependencies

**Key Components**:
- `PluginLoader` - Handles dynamic library loading and symbol resolution
- `Plugin` trait - Core trait that all plugins must implement
- `BridgedPlugin` trait - Internal bridge between raw plugin and typed interface

**Usage Example**:
```rust
let loader = Arc::new(PluginLoader::new(security));
let (plugin, library) = loader.load("/path/to/plugin.so", TrustLevel::Trusted)?;
```

### Plugin Lifecycle Management

**Status**: ✅ Fully Implemented

- **Initialization**: Async `init()` method with context access
- **Shutdown**: Graceful `shutdown()` with cleanup guarantees
- **Hook Registration**: Register event handlers during initialization
- **Status Tracking**: Monitor plugin state (Available, Active, Inactive, Failed, Untrusted)

**Plugin States**:
1. **Available** - Discovered but not loaded
2. **Active** - Loaded and running
3. **Inactive** - Loaded but paused
4. **Failed** - Load/init failed
5. **Untrusted** - Not in trust list

### Macro System

**Status**: ✅ Fully Implemented

The `make_plugin!` macro provides a declarative way to create plugins:

```rust
make_plugin!({
    plugin_name: MyPlugin,
    name: "my_plugin",
    version: "1.0.0",
    author: "Author Name",
    description: "Optional markdown description",
    
    // Optional: Declare resource limits
    resource_limits() -> ResourceLimits {
        ResourceLimits {
            max_heap_bytes: 100 * 1024 * 1024, // 100 MB
            max_cpu_time_ms: 5000,              // 5 seconds
            max_threads: 10,
            max_file_descriptors: 50,
            max_connections: 20,
        }
    }
    
    // Optional: Declare requirements
    requirements() -> PluginRequirements {
        PluginRequirements {
            network: NetworkRequirement::Restricted {
                allowed_targets: vec![
                    NetworkTarget::Domain("api.example.com".to_string()),
                ],
            },
            filesystem: FilesystemRequirement::ReadOnly {
                allowed_paths: vec!["/data/readonly".into()],
            },
            ..Default::default()
        }
    }
    
    init(&mut self, context: &mut PluginContext) -> Result<(), PluginError> {
        // Initialize plugin
        Ok(())
    }
    
    shutdown(&mut self) -> Result<(), PluginError> {
        // Cleanup
        Ok(())
    }
    
    register_hooks(&self, hook_registry: *mut ()) -> Result<(), PluginError> {
        // Register hooks
        Ok(())
    }
});
```

---

## Security System

### Cryptographic Verification

**Status**: ✅ Fully Implemented

#### Ed25519 Signatures

- **Algorithm**: Ed25519 (Curve25519)
- **Key Size**: 256-bit public keys
- **Signature Size**: 512-bit signatures
- **Performance**: Fast verification (~50μs per signature)

**Key Generation**:
```bash
./scripts/sign-plugin.sh generate-key my-key
```

**Plugin Signing**:
```bash
./scripts/sign-plugin.sh sign target/release/libplugin.so keys/my-key.key
```

**Verification**: Automatic during plugin load

#### SHA3-512 Hashing

- **Algorithm**: SHA3-512 (Keccak)
- **Hash Size**: 512-bit (64 bytes)
- **Collision Resistance**: 2^256 security level
- **Purpose**: Plugin integrity verification

**Hash Computation**:
```bash
./scripts/compute-plugin-hashes.sh release
```

### Trust Management

**Status**: ✅ Fully Implemented

#### Trust Levels

- **Trusted**: Verified plugins that can be loaded
- **Untrusted**: Unverified plugins (blocked by default)

#### Trust List

**Features**:
- **Encrypted Storage**: XChaCha20-Poly1305 encryption
- **Binary Format**: Opaque encrypted blob
- **Version Tracking**: Associate hash with version
- **Notes**: Optional metadata for each entry

**Structure**:
```rust
pub struct TrustedPluginEntry {
    pub hash: String,              // SHA3-512 hash
    pub version: PluginVersion,    // Semantic version
    pub signature: PluginSignature, // Ed25519 signature
    pub note: Option<String>,      // Optional description
}
```

#### Public Key Pinning

Hardcoded public keys in the application:

```rust
let hardcoded_public_keys = vec![
    PublicKey::from_hex("a1b2c3...")?,
    PublicKey::from_hex("d4e5f6...")?,
];
```

### Security Policy

**Status**: ✅ Fully Implemented

```rust
pub struct SecurityPolicy {
    pub only_trusted: bool,              // Only load trusted plugins
    pub trust_list_path: Option<PathBuf>, // Encrypted trust list location
}
```

**Default Policy**:
- Only trusted plugins allowed
- Trust list at `data/plugin_trust_list.enc`

---

## Resource Management

### Resource Limits

**Status**: ✅ Fully Implemented

Each plugin can declare resource limits:

```rust
pub struct ResourceLimits {
    pub max_heap_bytes: usize,        // Maximum heap memory
    pub max_cpu_time_ms: u64,         // Maximum CPU time per operation
    pub max_threads: u32,             // Maximum thread count
    pub max_file_descriptors: u32,    // Maximum open files
    pub max_connections: u32,         // Maximum network connections
}
```

**Default Limits**:
- Heap: 50 MB
- CPU Time: 1000 ms
- Threads: 4
- File Descriptors: 32
- Connections: 10

**Validation**:
- Automatic validation on plugin load
- Configurable bounds checking
- Violations trigger warnings/unmount

### Violation Tracking

**Status**: ✅ Fully Implemented

#### Violation Types

```rust
pub enum ViolationType {
    HeapMemory { used: usize, limit: usize },
    CpuTime { used_ms: u64, limit_ms: u64 },
    Threads { used: u32, limit: u32 },
    FileDescriptors { used: u32, limit: u32 },
    Connections { used: u32, limit: u32 },
}
```

#### Violation Tracker

```rust
pub struct ViolationTracker {
    violations: Vec<(ViolationType, SystemTime)>,
    violation_threshold: usize,  // Default: 10
}
```

**Features**:
- Track all violations with timestamps
- Configurable threshold for auto-unmount
- Manual reset capability
- Query violation count

### Unmount Behavior

**Status**: ✅ Fully Implemented

```rust
pub struct UnmountBehavior {
    pub auto_unmount: bool,      // Auto-unmount on threshold
    pub log_violations: bool,    // Log all violations
}
```

**Default Behavior**:
- Auto-unmount enabled
- Violation logging enabled

---

## Monitoring System

### Resource Monitor

**Status**: ✅ Fully Implemented

**Features**:
- **Background Task**: Tokio-based async monitoring
- **Configurable Interval**: Default 10 seconds
- **Automatic Enforcement**: Auto-unmount on violations
- **Status Logging**: Periodic status reports
- **Platform-specific**: Full support on Linux, limited elsewhere

**Usage**:
```rust
let registry = Arc::new(PluginRegistry::new(...));
let monitor = registry.clone().start_resource_monitor(Some(Duration::from_secs(10)));

// Later: stop monitoring
monitor.abort();
```

### Linux Monitoring

**Status**: ✅ Fully Implemented (Linux only)

**Monitored Resources** (via `/proc` filesystem):
- **Memory**: `/proc/[pid]/status` - VmRSS field
- **Threads**: `/proc/[pid]/status` - Threads field
- **File Descriptors**: `/proc/[pid]/fd/` - count entries
- **Network Connections**: `/proc/net/tcp` and `/proc/net/tcp6`
- **CPU Time**: `/proc/[pid]/stat` - utime + stime

**Platform Support**:
- ✅ **Linux**: Full monitoring via procfs
- ⚠️ **Windows**: Limited (basic monitoring via WinAPI)
- ⚠️ **macOS**: Limited (basic monitoring via sysctl)

### Metrics Collection

**Status**: ✅ Fully Implemented

```rust
pub struct PluginMetrics {
    pub name: String,
    pub heap_bytes: usize,
    pub cpu_time_ms: u64,
    pub thread_count: u32,
    pub fd_count: u32,
    pub connection_count: u32,
    pub violation_count: usize,
}
```

---

## Sandboxing (Linux Only)

### Process Isolation

**Status**: ✅ Implemented (Linux only)

**Features**:
- Separate process per plugin
- IPC-based communication
- Crash isolation
- Memory isolation

**Components**:
- `PluginProcessManager` - Manages sandboxed plugin processes
- `PluginProcess` - Represents a single sandboxed plugin
- IPC channels for communication

### Linux Namespaces

**Status**: ✅ Implemented

**Supported Namespaces**:
- **PID Namespace**: Isolated process tree
- **Network Namespace**: Isolated network stack
- **Mount Namespace**: Isolated filesystem view
- **IPC Namespace**: Isolated IPC resources
- **UTS Namespace**: Isolated hostname
- **User Namespace**: Mapped user/group IDs

**Usage**:
```rust
let config = SandboxConfig {
    enable_pid_namespace: true,
    enable_network_namespace: true,
    enable_mount_namespace: true,
    enable_ipc_namespace: true,
    enable_uts_namespace: true,
    enable_user_namespace: false, // Requires root
    ..Default::default()
};
```

### Cgroups

**Status**: ✅ Implemented

**Resource Controls**:
- **CPU Quota**: Limit CPU time
- **Memory Limit**: Hard memory cap
- **PIDs Limit**: Maximum processes
- **I/O Weight**: Prioritize disk I/O

**Example**:
```rust
let cgroup_config = CgroupConfig {
    memory_limit_bytes: Some(100 * 1024 * 1024), // 100 MB
    cpu_quota_us: Some(100_000),                  // 100ms per 100ms period
    pids_limit: Some(50),
    ..Default::default()
};
```

### Seccomp Filtering

**Status**: ✅ Implemented

**Security Modes**:
1. **Strict**: Only read, write, exit, sigreturn allowed
2. **Basic**: Common syscalls allowed (file I/O, networking)
3. **Moderate**: Extended syscalls for most applications
4. **Permissive**: Most syscalls allowed (for development)

**Custom Filters**: Support for custom seccomp-bpf filters

### Capability Management

**Status**: ✅ Implemented

**Features**:
- Drop all capabilities by default
- Selective capability grants
- Ambient capability control
- No-new-privs enforcement

**Example**:
```rust
let caps_config = CapabilitiesConfig {
    drop_all: true,
    allowed_caps: vec![
        Capability::CAP_NET_BIND_SERVICE, // Bind ports < 1024
    ],
    no_new_privs: true,
};
```

### Filesystem Sandboxing

**Status**: ✅ Implemented

**Features**:
- **Read-only mounts**: Restrict write access
- **Tmpfs overlays**: Temporary writable layer
- **Path restrictions**: Whitelist specific paths
- **Bind mounts**: Map host paths into sandbox

**Example**:
```rust
let fs_config = FilesystemConfig {
    root_path: PathBuf::from("/var/lib/plugins/root"),
    read_only_paths: vec![
        "/usr".into(),
        "/lib".into(),
    ],
    writable_paths: vec![
        "/tmp".into(),
    ],
    bind_mounts: vec![
        BindMount {
            source: "/data/shared".into(),
            target: "/data".into(),
            readonly: true,
        },
    ],
};
```

### Network Sandboxing

**Status**: ✅ Implemented

**Network Modes**:
1. **Isolated**: No network access
2. **Restricted**: Whitelist specific targets
3. **Full**: Unrestricted access

**Features**:
- Virtual network interfaces
- Firewall rules (iptables/nftables)
- DNS resolution control
- Port binding restrictions

---

## IPC System

### IPC Protocol

**Status**: ✅ Fully Implemented

**Features**:
- **Request/Response**: Synchronous RPC-style calls
- **Binary Protocol**: Efficient bincode serialization
- **Type Safety**: Strongly typed messages
- **Error Handling**: Rich error types

**Message Types**:
```rust
pub enum IpcRequest {
    Initialize { config: PluginConfig },
    Shutdown,
    ExecuteHook { hook_name: String, data: Vec<u8> },
    GetMetrics,
}

pub enum IpcResponse {
    Success,
    Error { message: String },
    Metrics { data: PluginMetrics },
    HookResult { data: Vec<u8> },
}
```

### Unix Domain Sockets

**Status**: ✅ Implemented (Linux/macOS)

**Features**:
- **File-based**: Access control via filesystem permissions
- **High Performance**: No network overhead
- **Reliable**: TCP-like guarantees
- **Credentials Passing**: SCM_CREDENTIALS support

### Named Pipes (Windows)

**Status**: ✅ Implemented (Windows)

**Features**:
- **Windows-native**: Using Windows named pipes API
- **Security**: ACL-based access control
- **Duplex**: Bidirectional communication

### IPC Channel Management

**Status**: ✅ Fully Implemented

```rust
pub struct IpcChannel {
    // Platform-specific implementation
}

impl IpcChannel {
    pub async fn send_request(&mut self, request: IpcRequest) -> Result<IpcResponse>;
    pub async fn receive_request(&mut self) -> Result<IpcRequest>;
    pub async fn send_response(&mut self, response: IpcResponse) -> Result<()>;
}
```

---

## Hook System

### Hook Registry

**Status**: ✅ Fully Implemented

**Features**:
- **Priority-based**: Execute hooks in priority order
- **Multiple Handlers**: Many plugins can handle same hook
- **Async Execution**: Async/await support
- **Type Safety**: Strongly typed hook data

**Usage**:
```rust
let mut hook_registry = HookRegistry::new();

// Register a hook
hook_registry.register("on_request", priority, Box::new(|data| {
    Box::pin(async move {
        // Handle hook
        Ok(data)
    })
}));

// Execute all handlers
let result = hook_registry.execute("on_request", data).await?;
```

### Hook Priorities

**Priority Levels**:
- **Critical** (0-99): System-level hooks
- **High** (100-199): Important processing
- **Normal** (200-299): Regular processing (default: 200)
- **Low** (300-399): Optional processing
- **Cleanup** (400+): Final processing

### Hook Types

**Common Hook Points**:
- `on_request` - HTTP request handling
- `on_response` - HTTP response modification
- `on_error` - Error handling
- `on_startup` - Application startup
- `on_shutdown` - Application shutdown
- `on_plugin_load` - Plugin loaded
- `on_plugin_unload` - Plugin unloaded

---

## Context Management

### Plugin Context

**Status**: ✅ Fully Implemented

**Purpose**: Safe data sharing between host and plugins

**Features**:
- **Type-safe Storage**: `Any` type with downcast support
- **Key-based Access**: Predefined and custom keys
- **Arc-based**: Shared ownership
- **Thread-safe**: Send + Sync requirements

**Usage**:
```rust
let mut context = PluginContext::new();

// Store database connection
context.set(
    ContextKey::Predefined(PredefinedContextKey::DatabaseConnection),
    Arc::new(database),
)?;

// Retrieve in plugin
let db = context.get::<DatabaseConnection>(
    ContextKey::Predefined(PredefinedContextKey::DatabaseConnection)
)?;
```

### Predefined Context Keys

**Status**: ✅ Implemented

```rust
pub enum PredefinedContextKey {
    DatabaseConnection,
    Configuration,
    Logger,
    HttpClient,
    CacheManager,
    // Extensible...
}
```

### Custom Context Keys

**Status**: ✅ Implemented

```rust
let key = ContextKey::Custom("my_custom_key".to_string());
context.set(key, Arc::new(my_data))?;
```

### Context Proxy (IPC)

**Status**: ✅ Implemented

For sandboxed plugins, context access goes through IPC proxy:

```rust
pub struct ContextProxy {
    channel: IpcChannel,
}

impl ContextProxy {
    pub async fn get<T>(&self, key: ContextKey) -> Result<Arc<T>>;
    pub async fn set<T>(&mut self, key: ContextKey, value: Arc<T>) -> Result<()>;
}
```

---

## Registry Management

### Plugin Registry

**Status**: ✅ Fully Implemented

**Central Management**:
- Track all plugins (loaded + available)
- Load/unload plugins
- Query plugin status
- Manage violations
- Resource monitoring

### Plugin Discovery

**Status**: ✅ Fully Implemented

**Features**:
- **Directory Scanning**: Scan folders for plugins
- **Automatic Registration**: Discover and register plugins
- **Metadata Extraction**: Read version, hash, etc.
- **Trust Verification**: Check against trust list

**Usage**:
```rust
// Scan directory
registry.scan_directory("/path/to/plugins")?;

// Get available plugins
let available = registry.get_available_plugins()?;

// Load by name
registry.load_plugin_by_name("my_plugin").await?;
```

### Plugin Information

**Status**: ✅ Fully Implemented

```rust
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: Option<String>,
    pub status: PluginStatus,
    pub trust_level: TrustLevel,
    pub path: PathBuf,
    pub hash: String,
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub trust_info: Option<TrustedPluginEntry>,
    pub error: Option<String>,
    pub resource_limits: Option<ResourceLimits>,
}
```

### Plugin Queries

**Status**: ✅ Fully Implemented

```rust
// Get all plugins
let all_plugins = registry.get_all_plugins()?;

// Get loaded plugins only
let loaded = registry.list_loaded_plugins()?;

// Get available (unloaded) plugins
let available = registry.get_available_plugins()?;

// Get untrusted plugins
let untrusted = registry.get_untrusted_plugins()?;

// Get specific plugin info
let info = registry.get_plugin_info("my_plugin")?;

// Check if loaded
let is_loaded = registry.is_loaded("my_plugin");

// Get counts
let loaded_count = registry.plugin_count();
let total_count = registry.total_plugin_count();
```

### Batch Operations

**Status**: ✅ Fully Implemented

```rust
// Unload all plugins
registry.unload_all().await?;

// Reset all violations
for plugin in registry.list_loaded_plugins()? {
    registry.reset_violations(&plugin.name)?;
}
```

---

## Feature Matrix

| Feature | Status | Linux | Windows | macOS |
|---------|--------|-------|---------|-------|
| Dynamic Loading | ✅ | ✅ | ✅ | ✅ |
| Hot Reload | ✅ | ✅ | ✅ | ✅ |
| Ed25519 Signatures | ✅ | ✅ | ✅ | ✅ |
| SHA3-512 Hashing | ✅ | ✅ | ✅ | ✅ |
| Encrypted Trust Lists | ✅ | ✅ | ✅ | ✅ |
| Resource Limits | ✅ | ✅ | ⚠️ | ⚠️ |
| Resource Monitoring | ✅ | ✅ | ⚠️ | ⚠️ |
| Violation Tracking | ✅ | ✅ | ✅ | ✅ |
| Auto-unmount | ✅ | ✅ | ✅ | ✅ |
| Process Sandboxing | ✅ | ✅ | ❌ | ❌ |
| Namespaces | ✅ | ✅ | ❌ | ❌ |
| Cgroups | ✅ | ✅ | ❌ | ❌ |
| Seccomp | ✅ | ✅ | ❌ | ❌ |
| IPC (Unix Sockets) | ✅ | ✅ | ❌ | ✅ |
| IPC (Named Pipes) | ✅ | ❌ | ✅ | ❌ |
| Hook System | ✅ | ✅ | ✅ | ✅ |
| Context Management | ✅ | ✅ | ✅ | ✅ |

Legend:
- ✅ Fully Supported
- ⚠️ Limited/Partial Support
- ❌ Not Supported

---

## Coming Soon

Features planned for future releases:

- **Hot Reload with State Preservation**: Maintain plugin state across reloads
- **Plugin Dependencies**: Declare and manage plugin dependencies
- **Plugin Marketplace**: Discover and install community plugins
- **WebAssembly Support**: Run plugins in WASM sandbox
- **Docker Container Sandboxing**: Alternative to process sandboxing
- **Performance Profiling**: Built-in profiler for plugin performance
- **Plugin Testing Framework**: Unit and integration test support
- **Dynamic Permission Grants**: Runtime permission requests
- **Plugin Communication**: Direct inter-plugin messaging
- **Rollback Support**: Revert to previous plugin versions

---

For more details on specific features, see:
- [Security Guide](security.md)
- [Plugin Development Guide](plugin-development.md)
- [Integration Guide](integration.md)
- [API Reference](api-reference.md)

