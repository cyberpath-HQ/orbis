/// Enhanced resource monitoring with comprehensive metrics collection
/// 
/// This module provides comprehensive resource monitoring for plugins including:
/// - Memory (RSS, VMS, shared, peak, page faults)
/// - CPU (user/system time, usage %, context switches)
/// - Disk I/O (read/write bytes/ops, bandwidth)
/// - Network (rx/tx bytes/packets, bandwidth, connections)
/// - Process (threads, file descriptors, state, children)
/// - GPU (planned - memory, utilization, temperature, power)

use crate::{PluginError, ResourceLimits};
use crate::metrics::*;
use crate::limits::ViolationType;

#[cfg(target_os = "linux")]
use std::fs;
#[cfg(target_os = "linux")]
use std::path::Path;
#[cfg(target_os = "linux")]
use std::time::{Duration, SystemTime, Instant};
#[cfg(target_os = "linux")]
use tracing::{debug, warn};

/// Enhanced resource monitor with comprehensive metrics collection
pub struct EnhancedResourceMonitor {
    plugin_name: String,
    pid: u32,
    limits: ResourceLimits,
    start_time: Instant,
    
    // Previous values for bandwidth calculation
    prev_disk_read: u64,
    prev_disk_write: u64,
    prev_net_rx: u64,
    prev_net_tx: u64,
    prev_cpu_time: u64,
    prev_measurement_time: Instant,
}

#[cfg(target_os = "linux")]
impl EnhancedResourceMonitor {
    pub fn new(plugin_name: String, pid: u32, limits: ResourceLimits) -> Self {
        Self {
            plugin_name,
            pid,
            limits,
            start_time: Instant::now(),
            prev_disk_read: 0,
            prev_disk_write: 0,
            prev_net_rx: 0,
            prev_net_tx: 0,
            prev_cpu_time: 0,
            prev_measurement_time: Instant::now(),
        }
    }
    
    /// Collect comprehensive metrics
    pub fn collect_metrics(&mut self) -> Result<PluginMetrics, PluginError> {
        let timestamp = SystemTime::now();
        let uptime = self.start_time.elapsed();
        
        Ok(PluginMetrics {
            plugin_name: self.plugin_name.clone(),
            pid: self.pid,
            timestamp,
            memory: self.collect_memory_metrics()?,
            cpu: self.collect_cpu_metrics()?,
            disk_io: self.collect_disk_io_metrics()?,
            network: self.collect_network_metrics()?,
            process: self.collect_process_metrics()?,
            gpu: self.collect_gpu_metrics().ok(),
            uptime,
        })
    }
    
    /// Check resource violations
    pub fn check_violations(&self) -> Result<Vec<ViolationType>, PluginError> {
        let mut violations = Vec::new();
        
        // Check memory
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
        
        Ok(violations)
    }
    
    /// Collect memory metrics from /proc/[pid]/status and /proc/[pid]/statm
    fn collect_memory_metrics(&self) -> Result<MemoryMetrics, PluginError> {
        let status_path = format!("/proc/{}/status", self.pid);
        let statm_path = format!("/proc/{}/statm", self.pid);
        
        let status = fs::read_to_string(&status_path)
            .map_err(|e| PluginError::LoadError(format!("Failed to read {}: {}", status_path, e)))?;
        
        let statm = fs::read_to_string(&statm_path)
            .map_err(|e| PluginError::LoadError(format!("Failed to read {}: {}", statm_path, e)))?;
        
        let mut metrics = MemoryMetrics::default();
        
        // Parse /proc/[pid]/status
        for line in status.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }
            
