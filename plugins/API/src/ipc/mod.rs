/// Inter-Process Communication module for plugin sandboxing
/// 
/// This module provides communication between the main server process
/// and isolated plugin worker processes via Unix Domain Sockets (Linux/macOS)
/// or Named Pipes (Windows).

pub mod protocol;
pub mod server;
pub mod channel;

#[cfg(unix)]
pub mod unix;

#[cfg(windows)]
pub mod windows;

pub use protocol::{IpcMessage, IpcError};
pub use server::IpcServer;
pub use channel::IpcChannel;

use std::path::PathBuf;

/// IPC configuration
#[derive(Debug, Clone)]
pub struct IpcConfig {
    /// Directory for Unix domain socket files or named pipe prefix
    pub socket_dir: PathBuf,
    
    /// Timeout for IPC operations in milliseconds
    pub timeout_ms: u64,
    
    /// Buffer size for messages
    pub buffer_size: usize,
}

impl Default for IpcConfig {
    fn default() -> Self {
        Self {
            socket_dir: PathBuf::from("/tmp/orbis-plugins"),
            timeout_ms: 5000,
            buffer_size: 65536, // 64KB
        }
    }
}

