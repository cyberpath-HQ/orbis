# API Reference

Complete API reference for the Plugin API system.

## Table of Contents

- [Core Traits](#core-traits)
- [Plugin Registry](#plugin-registry)
- [Plugin Loader](#plugin-loader)
- [Plugin Security](#plugin-security)
- [Plugin Context](#plugin-context)
- [Hook Registry](#hook-registry)
- [Resource Management](#resource-management)
- [Monitoring](#monitoring)
- [Sandboxing (Linux)](#sandboxing-linux)
- [IPC System](#ipc-system)
- [Error Types](#error-types)

---

## Core Traits

### `Plugin` Trait

The main trait that all plugins must implement.

```rust
#[async_trait::async_trait(?Send)]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn author(&self) -> &str;
    fn description(&self) -> Option<&str>;
    fn resource_limits(&self) -> Option<ResourceLimits>;
    fn requirements(&self) -> PluginRequirements;
    
    async fn init(&mut self, context: *const ()) -> Result<(), PluginError>;
    async fn shutdown(&mut self) -> Result<(), PluginError>;
    async fn register_hooks(&self, hook_registry: *mut ()) -> Result<(), PluginError>;
}
```

**Methods:**

- **`name()`**: Returns the plugin's unique identifier
  - **Returns**: `&str` - Plugin name
  - **Thread-safe**: Yes

- **`version()`**: Returns the plugin's semantic version
  - **Returns**: `&str` - Version string (e.g., "1.0.0")
  - **Thread-safe**: Yes

- **`author()`**: Returns the plugin author information
  - **Returns**: `&str` - Author name/email
  - **Thread-safe**: Yes

- **`description()`**: Returns optional Markdown description
  - **Returns**: `Option<&str>` - Description or None
  - **Thread-safe**: Yes

- **`resource_limits()`**: Declares resource limits
  - **Returns**: `Option<ResourceLimits>` - Limits or None for defaults
  - **Thread-safe**: Yes

- **`requirements()`**: Declares plugin requirements
  - **Returns**: `PluginRequirements` - Network, filesystem, etc.
  - **Thread-safe**: Yes

- **`init(context)`**: Initialize the plugin
  - **Parameters**: `context: *const ()` - Opaque context pointer
  - **Returns**: `Result<(), PluginError>`
  - **Async**: Yes
  - **Called**: Once on plugin load

- **`shutdown()`**: Clean up the plugin
  - **Returns**: `Result<(), PluginError>`
  - **Async**: Yes
  - **Called**: Once on plugin unload

- **`register_hooks(hook_registry)`**: Register event hooks
  - **Parameters**: `hook_registry: *mut ()` - Opaque registry pointer
  - **Returns**: `Result<(), PluginError>`
  - **Async**: Yes
  - **Called**: Once after init

---

## Plugin Registry

### `PluginRegistry`

Central management for all plugins.

```rust
pub struct PluginRegistry {
    // Internal fields...
}
```

**Constructor:**

```rust
pub fn new(
    loader: Arc<PluginLoader>,
    context: Arc<PluginContext>,
    hook_registry: Arc<RwLock<HookRegistry>>,
    security: Arc<PluginSecurity>,
) -> Self
```

**Methods:**

#### Plugin Loading

- **`load_plugin<P: AsRef<Path>>(path, trust_level) -> Result<String, PluginError>`**
  - Load a plugin from a file path
  - **Parameters**:
    - `path`: Path to plugin file (.so, .dll, .dylib)
    - `trust_level`: Trust level (Trusted or Untrusted)
  - **Returns**: Plugin name on success
  - **Async**: Yes
  - **Thread-safe**: Yes
  
  ```rust
  let name = registry.load_plugin(
      "/path/to/plugin.so",
      TrustLevel::Trusted,
  ).await?;
  ```

- **`load_plugin_by_name(name: &str) -> Result<(), PluginError>`**
  - Load a previously discovered plugin by name
  - **Parameters**: `name` - Plugin name
  - **Returns**: Unit on success
  - **Async**: Yes
  - **Thread-safe**: Yes
  
  ```rust
  registry.load_plugin_by_name("my_plugin").await?;
  ```

- **`unload_plugin(name: &str) -> Result<(), PluginError>`**
  - Unload a plugin by name
  - **Parameters**: `name` - Plugin name
  - **Returns**: Unit on success
  - **Async**: Yes (calls plugin's shutdown)
  - **Thread-safe**: Yes
  
  ```rust
  registry.unload_plugin("my_plugin").await?;
  ```

- **`unload_all() -> Result<(), PluginError>`**
  - Unload all loaded plugins
  - **Returns**: Unit on success
  - **Async**: Yes
  - **Thread-safe**: Yes
  
  ```rust
  registry.unload_all().await?;
  ```

#### Plugin Discovery

- **`scan_directory<P: AsRef<Path>>(path) -> Result<usize, PluginError>`**
  - Scan a directory for plugins
  - **Parameters**: `path` - Directory path
  - **Returns**: Number of plugins discovered
  - **Thread-safe**: Yes
  
  ```rust
  let count = registry.scan_directory("/plugins")?;
  ```

- **`discover_plugin<P: AsRef<Path>>(path) -> Result<(), PluginError>`**
  - Discover a single plugin without loading it
  - **Parameters**: `path` - Plugin file path
  - **Returns**: Unit on success
  - **Thread-safe**: Yes

#### Plugin Queries

- **`get_all_plugins() -> Result<Vec<PluginInfo>, PluginError>`**
  - Get all tracked plugins (loaded + available)
  - **Returns**: Vector of plugin info
  - **Thread-safe**: Yes
  
  ```rust
  let all = registry.get_all_plugins()?;
  ```

- **`list_loaded_plugins() -> Result<Vec<PluginInfo>, PluginError>`**
  - Get only loaded plugins
  - **Returns**: Vector of plugin info
  - **Thread-safe**: Yes

- **`get_available_plugins() -> Result<Vec<PluginInfo>, PluginError>`**
  - Get available (unloaded but trusted) plugins
  - **Returns**: Vector of plugin info
  - **Thread-safe**: Yes

- **`get_untrusted_plugins() -> Result<Vec<PluginInfo>, PluginError>`**
  - Get untrusted plugins
  - **Returns**: Vector of plugin info
  - **Thread-safe**: Yes

- **`get_plugin_info(name: &str) -> Result<PluginInfo, PluginError>`**
  - Get information about a specific plugin
  - **Parameters**: `name` - Plugin name
  - **Returns**: Plugin info
  - **Thread-safe**: Yes

- **`is_loaded(name: &str) -> bool`**
  - Check if a plugin is loaded
  - **Parameters**: `name` - Plugin name
  - **Returns**: True if loaded
  - **Thread-safe**: Yes

- **`plugin_count() -> usize`**
  - Get number of loaded plugins
  - **Returns**: Count
  - **Thread-safe**: Yes

- **`total_plugin_count() -> usize`**
  - Get total number of tracked plugins
  - **Returns**: Count
  - **Thread-safe**: Yes

#### Resource Management

- **`get_resource_limits(name: &str) -> Result<ResourceLimits, PluginError>`**
  - Get resource limits for a plugin
  - **Parameters**: `name` - Plugin name
  - **Returns**: Resource limits
  - **Thread-safe**: Yes

- **`record_violation(name: &str, violation: ViolationType) -> Result<bool, PluginError>`**
  - Record a resource violation
  - **Parameters**:
    - `name` - Plugin name
    - `violation` - Violation type
  - **Returns**: True if plugin should be unmounted
  - **Thread-safe**: Yes

- **`get_violation_count(name: &str) -> Result<usize, PluginError>`**
  - Get violation count for a plugin
  - **Parameters**: `name` - Plugin name
  - **Returns**: Violation count
  - **Thread-safe**: Yes

- **`reset_violations(name: &str) -> Result<(), PluginError>`**
  - Reset violation tracker
  - **Parameters**: `name` - Plugin name
  - **Returns**: Unit on success
  - **Thread-safe**: Yes

- **`set_unmount_behavior(name: &str, behavior: UnmountBehavior) -> Result<(), PluginError>`**
  - Configure unmount behavior
  - **Parameters**:
    - `name` - Plugin name
    - `behavior` - Unmount behavior
  - **Returns**: Unit on success
  - **Thread-safe**: Yes

#### Monitoring

- **`start_resource_monitor(check_interval: Option<Duration>) -> JoinHandle<()>`**
  - Start background resource monitoring
  - **Parameters**: `check_interval` - Check frequency (default: 10s)
  - **Returns**: Tokio join handle
  - **Thread-safe**: Yes
  - **Note**: Must call on `Arc<PluginRegistry>`
  
  ```rust
  let monitor = registry.clone().start_resource_monitor(None);
  // Later: monitor.abort();
  ```

---

## Plugin Loader

### `PluginLoader`

Handles dynamic library loading.

```rust
pub struct PluginLoader {
    // Internal fields...
}
```

**Constructor:**

```rust
pub fn new(security: Arc<PluginSecurity>) -> Self
```

**Methods:**

- **`load<P: AsRef<Path>>(path, trust_level) -> Result<(Box<PluginBridge>, Library), PluginError>`**
  - Load a plugin from file
  - **Parameters**:
    - `path` - Plugin file path
    - `trust_level` - Trust level
  - **Returns**: Tuple of (plugin bridge, library handle)
  - **Thread-safe**: Yes
  - **Note**: Library must be kept alive

---

## Plugin Security

### `PluginSecurity`

Manages cryptographic verification and trust.

```rust
pub struct PluginSecurity {
    // Internal fields...
}
```

**Constructor:**

```rust
pub fn new(
    policy: SecurityPolicy,
    hardcoded_public_keys: Vec<PublicKey>,
    trusted_plugins: Vec<TrustedPluginEntry>,
) -> Self
```

**Methods:**

#### Verification

- **`verify_plugin<P: AsRef<Path>>(path) -> Result<bool, PluginError>`**
  - Verify plugin signature and hash
  - **Parameters**: `path` - Plugin file path
  - **Returns**: True if verified
  - **Thread-safe**: Yes

- **`calculate_hash<P: AsRef<Path>>(path) -> Result<String, PluginError>`**
  - Calculate SHA3-512 hash of plugin
  - **Parameters**: `path` - Plugin file path
  - **Returns**: Hex-encoded hash
  - **Thread-safe**: Yes

- **`verify_signature(message, signature, public_key) -> Result<bool, PluginError>`**
  - Verify Ed25519 signature
  - **Parameters**:
    - `message` - Message bytes
    - `signature` - Signature bytes
    - `public_key` - Public key
  - **Returns**: True if valid
  - **Thread-safe**: Yes

#### Trust Management

- **`is_trusted_hash(hash: &str) -> Result<bool, PluginError>`**
  - Check if hash is in trust list
  - **Parameters**: `hash` - SHA3-512 hash (hex)
  - **Returns**: True if trusted
  - **Thread-safe**: Yes

- **`get_plugin_info(hash: &str) -> Result<Option<TrustedPluginEntry>, PluginError>`**
  - Get trust entry for hash
  - **Parameters**: `hash` - SHA3-512 hash
  - **Returns**: Trust entry or None
  - **Thread-safe**: Yes

- **`add_trusted_plugin(entry: TrustedPluginEntry) -> Result<(), PluginError>`**
  - Add plugin to trust list
  - **Parameters**: `entry` - Trust entry
  - **Returns**: Unit on success
  - **Thread-safe**: Yes

- **`remove_trusted_plugin(hash: &str) -> Result<(), PluginError>`**
  - Remove plugin from trust list
  - **Parameters**: `hash` - SHA3-512 hash
  - **Returns**: Unit on success
  - **Thread-safe**: Yes

#### Encryption

- **`encrypt_and_save_trust_list(key_hex: &str) -> Result<(), PluginError>`**
  - Encrypt and save trust list to disk
  - **Parameters**: `key_hex` - 32-byte hex-encoded key
  - **Returns**: Unit on success
  - **Thread-safe**: Yes

- **`load_and_decrypt_trust_list(key_hex: &str) -> Result<Vec<TrustedPluginEntry>, PluginError>`**
  - Load and decrypt trust list from disk
  - **Parameters**: `key_hex` - 32-byte hex-encoded key
  - **Returns**: Vector of trust entries
  - **Thread-safe**: Yes

### `SecurityPolicy`

Security configuration.

```rust
pub struct SecurityPolicy {
    pub only_trusted: bool,
    pub trust_list_path: Option<PathBuf>,
}
```

**Default:**
```rust
SecurityPolicy {
    only_trusted: true,
    trust_list_path: Some("data/plugin_trust_list.enc".into()),
}
```

### `TrustedPluginEntry`

Trust list entry.

```rust
pub struct TrustedPluginEntry {
    pub hash: String,
    pub version: PluginVersion,
    pub signature: PluginSignature,
    pub note: Option<String>,
}
```

### `PluginVersion`

Semantic version.

```rust
pub struct PluginVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}
```

**Methods:**

- **`new(major, minor, patch) -> Self`**
- **`from_string(s: &str) -> Result<Self, PluginError>`**
- **`to_string() -> String`**

---

## Plugin Context

### `PluginContext`

Shared data store.

```rust
pub struct PluginContext {
    // Internal fields...
}
```

**Constructor:**

```rust
pub fn new() -> Self
```

**Methods:**

- **`set<T: Send + Sync + 'static>(key: ContextKey, value: Arc<T>) -> Result<(), PluginError>`**
  - Store a value
  - **Parameters**:
    - `key` - Context key
    - `value` - Value to store (Arc-wrapped)
  - **Returns**: Unit on success
  - **Thread-safe**: Yes
  
  ```rust
  context.set(
      ContextKey::Predefined(PredefinedContextKey::DatabaseConnection),
      Arc::new(db),
  )?;
  ```

- **`get<T: Send + Sync + 'static>(key: ContextKey) -> Result<Arc<T>, PluginError>`**
  - Retrieve a value
  - **Parameters**: `key` - Context key
  - **Returns**: Arc to value
  - **Thread-safe**: Yes
  
  ```rust
  let db = context.get::<DatabaseConnection>(
      ContextKey::Predefined(PredefinedContextKey::DatabaseConnection)
  )?;
  ```

- **`remove(key: ContextKey) -> Result<(), PluginError>`**
  - Remove a value
  - **Parameters**: `key` - Context key
  - **Returns**: Unit on success
  - **Thread-safe**: Yes

### `ContextKey`

Key for context storage.

```rust
pub enum ContextKey {
    Predefined(PredefinedContextKey),
    Custom(String),
}
```

### `PredefinedContextKey`

Predefined keys.

```rust
pub enum PredefinedContextKey {
    DatabaseConnection,
    Configuration,
    Logger,
    HttpClient,
    CacheManager,
}
```

---

## Hook Registry

### `HookRegistry`

Event hook management.

```rust
pub struct HookRegistry {
    // Internal fields...
}
```

**Constructor:**

```rust
pub fn new() -> Self
```

**Methods:**

- **`register(name: &str, priority: u32, handler: HookHandler)`**
  - Register a hook handler
  - **Parameters**:
    - `name` - Hook name
    - `priority` - Execution priority (lower = earlier)
    - `handler` - Handler function
  - **Thread-safe**: Yes (with write lock)
  
  ```rust
  registry.register(
      "on_request",
      200,
      Box::new(|data| Box::pin(async move {
          // Handle hook
          Ok(data)
      })),
  );
  ```

- **`execute(name: &str, data: Vec<u8>) -> Result<Vec<u8>, PluginError>`**
  - Execute all handlers for a hook
  - **Parameters**:
    - `name` - Hook name
    - `data` - Input data
  - **Returns**: Modified data
  - **Async**: Yes
  - **Thread-safe**: Yes (with read lock)

- **`get_hooks(name: &str) -> Vec<(u32, HookHandler)>`**
  - Get all handlers for a hook
  - **Parameters**: `name` - Hook name
  - **Returns**: Vector of (priority, handler) tuples
  - **Thread-safe**: Yes (with read lock)

### `HookHandler`

Hook handler type.

```rust
pub type HookHandler = Box<
    dyn Fn(Vec<u8>) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, PluginError>>>>
    + Send
    + Sync
>;
```

---

## Resource Management

### `ResourceLimits`

Resource limit configuration.

```rust
pub struct ResourceLimits {
    pub max_heap_bytes: usize,
    pub max_cpu_time_ms: u64,
    pub max_threads: u32,
    pub max_file_descriptors: u32,
    pub max_connections: u32,
}
```

**Default:**
```rust
ResourceLimits {
    max_heap_bytes: 50 * 1024 * 1024,  // 50 MB
    max_cpu_time_ms: 1000,              // 1 second
    max_threads: 4,
    max_file_descriptors: 32,
    max_connections: 10,
}
```

**Methods:**

- **`validate() -> Result<(), PluginError>`**
  - Validate limits are reasonable
  - **Returns**: Unit on success

### `ViolationType`

Resource violation types.

```rust
pub enum ViolationType {
    HeapMemory { used: usize, limit: usize },
    CpuTime { used_ms: u64, limit_ms: u64 },
    Threads { used: u32, limit: u32 },
    FileDescriptors { used: u32, limit: u32 },
    Connections { used: u32, limit: u32 },
}
```

### `ViolationTracker`

Tracks violations.

```rust
pub struct ViolationTracker {
    violations: Vec<(ViolationType, SystemTime)>,
    violation_threshold: usize,
}
```

**Methods:**

- **`record_violation(violation: ViolationType)`**
- **`violation_count() -> usize`**
- **`should_unmount() -> bool`**
- **`reset()`**

### `UnmountBehavior`

Unmount configuration.

```rust
pub struct UnmountBehavior {
    pub auto_unmount: bool,
    pub log_violations: bool,
}
```

**Default:**
```rust
UnmountBehavior {
    auto_unmount: true,
    log_violations: true,
}
```

---

## Monitoring

### `PluginMetrics`

Resource usage metrics.

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

## Sandboxing (Linux)

### `SandboxConfig`

Sandbox configuration.

```rust
pub struct SandboxConfig {
    pub enable_pid_namespace: bool,
    pub enable_network_namespace: bool,
    pub enable_mount_namespace: bool,
    pub enable_ipc_namespace: bool,
    pub enable_uts_namespace: bool,
    pub enable_user_namespace: bool,
    pub cgroup_config: Option<CgroupConfig>,
    pub seccomp_mode: SeccompMode,
    pub capabilities_config: CapabilitiesConfig,
    pub filesystem_config: FilesystemConfig,
    pub network_config: NetworkConfig,
}
```

### `CgroupConfig`

Cgroup limits.

```rust
pub struct CgroupConfig {
    pub memory_limit_bytes: Option<usize>,
    pub cpu_quota_us: Option<u64>,
    pub cpu_period_us: u64,
    pub pids_limit: Option<u32>,
    pub io_weight: Option<u16>,
}
```

### `SeccompMode`

Seccomp filtering modes.

```rust
pub enum SeccompMode {
    Disabled,
    Strict,
    Basic,
    Moderate,
    Permissive,
    Custom(SeccompFilter),
}
```

### `CapabilitiesConfig`

Capability configuration.

```rust
pub struct CapabilitiesConfig {
    pub drop_all: bool,
    pub allowed_caps: Vec<Capability>,
    pub no_new_privs: bool,
}
```

---

## IPC System

### `IpcChannel`

IPC communication channel.

```rust
pub struct IpcChannel {
    // Platform-specific
}
```

**Methods:**

- **`send_request(request: IpcRequest) -> Result<IpcResponse, PluginError>`**
  - Send request and wait for response
  - **Async**: Yes

- **`receive_request() -> Result<IpcRequest, PluginError>`**
  - Receive a request
  - **Async**: Yes

- **`send_response(response: IpcResponse) -> Result<(), PluginError>`**
  - Send a response
  - **Async**: Yes

### `IpcRequest`

Request message types.

```rust
pub enum IpcRequest {
    Initialize { config: PluginConfig },
    Shutdown,
    ExecuteHook { hook_name: String, data: Vec<u8> },
    GetMetrics,
}
```

### `IpcResponse`

Response message types.

```rust
pub enum IpcResponse {
    Success,
    Error { message: String },
    Metrics { data: PluginMetrics },
    HookResult { data: Vec<u8> },
}
```

---

## Error Types

### `PluginError`

Main error type.

```rust
pub enum PluginError {
    LoadError(String),
    InitializationError(String),
    HookError(String),
    SecurityError(String),
    UntrustedPlugin,
    AlreadyLoaded(String),
    NotFound(String),
    ContextError(String),
    InvalidResourceLimits(String),
    ResourceViolation(ViolationType),
    IpcError(String),
    IoError(std::io::Error),
}
```

**Methods:**

- **`to_string() -> String`** - Get error message

**Conversions:**

- `From<std::io::Error>`
- `From<libloading::Error>`
- `From<bincode::Error>`

---

## Constants

### Symbol Names

```rust
pub const PLUGIN_CONSTRUCTOR_SYMBOL: &str = "create_plugin";
pub const PLUGIN_SIGNATURE_SYMBOL: &str = "plugin_signature";
pub const PLUGIN_HASH_SYMBOL: &str = "plugin_hash";
```

### Default Values

```rust
// Resource limits
pub const DEFAULT_MAX_HEAP_BYTES: usize = 50 * 1024 * 1024;  // 50 MB
pub const DEFAULT_MAX_CPU_TIME_MS: u64 = 1000;                // 1 second
pub const DEFAULT_MAX_THREADS: u32 = 4;
pub const DEFAULT_MAX_FILE_DESCRIPTORS: u32 = 32;
pub const DEFAULT_MAX_CONNECTIONS: u32 = 10;

// Violation tracking
pub const DEFAULT_VIOLATION_THRESHOLD: usize = 10;

// Monitoring
pub const DEFAULT_MONITOR_INTERVAL_SECS: u64 = 10;
```

---

## Type Aliases

```rust
pub type PluginConstructor = unsafe extern "C" fn() -> *mut dyn Plugin;
```

---

For more information:
- [Features Documentation](features.md)
- [Security Guide](security.md)
- [Plugin Development Guide](plugin-development.md)
- [Integration Guide](integration.md)