            match parts[0] {
                "VmRSS:" => {
                    if let Ok(kb) = parts[1].parse::<usize>() {
                        metrics.rss_bytes = kb * 1024;
                    }
                }
                "VmSize:" => {
                    if let Ok(kb) = parts[1].parse::<usize>() {
                        metrics.vms_bytes = kb * 1024;
                    }
                }
                "RssFile:" | "RssShmem:" => {
                    if let Ok(kb) = parts[1].parse::<usize>() {
                        metrics.shared_bytes += kb * 1024;
                    }
                }
                "VmHWM:" => {
                    if let Ok(kb) = parts[1].parse::<usize>() {
                        metrics.peak_rss_bytes = kb * 1024;
                    }
                }
                _ => {}
            }
        }
        
        // Parse /proc/[pid]/statm for additional info
        let statm_parts: Vec<&str> = statm.trim().split_whitespace().collect();
        if statm_parts.len() >= 3 {
            // Pages are typically 4096 bytes
            let page_size = 4096;
            if let Ok(shared_pages) = statm_parts[2].parse::<usize>() {
                metrics.shared_bytes = shared_pages * page_size;
            }
        }
        
        // Get page faults from /proc/[pid]/stat
        if let Ok(page_faults) = self.get_page_faults() {
            metrics.page_faults_minor = page_faults.0;
            metrics.page_faults_major = page_faults.1;
        }
        
        Ok(metrics)
    }
    
    /// Collect CPU metrics from /proc/[pid]/stat
    fn collect_cpu_metrics(&mut self) -> Result<CpuMetrics, PluginError> {
        let stat_path = format!("/proc/{}/stat", self.pid);
        let stat = fs::read_to_string(&stat_path)
            .map_err(|e| PluginError::LoadError(format!("Failed to read {}: {}", stat_path, e)))?;
        
        // Parse stat file (format is complex, fields are space-separated)
        let parts: Vec<&str> = stat.split_whitespace().collect();
        if parts.len() < 52 {
            return Err(PluginError::LoadError("Invalid /proc/[pid]/stat format".to_string()));
        }
        
        // Fields: utime (14), stime (15), cutime (16), cstime (17)
        let utime = parts[13].parse::<u64>().unwrap_or(0);
        let stime = parts[14].parse::<u64>().unwrap_or(0);
        
        // Convert clock ticks to microseconds (assuming 100 ticks/sec)
        let ticks_per_sec = 100;
        let user_time_us = (utime * 1_000_000) / ticks_per_sec;
        let system_time_us = (stime * 1_000_000) / ticks_per_sec;
        let total_time_us = user_time_us + system_time_us;
        
        // Calculate CPU usage percentage
        let now = Instant::now();
        let time_delta = now.duration_since(self.prev_measurement_time).as_micros() as u64;
        let cpu_delta = total_time_us.saturating_sub(self.prev_cpu_time);
        
        let usage_percent = if time_delta > 0 {
            (cpu_delta as f64 / time_delta as f64) * 100.0
        } else {
            0.0
        };
        
        // Update previous values
        self.prev_cpu_time = total_time_us;
        self.prev_measurement_time = now;
        
        // Get context switches from /proc/[pid]/status
        let (voluntary, involuntary) = self.get_context_switches()?;
        
        Ok(CpuMetrics {
            user_time_us,
            system_time_us,
            total_time_us,
            usage_percent,
            context_switches_voluntary: voluntary,
            context_switches_involuntary: involuntary,
        })
    }
    
    /// Collect disk I/O metrics from /proc/[pid]/io
    fn collect_disk_io_metrics(&mut self) -> Result<DiskIoMetrics, PluginError> {
        let io_path = format!("/proc/{}/io", self.pid);
        
        // /proc/[pid]/io requires root or same user, may not be readable
        let content = match fs::read_to_string(&io_path) {
            Ok(c) => c,
            Err(_) => {
                // If we can't read it, return zeros
                return Ok(DiskIoMetrics::default());
            }
        };
        
        let mut read_bytes = 0u64;
        let mut write_bytes = 0u64;
        let mut read_ops = 0u64;
        let mut write_ops = 0u64;
        
        for line in content.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() != 2 {
                continue;
            }
            
            let value = parts[1].trim().parse::<u64>().unwrap_or(0);
            
            match parts[0].trim() {
                "read_bytes" => read_bytes = value,
                "write_bytes" => write_bytes = value,
                "syscr" => read_ops = value,
                "syscw" => write_ops = value,
                _ => {}
            }
        }
        
        // Calculate bandwidth
        let now = Instant::now();
        let time_delta_secs = now.duration_since(self.prev_measurement_time).as_secs_f64();
        
        let read_bps = if time_delta_secs > 0.0 {
            ((read_bytes.saturating_sub(self.prev_disk_read)) as f64 / time_delta_secs) as u64
        } else {
            0
        };
        
        let write_bps = if time_delta_secs > 0.0 {
            ((write_bytes.saturating_sub(self.prev_disk_write)) as f64 / time_delta_secs) as u64
        } else {
            0
        };
        
        // Update previous values
        self.prev_disk_read = read_bytes;
        self.prev_disk_write = write_bytes;
        
        Ok(DiskIoMetrics {
            read_bytes,
            write_bytes,
            read_ops,
            write_ops,
            read_bps,
            write_bps,
        })
    }
    
    /// Collect network metrics from /proc/[pid]/net/dev and /proc/net/tcp*
    fn collect_network_metrics(&mut self) -> Result<NetworkMetrics, PluginError> {
        // Network stats are system-wide, not per-process in /proc/net/dev
        // For per-process, we'd need to use netlink or eBPF
        // As a simpler approach, we'll approximate using /proc/net/tcp counts
        
        let active_connections = self.get_connection_count().unwrap_or(0);
        
        // For now, return basic metrics (full implementation would require netlink)
        Ok(NetworkMetrics {
            rx_bytes: 0,
            tx_bytes: 0,
            rx_packets: 0,
            tx_packets: 0,
            rx_bps: 0,
            tx_bps: 0,
            active_connections,
        })
    }
    
    /// Collect process metrics
    fn collect_process_metrics(&self) -> Result<ProcessMetrics, PluginError> {
        let thread_count = self.get_thread_count()?;
        let fd_count = self.get_fd_count()?;
        let (state, nice) = self.get_process_state()?;
        let num_children = self.get_child_count()?;
        
        Ok(ProcessMetrics {
            thread_count,
            fd_count,
            state,
            nice,
            num_children,
        })
    }
    
    /// Collect GPU metrics (if nvidia-smi or similar is available)
    fn collect_gpu_metrics(&self) -> Result<GpuMetrics, PluginError> {
        // GPU monitoring would require nvidia-smi, rocm-smi, or similar
        // This is a placeholder - full implementation would exec nvidia-smi
        // and parse its output, or use NVML library
        Err(PluginError::LoadError("GPU monitoring not yet implemented".to_string()))
    }
    
    // Helper methods
    
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
                    return Ok(kb * 1024);
                }
            }
        }
        
        Err(PluginError::LoadError("VmRSS not found".to_string()))
    }
    
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
        
        Err(PluginError::LoadError("Threads not found".to_string()))
    }
    
    fn get_fd_count(&self) -> Result<u32, PluginError> {
        let fd_path = format!("/proc/{}/fd", self.pid);
        match fs::read_dir(&fd_path) {
            Ok(entries) => Ok(entries.count() as u32),
            Err(e) => Err(PluginError::LoadError(format!("Failed to read {}: {}", fd_path, e))),
        }
    }
    
    fn get_connection_count(&self) -> Result<u32, PluginError> {
        // Count connections for this PID
        let mut count = 0;
        
        // This is simplified - proper implementation would parse /proc/net/tcp
        // and match inode numbers with /proc/[pid]/fd/* symlinks
        if let Ok(content) = fs::read_to_string("/proc/net/tcp") {
            count += content.lines().skip(1).count();
        }
        
        if let Ok(content) = fs::read_to_string("/proc/net/tcp6") {
            count += content.lines().skip(1).count();
        }
        
        Ok(count as u32)
    }
    
    fn get_page_faults(&self) -> Result<(u64, u64), PluginError> {
        let stat_path = format!("/proc/{}/stat", self.pid);
        let stat = fs::read_to_string(&stat_path)
            .map_err(|e| PluginError::LoadError(format!("Failed to read {}: {}", stat_path, e)))?;
        
        let parts: Vec<&str> = stat.split_whitespace().collect();
        if parts.len() < 13 {
            return Ok((0, 0));
        }
        
        // Fields: minflt (10), majflt (12)
        let minflt = parts[9].parse::<u64>().unwrap_or(0);
        let majflt = parts[11].parse::<u64>().unwrap_or(0);
        
        Ok((minflt, majflt))
    }
    
    fn get_context_switches(&self) -> Result<(u64, u64), PluginError> {
        let status_path = format!("/proc/{}/status", self.pid);
        let content = fs::read_to_string(&status_path)
            .map_err(|e| PluginError::LoadError(format!("Failed to read {}: {}", status_path, e)))?;
        
        let mut voluntary = 0u64;
        let mut involuntary = 0u64;
        
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }
            
            match parts[0] {
                "voluntary_ctxt_switches:" => {
                    voluntary = parts[1].parse().unwrap_or(0);
                }
                "nonvoluntary_ctxt_switches:" => {
                    involuntary = parts[1].parse().unwrap_or(0);
                }
                _ => {}
            }
        }
        
        Ok((voluntary, involuntary))
    }
    
    fn get_process_state(&self) -> Result<(char, i32), PluginError> {
        let stat_path = format!("/proc/{}/stat", self.pid);
        let stat = fs::read_to_string(&stat_path)
            .map_err(|e| PluginError::LoadError(format!("Failed to read {}: {}", stat_path, e)))?;
        
        let parts: Vec<&str> = stat.split_whitespace().collect();
        if parts.len() < 19 {
            return Ok(('S', 0));
        }
        
        // State is field 3, nice is field 19
        let state = parts[2].chars().next().unwrap_or('S');
        let nice = parts[18].parse::<i32>().unwrap_or(0);
        
        Ok((state, nice))
    }
    
    fn get_child_count(&self) -> Result<u32, PluginError> {
        let children_path = format!("/proc/{}/task/{}/children", self.pid, self.pid);
        
        match fs::read_to_string(&children_path) {
            Ok(content) => {
                let count = content.split_whitespace().count() as u32;
                Ok(count)
            }
            Err(_) => Ok(0), // File might not exist if no children
        }
    }
    
    pub fn process_exists(&self) -> bool {
        Path::new(&format!("/proc/{}", self.pid)).exists()
    }
}

