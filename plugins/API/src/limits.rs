/// Resource limits and timeout management for plugins
use serde::{Deserialize, Serialize};
use std::time::Duration;
use crate::PluginError;

/// Resource limits that a plugin must declare or defaults will be used
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum heap memory in bytes (default: 512MB)
    pub max_heap_bytes: usize,
    
    /// Maximum CPU time per operation in milliseconds (default: 5000ms)
    pub max_cpu_time_ms: u64,
    
    /// Maximum number of open file descriptors (default: 100)
    pub max_file_descriptors: u32,
    
    /// Maximum number of threads (default: 10)
    pub max_threads: u32,
    
    /// Maximum network connections (default: 50)
    pub max_connections: u32,
    
    /// Maximum duration for database queries in milliseconds (default: 10000ms)
    pub max_db_query_ms: u64,
    
    /// Maximum duration for external API calls in milliseconds (default: 30000ms)
    pub max_external_api_ms: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_heap_bytes: 512 * 1024 * 1024,     // 512MB
            max_cpu_time_ms: 5000,                  // 5 seconds
            max_file_descriptors: 100,
            max_threads: 10,
            max_connections: 50,
            max_db_query_ms: 10_000,                // 10 seconds
            max_external_api_ms: 30_000,            // 30 seconds
        }
    }
}

impl ResourceLimits {
    /// Create custom resource limits
    pub fn new(
        max_heap_bytes: usize,
        max_cpu_time_ms: u64,
        max_file_descriptors: u32,
        max_threads: u32,
        max_connections: u32,
        max_db_query_ms: u64,
        max_external_api_ms: u64,
    ) -> Self {
        Self {
            max_heap_bytes,
            max_cpu_time_ms,
            max_file_descriptors,
            max_threads,
            max_connections,
            max_db_query_ms,
            max_external_api_ms,
        }
    }
    
    /// Validate that limits are reasonable
    pub fn validate(&self) -> Result<(), PluginError> {
        if self.max_heap_bytes == 0 {
            return Err(PluginError::ResourceLimitError("max_heap_bytes cannot be 0".to_string()));
        }
        if self.max_cpu_time_ms == 0 {
            return Err(PluginError::ResourceLimitError("max_cpu_time_ms cannot be 0".to_string()));
        }
        if self.max_heap_bytes > 4 * 1024 * 1024 * 1024 {  // 4GB max
            return Err(PluginError::ResourceLimitError("max_heap_bytes exceeds 4GB".to_string()));
        }
        Ok(())
    }
}

/// Hook execution timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookTimeout {
    /// Hook name pattern (e.g., "before_request", "*" for all)
    pub hook_pattern: String,
    
    /// Timeout duration in milliseconds
    pub timeout_ms: u64,
    
    /// Whether to unmount plugin on timeout
    pub unmount_on_timeout: bool,
}

impl HookTimeout {
    pub fn new(hook_pattern: String, timeout_ms: u64, unmount_on_timeout: bool) -> Self {
        Self {
            hook_pattern,
            timeout_ms,
            unmount_on_timeout,
        }
    }
    
    pub fn as_duration(&self) -> Duration {
        Duration::from_millis(self.timeout_ms)
    }
}

impl Default for HookTimeout {
    fn default() -> Self {
        Self {
            hook_pattern: "*".to_string(),  // All hooks
            timeout_ms: 5000,               // 5 seconds default
            unmount_on_timeout: false,      // Don't unmount by default
        }
    }
}

/// Violation tracking for automatic plugin unmounting
#[derive(Debug, Clone)]
pub struct ViolationTracker {
    /// Violations in the current time window
    violations: Vec<Violation>,
    
    /// Time window for tracking violations (default: 60 seconds)
    window_duration: Duration,
    
    /// Maximum violations before unmount (default: 5)
    max_violations: usize,
}

/// Type of resource violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationType {
    HeapMemory { used: usize, limit: usize },
    CpuTime { used_ms: u64, limit_ms: u64 },
    FileDescriptors { used: u32, limit: u32 },
    Threads { used: u32, limit: u32 },
    Connections { used: u32, limit: u32 },
    DatabaseQuery { duration_ms: u64, limit_ms: u64 },
    ExternalApi { duration_ms: u64, limit_ms: u64 },
    HookTimeout { hook_name: String, duration_ms: u64, limit_ms: u64 },
}

