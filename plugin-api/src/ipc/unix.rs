/// Unix Domain Socket implementation for IPC
use crate::ipc::{IpcConfig, IpcError};
use crate::ipc::protocol::{IpcMessage, MessageFrame};
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::time::timeout;
use std::time::Duration;

/// Unix domain socket server
pub struct UnixIpcServer {
    listener: UnixListener,
    config: IpcConfig,
    socket_path: PathBuf,
}

impl UnixIpcServer {
    /// Create a new Unix socket server for a plugin
    pub async fn new(plugin_name: &str, config: IpcConfig) -> Result<Self, IpcError> {
        // Create socket directory if it doesn't exist
        tokio::fs::create_dir_all(&config.socket_dir).await?;
        
        // Socket path: /tmp/orbis-plugins/plugin-<name>.sock
        let socket_path = config.socket_dir.join(format!("plugin-{}.sock", plugin_name));
        
        // Remove old socket if exists
        let _ = tokio::fs::remove_file(&socket_path).await;
        
        // Create listener
        let listener = UnixListener::bind(&socket_path)?;
        
        Ok(Self {
            listener,
            config,
            socket_path,
        })
    }
    
    /// Accept a connection from a plugin worker
    pub async fn accept(&self) -> Result<UnixIpcChannel, IpcError> {
        let (stream, _addr) = self.listener.accept().await?;
        
        Ok(UnixIpcChannel {
            stream,
            config: self.config.clone(),
        })
    }
    
    /// Get the socket path
    pub fn socket_path(&self) -> &PathBuf {
        &self.socket_path
    }
}

impl Drop for UnixIpcServer {
    fn drop(&mut self) {
        // Clean up socket file
        let _ = std::fs::remove_file(&self.socket_path);
    }
}

/// Unix domain socket channel for communication
pub struct UnixIpcChannel {
    stream: UnixStream,
    config: IpcConfig,
}

impl UnixIpcChannel {
    /// Connect to a Unix socket server
    pub async fn connect(socket_path: &PathBuf, config: IpcConfig) -> Result<Self, IpcError> {
        let stream = UnixStream::connect(socket_path).await?;
        
        Ok(Self {
            stream,
            config,
        })
    }
    
    /// Send a message
    pub async fn send(&mut self, msg: &IpcMessage) -> Result<(), IpcError> {
        let frame = MessageFrame::from_message(msg)?;
        let encoded = frame.encode();
        
        let timeout_duration = Duration::from_millis(self.config.timeout_ms);
        
        timeout(timeout_duration, self.stream.write_all(&encoded))
            .await
            .map_err(|_| IpcError::Timeout(self.config.timeout_ms))?
            .map_err(IpcError::Io)?;
        
        self.stream.flush().await?;
        
        Ok(())
    }
    
    /// Receive a message
    pub async fn recv(&mut self) -> Result<IpcMessage, IpcError> {
        let timeout_duration = Duration::from_millis(self.config.timeout_ms);
        
        // Read length prefix (4 bytes)
        let mut len_buf = [0u8; 4];
        timeout(timeout_duration, self.stream.read_exact(&mut len_buf))
            .await
            .map_err(|_| IpcError::Timeout(self.config.timeout_ms))?
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    IpcError::ConnectionClosed
                } else {
                    IpcError::Io(e)
                }
            })?;
        
        let len = u32::from_be_bytes(len_buf) as usize;
        
        // Validate length
        if len > self.config.buffer_size {
            return Err(IpcError::Protocol(format!(
                "Message too large: {} bytes (max: {})",
                len, self.config.buffer_size
            )));
        }
        
        // Read message data
        let mut data = vec![0u8; len];
        timeout(timeout_duration, self.stream.read_exact(&mut data))
            .await
            .map_err(|_| IpcError::Timeout(self.config.timeout_ms))?
            .map_err(IpcError::Io)?;
        
        // Deserialize
        let frame = MessageFrame { data };
        frame.to_message()
    }
    
    /// Send and wait for response
    pub async fn request(&mut self, msg: &IpcMessage) -> Result<IpcMessage, IpcError> {
        self.send(msg).await?;
        self.recv().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::protocol::LogLevel;
    
    #[tokio::test]
    async fn test_unix_socket_communication() {
        let config = IpcConfig::default();
        
        // Create server
        let server = UnixIpcServer::new("test", config.clone()).await.unwrap();
        let socket_path = server.socket_path().clone();
        
        // Spawn server task
        let server_handle = tokio::spawn(async move {
            let mut channel = server.accept().await.unwrap();
            
            // Receive ping
            let msg = channel.recv().await.unwrap();
            assert!(matches!(msg, IpcMessage::Ping));
            
            // Send pong
            channel.send(&IpcMessage::Pong).await.unwrap();
        });
        
        // Give server time to start
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Connect client
        let mut client = UnixIpcChannel::connect(&socket_path, config).await.unwrap();
        
        // Send ping
        client.send(&IpcMessage::Ping).await.unwrap();
        
        // Receive pong
        let response = client.recv().await.unwrap();
        assert!(matches!(response, IpcMessage::Pong));
        
        // Wait for server
        server_handle.await.unwrap();
    }
    
    #[tokio::test]
    async fn test_message_size_limit() {
        let config = IpcConfig {
            buffer_size: 1024,
            ..Default::default()
        };
        
        let server = UnixIpcServer::new("test2", config.clone()).await.unwrap();
        let socket_path = server.socket_path().clone();
        
        let server_handle = tokio::spawn(async move {
            let mut channel = server.accept().await.unwrap();
            
            // Try to receive oversized message
            let result = channel.recv().await;
            assert!(result.is_err());
        });
        
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let mut client = UnixIpcChannel::connect(&socket_path, config).await.unwrap();
        
        // Send oversized message (manually craft to bypass serialization limit)
        let large_data = vec![0u8; 2048];
        let len_bytes = (large_data.len() as u32).to_be_bytes();
        client.stream.write_all(&len_bytes).await.unwrap();
        client.stream.write_all(&large_data).await.unwrap();
        
        server_handle.await.unwrap();
    }
}

