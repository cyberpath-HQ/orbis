/// Process management for plugin sandboxing
/// 
/// This module provides the PluginProcessManager which orchestrates
/// plugin worker processes, managing their lifecycle, health, and communication.

pub mod manager;
pub mod process;

pub use manager::PluginProcessManager;
pub use process::{PluginProcess, ProcessStatus};

use std::path::PathBuf;

/// Configuration for plugin process management
#[derive(Debug, Clone)]
pub struct ProcessConfig {
    /// Path to the plugin-worker binary
    pub worker_binary: PathBuf,
    
    /// Maximum number of restart attempts before giving up
    pub max_restart_attempts: u32,
    
    /// Interval between health checks in milliseconds
    pub health_check_interval_ms: u64,
    
    /// Timeout for plugin startup in milliseconds
    pub startup_timeout_ms: u64,
    
    /// Grace period for shutdown in milliseconds
    pub shutdown_grace_period_ms: u64,
}

impl Default for ProcessConfig {
    fn default() -> Self {
        Self {
            worker_binary: PathBuf::from("target/debug/plugin-worker"),
            max_restart_attempts: 3,
            health_check_interval_ms: 10000, // 10 seconds
            startup_timeout_ms: 30000, // 30 seconds
            shutdown_grace_period_ms: 5000, // 5 seconds
        }
    }
}

