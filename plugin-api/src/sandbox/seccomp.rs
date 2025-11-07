/// Seccomp syscall filtering for plugin sandboxing
use crate::PluginError;
use tracing::{info, warn};

// Note: Full seccomp implementation requires libseccomp-sys or manual BPF
// This is a simplified interface that can be extended

/// Seccomp filter modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeccompMode {
    /// No filtering
    Disabled,
    /// Log violations but allow
    Log,
    /// Block violations
    Strict,
}

/// Seccomp filter configuration
#[derive(Debug, Clone)]
pub struct SeccompConfig {
    pub mode: SeccompMode,
    /// Allowed syscalls (whitelist)
    pub allowed_syscalls: Vec<String>,
}

impl Default for SeccompConfig {
    fn default() -> Self {
        Self {
            mode: SeccompMode::Strict,
            allowed_syscalls: Self::default_allowed_syscalls(),
        }
    }
}

impl SeccompConfig {
    /// Default whitelist of allowed syscalls for plugins
    pub fn default_allowed_syscalls() -> Vec<String> {
        vec![
            // Essential syscalls
            "read", "write", "open", "close", "stat", "fstat", "lstat",
            "poll", "lseek", "mmap", "mprotect", "munmap", "brk",
            "rt_sigaction", "rt_sigprocmask", "rt_sigreturn",
            "ioctl", "pread64", "pwrite64", "readv", "writev",
            "access", "pipe", "select", "sched_yield", "mremap",
            "msync", "mincore", "madvise", "dup", "dup2", "pause",
            "nanosleep", "getitimer", "alarm", "setitimer", "getpid",
            "sendfile", "socket", "connect", "accept", "sendto",
            "recvfrom", "sendmsg", "recvmsg", "shutdown", "bind",
            "listen", "getsockname", "getpeername", "socketpair",
            "setsockopt", "getsockopt", "clone", "fork", "vfork",
            "execve", "exit", "wait4", "kill", "uname", "fcntl",
            "flock", "fsync", "fdatasync", "truncate", "ftruncate",
            "getdents", "getcwd", "chdir", "fchdir", "rename",
            "mkdir", "rmdir", "creat", "link", "unlink", "symlink",
            "readlink", "chmod", "fchmod", "chown", "fchown",
            "lchown", "umask", "gettimeofday", "getrlimit", "getrusage",
            "sysinfo", "times", "getuid", "getgid", "setuid", "setgid",
            "geteuid", "getegid", "getppid", "getpgrp", "setsid",
            "getgroups", "setgroups", "sigaltstack", "prctl",
            "arch_prctl", "gettid", "futex", "sched_getaffinity",
            "epoll_create", "epoll_ctl", "epoll_wait", "epoll_pwait",
            "clock_gettime", "clock_getres", "clock_nanosleep",
            "exit_group", "epoll_create1", "dup3", "pipe2",
            "preadv", "pwritev", "getrandom", "memfd_create",
        ].into_iter().map(String::from).collect()
    }

    /// Create a minimal seccomp config (very permissive)
    pub fn minimal() -> Self {
        Self {
            mode: SeccompMode::Log,
            allowed_syscalls: Vec::new(), // Empty = allow all
        }
    }

    /// Create a strict seccomp config (restrictive)
    pub fn strict() -> Self {
        Self {
            mode: SeccompMode::Strict,
            allowed_syscalls: Self::default_allowed_syscalls(),
        }
    }
}

/// Apply seccomp filter to current process using prctl
pub fn apply_seccomp_filter(config: &SeccompConfig) -> Result<(), PluginError> {
    match config.mode {
        SeccompMode::Disabled => {
            info!("Seccomp disabled");
            Ok(())
        }
        SeccompMode::Log | SeccompMode::Strict => {
            let kill_on_violation = config.mode == SeccompMode::Strict;
            info!(
                "Applying seccomp filter with {} allowed syscalls (mode: {:?})",
                config.allowed_syscalls.len(),
                config.mode
            );

            // Use seccomp with whitelist approach
            apply_seccomp_whitelist(&config.allowed_syscalls, kill_on_violation)?;

            info!("Seccomp filter applied successfully");
            Ok(())
        }
    }
}