#[cfg(not(target_os = "linux"))]
impl EnhancedResourceMonitor {
    pub fn new(plugin_name: String, pid: u32, limits: ResourceLimits) -> Self {
        Self {
            plugin_name,
            pid,
            limits,
        }
    }
    
    pub fn collect_metrics(&mut self) -> Result<PluginMetrics, PluginError> {
        Err(PluginError::LoadError("Enhanced monitoring not available on this platform".to_string()))
    }
    
    pub fn check_violations(&self) -> Result<Vec<ViolationType>, PluginError> {
        Ok(Vec::new())
    }
    
    pub fn process_exists(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_monitor_creation() {
        let monitor = EnhancedResourceMonitor::new(
            "test".to_string(),
            std::process::id(),
            ResourceLimits::default(),
        );
        
        assert_eq!(monitor.plugin_name, "test");
    }
    
    #[test]
    #[cfg(target_os = "linux")]
    fn test_process_exists() {
        let monitor = EnhancedResourceMonitor::new(
            "test".to_string(),
            std::process::id(),
            ResourceLimits::default(),
        );
        
        assert!(monitor.process_exists());
    }
    
    #[test]
    #[cfg(target_os = "linux")]
    fn test_collect_metrics() {
        let mut monitor = EnhancedResourceMonitor::new(
            "test".to_string(),
            std::process::id(),
            ResourceLimits::default(),
        );
        
        // Should succeed for current process
        let metrics = monitor.collect_metrics();
        assert!(metrics.is_ok());
        
        let metrics = metrics.unwrap();
        assert!(metrics.memory.rss_bytes > 0);
        assert_eq!(metrics.pid, std::process::id());
    }
    
    #[test]
    #[cfg(target_os = "linux")]
    fn test_check_violations() {
        let monitor = EnhancedResourceMonitor::new(
            "test".to_string(),
            std::process::id(),
            ResourceLimits::default(),
        );
        
        let violations = monitor.check_violations();
        assert!(violations.is_ok());
    }
}

