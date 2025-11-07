/// Enhanced resource metrics for comprehensive plugin monitoring
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

/// Comprehensive resource metrics for a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(bincode::Encode, bincode::Decode)]
pub struct PluginMetrics {
    /// Plugin name
    pub plugin_name: String,
    
    /// Process ID
    pub pid: u32,
    
    /// Timestamp when metrics were collected
    pub timestamp: SystemTime,
    
    /// Memory metrics
    pub memory: MemoryMetrics,
    
    /// CPU metrics
    pub cpu: CpuMetrics,
    
    /// Disk I/O metrics
    pub disk_io: DiskIoMetrics,
    
    /// Network metrics
    pub network: NetworkMetrics,
    
    /// Process metrics
    pub process: ProcessMetrics,
    
    /// GPU metrics (if available)
    pub gpu: Option<GpuMetrics>,
    
    /// Uptime since plugin started
    pub uptime: Duration,
}

/// Memory usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(bincode::Encode, bincode::Decode)]
pub struct MemoryMetrics {
    /// Resident Set Size (RSS) - actual physical memory used
    pub rss_bytes: usize,
    
    /// Virtual Memory Size (VMS) - total virtual memory
    pub vms_bytes: usize,
    
    /// Shared memory
    pub shared_bytes: usize,
    
    /// Peak memory usage
    pub peak_rss_bytes: usize,
    
    /// Page faults (minor and major)
    pub page_faults_minor: u64,
    pub page_faults_major: u64,
}

/// CPU usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(bincode::Encode, bincode::Decode)]
pub struct CpuMetrics {
    /// CPU time in user mode (microseconds)
    pub user_time_us: u64,
    
    /// CPU time in kernel mode (microseconds)
    pub system_time_us: u64,
    
    /// Total CPU time
    pub total_time_us: u64,
    
    /// CPU usage percentage (0.0 - 100.0)
    pub usage_percent: f64,
    
    /// Number of context switches (voluntary)
    pub context_switches_voluntary: u64,
    
    /// Number of context switches (involuntary)
    pub context_switches_involuntary: u64,
}

/// Disk I/O metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(bincode::Encode, bincode::Decode)]
pub struct DiskIoMetrics {
    /// Bytes read from disk
    pub read_bytes: u64,
    
    /// Bytes written to disk
    pub write_bytes: u64,
    
    /// Number of read operations
    pub read_ops: u64,
    
    /// Number of write operations
    pub write_ops: u64,
    
    /// Read bandwidth (bytes/second)
    pub read_bps: u64,
    
    /// Write bandwidth (bytes/second)
    pub write_bps: u64,
}

/// Network I/O metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(bincode::Encode, bincode::Decode)]
pub struct NetworkMetrics {
    /// Bytes received
    pub rx_bytes: u64,
    
    /// Bytes transmitted
    pub tx_bytes: u64,
    
    /// Packets received
    pub rx_packets: u64,
    
    /// Packets transmitted
    pub tx_packets: u64,
    
    /// Receive bandwidth (bytes/second)
    pub rx_bps: u64,
    
    /// Transmit bandwidth (bytes/second)
    pub tx_bps: u64,
    
    /// Active connections
    pub active_connections: u32,
}

/// Process-level metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(bincode::Encode, bincode::Decode)]
pub struct ProcessMetrics {
    /// Number of threads
    pub thread_count: u32,
    
    /// Number of open file descriptors
    pub fd_count: u32,
    
    /// Process state (R=running, S=sleeping, D=disk sleep, Z=zombie, T=stopped)
    pub state: char,
    
    /// Nice value (-20 to 19)
    pub nice: i32,
    
    /// Number of children processes
    pub num_children: u32,
}

/// GPU usage metrics (if available)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(bincode::Encode, bincode::Decode)]
pub struct GpuMetrics {
    /// GPU device ID
    pub device_id: u32,
    
    /// GPU memory used (bytes)
    pub memory_used: u64,
    
    /// GPU memory total (bytes)
    pub memory_total: u64,
    
    /// GPU utilization percentage (0-100)
    pub utilization_percent: f64,
    
    /// GPU temperature (Celsius)
    pub temperature_celsius: Option<f64>,
    
    /// GPU power usage (watts)
    pub power_usage_watts: Option<f64>,
}

/// Termination reason when a plugin is killed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[derive(bincode::Encode, bincode::Decode)]
pub enum TerminationReason {
    /// Exceeded memory limit
    MemoryLimit { used: usize, limit: usize },
    
    /// Exceeded CPU time limit
    CpuLimit { used_ms: u64, limit_ms: u64 },
    
    /// Hook execution timeout
    HookTimeout { hook_name: String, duration_ms: u64, limit_ms: u64 },
    
    /// Exceeded file descriptor limit
    FileDescriptorLimit { used: u32, limit: u32 },
    
    /// Exceeded thread limit
    ThreadLimit { used: u32, limit: u32 },
    
    /// Exceeded network connection limit
    ConnectionLimit { used: u32, limit: u32 },
    
    /// Too many resource violations
    ViolationThreshold { count: usize, window_sec: u64 },
    
    /// Killed by cgroups OOM killer
    CgroupOomKill,
    
    /// Killed by seccomp (illegal syscall)
    SeccompViolation { syscall: Option<String> },
    
    /// Failed health check multiple times
    HealthCheckFailed { consecutive_failures: u32 },
    