/// Apply seccomp whitelist filter using libseccomp
#[cfg(target_os = "linux")]
fn apply_seccomp_whitelist(allowed: &[String], kill: bool) -> Result<(), PluginError> {
    use libseccomp::*;

    if allowed.is_empty() {
        warn!("Empty syscall whitelist - allowing all syscalls");
        return Ok(());
    }

    // Create a seccomp context with default action
    let default_action = if kill {
        ScmpAction::KillProcess
    } else {
        ScmpAction::Log
    };

    let mut ctx = ScmpFilterContext::new_filter(default_action)
        .map_err(|e| PluginError::LoadError(format!("Failed to create seccomp context: {}", e)))?;

    // Add rules to allow each whitelisted syscall
    for syscall_name in allowed {
        // Resolve syscall name to number
        match resolve_syscall_name(syscall_name) {
            Ok(syscall_num) => {
                ctx.add_rule(ScmpAction::Allow, syscall_num)
                    .map_err(|e| PluginError::LoadError(format!("Failed to add seccomp rule for {}: {}", syscall_name, e)))?;
            }
            Err(e) => {
                warn!("Unknown syscall '{}': {} - skipping", syscall_name, e);
            }
        }
    }

    // Load the filter
    ctx.load()
        .map_err(|e| PluginError::LoadError(format!("Failed to load seccomp filter: {}", e)))?;

    info!("Seccomp filter loaded with {} allowed syscalls", allowed.len());
    Ok(())
}

/// Resolve syscall name to syscall number
#[cfg(target_os = "linux")]
fn resolve_syscall_name(name: &str) -> Result<i32, String> {
    use libseccomp::*;

    ScmpSyscall::from_name(name)
        .map(|sc| sc.into())  // Convert ScmpSyscall to i32
        .map_err(|e| format!("Failed to resolve syscall: {}", e))
}

#[cfg(not(target_os = "linux"))]
fn apply_seccomp_whitelist(_allowed: &[String], _kill: bool) -> Result<(), PluginError> {
    warn!("Seccomp not available on this platform");
    Ok(())
}

/// Blocked syscalls (security-sensitive)
pub fn get_blocked_syscalls() -> Vec<&'static str> {
    vec![
        "ptrace",           // Process tracing
        "kexec_load",       // Load new kernel
        "kexec_file_load",
        "module_init",      // Load kernel modules
        "delete_module",
        "mount",            // Filesystem operations
        "umount",
        "umount2",
        "pivot_root",
        "swapon",           // Swap management
        "swapoff",
        "reboot",           // System control
        "sethostname",
        "setdomainname",
        "iopl",             // I/O privilege level
        "ioperm",
        "create_module",
        "init_module",
        "finit_module",
        "query_module",
        "quotactl",
        "nfsservctl",
        "acct",
        "settimeofday",
        "adjtimex",
        "clock_settime",
        "lookup_dcookie",
        "perf_event_open",
        "fanotify_init",
        "kcmp",
        "bpf",              // BPF operations
        "userfaultfd",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_seccomp_config() {
        let config = SeccompConfig::default();
        assert_eq!(config.mode, SeccompMode::Strict);
        assert!(!config.allowed_syscalls.is_empty());
    }

    #[test]
    fn test_minimal_seccomp() {
        let config = SeccompConfig::minimal();
        assert_eq!(config.mode, SeccompMode::Log);
    }

    #[test]
    fn test_blocked_syscalls() {
        let blocked = get_blocked_syscalls();
        assert!(blocked.contains(&"ptrace"));
        assert!(blocked.contains(&"mount"));
        assert!(blocked.contains(&"reboot"));
    }
}

