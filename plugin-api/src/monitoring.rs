/// Per-plugin resource monitoring using cgroups and /proc
use crate::{PluginError, ResourceLimits, limits::ViolationType};
use std::fs;
use std::path::Path;

/// Monitor resources for a plugin by PID
pub struct PluginResourceMonitor {
    plugin_name: String,
    pid: u32,
    limits: ResourceLimits,
}

impl PluginResourceMonitor {
    pub fn new(plugin_name: String, pid: u32, limits: ResourceLimits) -> Self {
        Self {
            plugin_name,
            pid,
            limits,
        }
    }
    
    /// Check all resource limits and return violations
    pub fn check_resources(&self) -> Result<Vec<ViolationType>, PluginError> {
        let mut violations = Vec::new();
        
        #[cfg(target_os = "linux")]
        {
            // Check memory via /proc/<pid>/status
            if let Ok(memory) = self.get_memory_usage() {
                if memory > self.limits.max_heap_bytes {
                    violations.push(ViolationType::HeapMemory {
                        used: memory,
                        limit: self.limits.max_heap_bytes,
                    });
                }
            }
            
            // Check threads
            if let Ok(threads) = self.get_thread_count() {
                if threads > self.limits.max_threads {
                    violations.push(ViolationType::Threads {
                        used: threads,
                        limit: self.limits.max_threads,
                    });
                }
            }
            
            // Check file descriptors
            if let Ok(fds) = self.get_fd_count() {
                if fds > self.limits.max_file_descriptors {
                    violations.push(ViolationType::FileDescriptors {
                        used: fds,
                        limit: self.limits.max_file_descriptors,
                    });
                }
            }
            
            // Check network connections
            if let Ok(conns) = self.get_connection_count() {
                if conns > self.limits.max_connections {
                    violations.push(ViolationType::Connections {
                        used: conns,
                        limit: self.limits.max_connections,
                    });
                }
            }
        }
        
        Ok(violations)
    }
    
    /// Get memory usage from /proc/<pid>/status (Linux only)
    #[cfg(target_os = "linux")]
    fn get_memory_usage(&self) -> Result<usize, PluginError> {
        let status_path = format!("/proc/{}/status", self.pid);
        let content = fs::read_to_string(&status_path)
            .map_err(|e| PluginError::LoadError(format!("Failed to read {}: {}", status_path, e)))?;
        
        for line in content.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let kb = parts[1].parse::<usize>()
                        .map_err(|e| PluginError::LoadError(format!("Failed to parse VmRSS: {}", e)))?;
                    return Ok(kb * 1024); // Convert KB to bytes
                }
            }
        }
        
        Err(PluginError::LoadError("VmRSS not found in status".to_string()))
    }
    
    /// Get thread count from /proc/<pid>/status (Linux only)
    #[cfg(target_os = "linux")]
    fn get_thread_count(&self) -> Result<u32, PluginError> {
        let status_path = format!("/proc/{}/status", self.pid);
        let content = fs::read_to_string(&status_path)
            .map_err(|e| PluginError::LoadError(format!("Failed to read {}: {}", status_path, e)))?;
        
        for line in content.lines() {
            if line.starts_with("Threads:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    return parts[1].parse::<u32>()
                        .map_err(|e| PluginError::LoadError(format!("Failed to parse Threads: {}", e)));
                }
            }
        }
        
        Err(PluginError::LoadError("Threads not found in status".to_string()))
    }
    
    /// Get file descriptor count from /proc/<pid>/fd (Linux only)
    #[cfg(target_os = "linux")]
    fn get_fd_count(&self) -> Result<u32, PluginError> {
        let fd_path = format!("/proc/{}/fd", self.pid);
        
        match fs::read_dir(&fd_path) {
            Ok(entries) => Ok(entries.count() as u32),
            Err(e) => Err(PluginError::LoadError(format!("Failed to read {}: {}", fd_path, e))),
        }
    }
    
    /// Get network connection count from /proc/net/tcp and /proc/net/tcp6 (Linux only)
    #[cfg(target_os = "linux")]
    fn get_connection_count(&self) -> Result<u32, PluginError> {
        // This is process-wide, not per-plugin perfect but good enough
        let mut count = 0;
        
        // Count TCP connections
        if let Ok(content) = fs::read_to_string("/proc/net/tcp") {
            count += content.lines().skip(1).count(); // Skip header
        }
        
        // Count TCP6 connections
        if let Ok(content) = fs::read_to_string("/proc/net/tcp6") {
            count += content.lines().skip(1).count(); // Skip header
        }
        
        Ok(count as u32)
    }
    
    /// Check if process still exists
    #[cfg(target_os = "linux")]
    pub fn process_exists(&self) -> bool {
        Path::new(&format!("/proc/{}", self.pid)).exists()
    }
    
    #[cfg(not(target_os = "linux"))]
    pub fn process_exists(&self) -> bool {
        // TODO: Implement for other platforms
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_monitor_creation() {
        let monitor = PluginResourceMonitor::new(
            "test".to_string(),
            std::process::id(),
            ResourceLimits::default(),
        );
        
        assert_eq!(monitor.plugin_name, "test");
    }
    
    #[test]
    #[cfg(target_os = "linux")]
    fn test_process_exists() {
        let monitor = PluginResourceMonitor::new(
            "test".to_string(),
            std::process::id(),
            ResourceLimits::default(),
        );
        
        assert!(monitor.process_exists());
    }
    
    #[test]
    #[cfg(target_os = "linux")]
    fn test_get_memory_usage() {
        let monitor = PluginResourceMonitor::new(
            "test".to_string(),
            std::process::id(),
            ResourceLimits::default(),
        );
        
        // Should succeed for current process
        let memory = monitor.get_memory_usage();
        assert!(memory.is_ok());
        assert!(memory.unwrap() > 0);
    }
}

