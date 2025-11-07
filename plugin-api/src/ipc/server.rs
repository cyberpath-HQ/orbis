/// IPC Server - manages communication with plugin worker processes
use crate::ipc::{IpcConfig, IpcError};
use crate::ipc::protocol::IpcMessage;
use std::path::PathBuf;

#[cfg(unix)]
use crate::ipc::unix::{UnixIpcServer, UnixIpcChannel};

/// Platform-agnostic IPC server
pub struct IpcServer {
    #[cfg(unix)]
    inner: UnixIpcServer,
    
    #[cfg(windows)]
    inner: crate::ipc::windows::WindowsIpcServer,
}

impl IpcServer {
    /// Create a new IPC server for a plugin
    pub async fn new(plugin_name: &str, config: IpcConfig) -> Result<Self, IpcError> {
        #[cfg(unix)]
        {
            let inner = UnixIpcServer::new(plugin_name, config).await?;
            Ok(Self { inner })
        }
        
        #[cfg(windows)]
        {
            let inner = crate::ipc::windows::WindowsIpcServer::new(plugin_name, config).await?;
            Ok(Self { inner })
        }
    }
    
    /// Accept a connection from a plugin worker
    pub async fn accept(&self) -> Result<IpcChannel, IpcError> {
        #[cfg(unix)]
        {
            let channel = self.inner.accept().await?;
            Ok(IpcChannel { inner: channel })
        }
        
        #[cfg(windows)]
        {
            let channel = self.inner.accept().await?;
            Ok(IpcChannel { inner: channel })
        }
    }
    
    /// Get the connection endpoint (socket path or pipe name)
    pub fn endpoint(&self) -> String {
        #[cfg(unix)]
        {
            self.inner.socket_path().to_string_lossy().to_string()
        }
        
        #[cfg(windows)]
        {
            self.inner.pipe_name().to_string()
        }
    }
}

/// Platform-agnostic IPC channel
pub struct IpcChannel {
    #[cfg(unix)]
    inner: UnixIpcChannel,
    
    #[cfg(windows)]
    inner: crate::ipc::windows::WindowsIpcChannel,
}

impl IpcChannel {
    /// Connect to an IPC server
    pub async fn connect(endpoint: &str, config: IpcConfig) -> Result<Self, IpcError> {
        #[cfg(unix)]
        {
            let path = PathBuf::from(endpoint);
            let inner = UnixIpcChannel::connect(&path, config).await?;
            Ok(Self { inner })
        }
        
        #[cfg(windows)]
        {
            let inner = crate::ipc::windows::WindowsIpcChannel::connect(endpoint, config).await?;
            Ok(Self { inner })
        }
    }
    
    /// Send a message
    pub async fn send(&mut self, msg: &IpcMessage) -> Result<(), IpcError> {
        self.inner.send(msg).await
    }
    
    /// Receive a message
    pub async fn recv(&mut self) -> Result<IpcMessage, IpcError> {
        self.inner.recv().await
    }
    
    /// Send and wait for response (request/response pattern)
    pub async fn request(&mut self, msg: &IpcMessage) -> Result<IpcMessage, IpcError> {
        self.inner.request(msg).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_ipc_server_creation() {
        let config = IpcConfig::default();
        let server = IpcServer::new("test_plugin", config).await.unwrap();
        
        // Verify endpoint is created
        let endpoint = server.endpoint();
        assert!(!endpoint.is_empty());
        
        #[cfg(unix)]
        assert!(endpoint.contains("plugin-test_plugin.sock"));
    }
}