impl ViolationType {
    pub fn severity(&self) -> ViolationSeverity {
        match self {
            ViolationType::HeapMemory { .. } => ViolationSeverity::Critical,
            ViolationType::CpuTime { .. } => ViolationSeverity::High,
            ViolationType::HookTimeout { .. } => ViolationSeverity::High,
            ViolationType::DatabaseQuery { .. } => ViolationSeverity::Medium,
            ViolationType::ExternalApi { .. } => ViolationSeverity::Medium,
            _ => ViolationSeverity::Low,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ViolationSeverity {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// Individual violation record
#[derive(Debug, Clone)]
pub struct Violation {
    pub violation_type: ViolationType,
    pub timestamp: std::time::Instant,
    pub severity: ViolationSeverity,
}

impl ViolationTracker {
    pub fn new(window_duration: Duration, max_violations: usize) -> Self {
        Self {
            violations: Vec::new(),
            window_duration,
            max_violations,
        }
    }
    
    /// Record a violation
    pub fn record_violation(&mut self, violation_type: ViolationType) {
        let severity = violation_type.severity();
        self.violations.push(Violation {
            violation_type,
            timestamp: std::time::Instant::now(),
            severity,
        });
        
        // Clean old violations outside the window
        self.clean_old_violations();
    }
    
    /// Check if plugin should be unmounted due to violations
    pub fn should_unmount(&mut self) -> bool {
        self.clean_old_violations();
        
        // Check total violations
        if self.violations.len() >= self.max_violations {
            return true;
        }
        
        // Check critical violations (unmount immediately)
        if self.violations.iter().any(|v| v.severity == ViolationSeverity::Critical) {
            return true;
        }
        
        // Check multiple high-severity violations
        let high_severity_count = self.violations.iter()
            .filter(|v| v.severity >= ViolationSeverity::High)
            .count();
        
        if high_severity_count >= 3 {
            return true;
        }
        
        false
    }
    
    /// Remove violations outside the time window
    fn clean_old_violations(&mut self) {
        let now = std::time::Instant::now();
        self.violations.retain(|v| now.duration_since(v.timestamp) < self.window_duration);
    }
    
    /// Get current violation count
    pub fn violation_count(&mut self) -> usize {
        self.clean_old_violations();
        self.violations.len()
    }
    
    /// Reset violation tracker
    pub fn reset(&mut self) {
        self.violations.clear();
    }
}

impl Default for ViolationTracker {
    fn default() -> Self {
        Self::new(Duration::from_secs(60), 5)
    }
}

/// Unmount behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnmountBehavior {
    /// Automatically unmount on violation threshold
    pub auto_unmount: bool,
    
    /// Grace period before unmount in milliseconds
    pub grace_period_ms: u64,
    
    /// Allow plugin to handle cleanup before unmount
    pub allow_cleanup: bool,
    
    /// Maximum cleanup time in milliseconds
    pub max_cleanup_time_ms: u64,
    
    /// Log violations before unmounting
    pub log_violations: bool,
    
    /// Call plugin shutdown hook before unmount
    pub call_shutdown_hook: bool,
}

impl Default for UnmountBehavior {
    fn default() -> Self {
        Self {
            auto_unmount: true,
            grace_period_ms: 1000,          // 1 second grace
            allow_cleanup: true,
            max_cleanup_time_ms: 5000,      // 5 seconds max cleanup
            log_violations: true,
            call_shutdown_hook: true,
        }
    }
}

impl UnmountBehavior {
    pub fn immediate() -> Self {
        Self {
            auto_unmount: true,
            grace_period_ms: 0,
            allow_cleanup: false,
            max_cleanup_time_ms: 0,
            log_violations: true,
            call_shutdown_hook: false,
        }
    }
    
    pub fn graceful() -> Self {
        Self {
            auto_unmount: true,
            grace_period_ms: 5000,
            allow_cleanup: true,
            max_cleanup_time_ms: 10000,
            log_violations: true,
            call_shutdown_hook: true,
        }
    }
    
    pub fn manual() -> Self {
        Self {
            auto_unmount: false,
            grace_period_ms: 0,
            allow_cleanup: true,
            max_cleanup_time_ms: 30000,
            log_violations: true,
            call_shutdown_hook: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_limits() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.max_heap_bytes, 512 * 1024 * 1024);
        assert_eq!(limits.max_cpu_time_ms, 5000);
    }
    
    #[test]
    fn test_violation_tracker() {
        let mut tracker = ViolationTracker::new(Duration::from_secs(1), 3);
        
        assert!(!tracker.should_unmount());
        
        tracker.record_violation(ViolationType::CpuTime { used_ms: 6000, limit_ms: 5000 });
        assert!(!tracker.should_unmount());
        
        tracker.record_violation(ViolationType::CpuTime { used_ms: 6000, limit_ms: 5000 });
        tracker.record_violation(ViolationType::CpuTime { used_ms: 6000, limit_ms: 5000 });
        
        assert!(tracker.should_unmount());
    }
    
    #[test]
    fn test_critical_violation_immediate_unmount() {
        let mut tracker = ViolationTracker::new(Duration::from_secs(60), 10);
        
        tracker.record_violation(ViolationType::HeapMemory { used: 1024, limit: 512 });
        
        assert!(tracker.should_unmount());
    }
}