    /// Graceful shutdown requested
    GracefulShutdown,
    
    /// Process crashed
    Crashed { exit_code: Option<i32>, signal: Option<i32> },
    
    /// Manual termination by operator
    Manual { reason: Option<String> },
}

impl TerminationReason {
    /// Get human-readable description
    pub fn description(&self) -> String {
        match self {
            Self::MemoryLimit { used, limit } => {
                format!("Memory limit exceeded: used {} MB, limit {} MB", 
                    used / (1024 * 1024), limit / (1024 * 1024))
            }
            Self::CpuLimit { used_ms, limit_ms } => {
                format!("CPU time limit exceeded: used {} ms, limit {} ms", used_ms, limit_ms)
            }
            Self::HookTimeout { hook_name, duration_ms, limit_ms } => {
                format!("Hook '{}' timeout: {} ms, limit {} ms", hook_name, duration_ms, limit_ms)
            }
            Self::FileDescriptorLimit { used, limit } => {
                format!("File descriptor limit exceeded: {} open, limit {}", used, limit)
            }
            Self::ThreadLimit { used, limit } => {
                format!("Thread limit exceeded: {} threads, limit {}", used, limit)
            }
            Self::ConnectionLimit { used, limit } => {
                format!("Connection limit exceeded: {} connections, limit {}", used, limit)
            }
            Self::ViolationThreshold { count, window_sec } => {
                format!("Too many violations: {} in {} seconds", count, window_sec)
            }
            Self::CgroupOomKill => {
                "Killed by cgroups OOM killer".to_string()
            }
            Self::SeccompViolation { syscall } => {
                match syscall {
                    Some(name) => format!("Seccomp violation: illegal syscall '{}'", name),
                    None => "Seccomp violation: illegal syscall".to_string(),
                }
            }
            Self::HealthCheckFailed { consecutive_failures } => {
                format!("Health check failed {} times consecutively", consecutive_failures)
            }
            Self::GracefulShutdown => {
                "Graceful shutdown".to_string()
            }
            Self::Crashed { exit_code, signal } => {
                match (exit_code, signal) {
                    (Some(code), None) => format!("Process crashed with exit code {}", code),
                    (None, Some(sig)) => format!("Process killed by signal {}", sig),
                    (Some(code), Some(sig)) => format!("Process crashed: exit code {}, signal {}", code, sig),
                    (None, None) => "Process crashed".to_string(),
                }
            }
            Self::Manual { reason } => {
                match reason {
                    Some(r) => format!("Manual termination: {}", r),
                    None => "Manual termination".to_string(),
                }
            }
        }
    }
    
    /// Check if this is a critical termination that should prevent restarts
    pub fn is_critical(&self) -> bool {
        matches!(self,
            Self::SeccompViolation { .. } |
            Self::ViolationThreshold { .. } |
            Self::Manual { .. }
        )
    }
    
    /// Check if this termination allows automatic restart
    pub fn allows_restart(&self) -> bool {
        matches!(self,
            Self::Crashed { .. } |
            Self::HealthCheckFailed { .. } |
            Self::CgroupOomKill
        )
    }
}

/// Termination event record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(bincode::Encode, bincode::Decode)]
pub struct TerminationEvent {
    /// Plugin name
    pub plugin_name: String,
    
    /// Process ID
    pub pid: u32,
    
    /// When the termination occurred
    pub timestamp: SystemTime,
    
    /// Why the plugin was terminated
    pub reason: TerminationReason,
    
    /// Final metrics before termination
    pub final_metrics: Option<PluginMetrics>,
    
    /// Plugin uptime before termination
    pub uptime: Duration,
    
    /// Number of restarts before this termination
    pub restart_count: u32,
}

impl TerminationEvent {
    pub fn new(
        plugin_name: String,
        pid: u32,
        reason: TerminationReason,
        final_metrics: Option<PluginMetrics>,
        uptime: Duration,
        restart_count: u32,
    ) -> Self {
        Self {
            plugin_name,
            pid,
            timestamp: SystemTime::now(),
            reason,
            final_metrics,
            uptime,
            restart_count,
        }
    }
}

impl Default for MemoryMetrics {
    fn default() -> Self {
        Self {
            rss_bytes: 0,
            vms_bytes: 0,
            shared_bytes: 0,
            peak_rss_bytes: 0,
            page_faults_minor: 0,
            page_faults_major: 0,
        }
    }
}

impl Default for CpuMetrics {
    fn default() -> Self {
        Self {
            user_time_us: 0,
            system_time_us: 0,
            total_time_us: 0,
            usage_percent: 0.0,
            context_switches_voluntary: 0,
            context_switches_involuntary: 0,
        }
    }
}

impl Default for DiskIoMetrics {
    fn default() -> Self {
        Self {
            read_bytes: 0,
            write_bytes: 0,
            read_ops: 0,
            write_ops: 0,
            read_bps: 0,
            write_bps: 0,
        }
    }
}

impl Default for NetworkMetrics {
    fn default() -> Self {
        Self {
            rx_bytes: 0,
            tx_bytes: 0,
            rx_packets: 0,
            tx_packets: 0,
            rx_bps: 0,
            tx_bps: 0,
            active_connections: 0,
        }
    }
}

impl Default for ProcessMetrics {
    fn default() -> Self {
        Self {
            thread_count: 1,
            fd_count: 0,
            state: 'S',
            nice: 0,
            num_children: 0,
        }
    }
}

