/// Context proxy for isolated plugins to access context data via IPC
///
/// This module provides a transparent way for plugins running in separate processes
/// to access shared context data (like database connections, routers, etc.) through IPC.

use crate::PluginError;
use crate::ipc::protocol::{IpcMessage, IpcError};
use serde::{Serialize, de::DeserializeOwned};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::{RwLock, oneshot};
use std::collections::HashMap;

/// Context proxy for IPC-based context access
///
/// This allows plugins in isolated processes to access context data
/// as if it were local, but all operations go through IPC to the server.
pub struct ContextProxy {
    /// Channel to send IPC messages to server
    sender: Arc<RwLock<Box<dyn ContextProxySender>>>,

    /// Request ID counter
    request_counter: AtomicU64,

    /// Pending requests waiting for responses
    pending: Arc<RwLock<HashMap<u64, oneshot::Sender<IpcMessage>>>>,
}

/// Trait for sending IPC messages (allows testing and abstraction)
#[async_trait::async_trait]
pub trait ContextProxySender: Send + Sync {
    async fn send(&mut self, msg: IpcMessage) -> Result<(), IpcError>;
}

impl ContextProxy {
    /// Create a new context proxy
    pub fn new(sender: Box<dyn ContextProxySender>) -> Self {
        Self {
            sender: Arc::new(RwLock::new(sender)),
            request_counter: AtomicU64::new(1),
            pending: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get context data by key
    ///
    /// # Arguments
    /// * `key` - The context key to retrieve
    ///
    /// # Returns
    /// The deserialized context data
    ///
    /// # Errors
    /// Returns error if:
    /// - Permission denied (plugin doesn't have access)
    /// - Context key doesn't exist
    /// - Deserialization fails
    /// - IPC communication fails
    pub async fn get<T: DeserializeOwned + bincode::Decode<()>>(&self, key: &str) -> Result<T, PluginError> {
        let request_id = self.request_counter.fetch_add(1, Ordering::SeqCst);

        // Create response channel
        let (tx, rx) = oneshot::channel();

        // Register pending request
        {
            let mut pending = self.pending.write().await;
            pending.insert(request_id, tx);
        }

        // Send request
        let msg = IpcMessage::ContextGet {
            key: key.to_string(),
            request_id,
        };

        {
            let mut sender = self.sender.write().await;
            sender.send(msg).await
                .map_err(|e| PluginError::InitializationError(format!("Failed to send context request: {}", e)))?;
        }

        // Wait for response
        let response = rx.await
            .map_err(|_| PluginError::InitializationError("Context request cancelled".to_string()))?;

        // Process response
        match response {
            IpcMessage::ContextGetResponse { data, error, .. } => {
                if let Some(err) = error {
                    return Err(PluginError::InitializationError(err));
                }

                let data = data.ok_or_else(|| {
                    PluginError::InitializationError(format!("Context key '{}' not found", key))
                })?;

                let (value, _len) = bincode::decode_from_slice(&data, bincode::config::standard())
                    .map_err(|e| PluginError::InitializationError(format!("Failed to deserialize context data: {}", e)))?;
                Ok(value)
            }
            _ => Err(PluginError::InitializationError("Unexpected response type".to_string())),
        }
    }

    /// Set context data by key
    ///
    /// # Arguments
    /// * `key` - The context key to set
    /// * `value` - The value to set
    ///
    /// # Errors
    /// Returns error if:
    /// - Permission denied (plugin doesn't have write access)
    /// - Serialization fails
    /// - IPC communication fails
    pub async fn set<T: Serialize + bincode::Encode>(&self, key: &str, value: &T) -> Result<(), PluginError> {
        let request_id = self.request_counter.fetch_add(1, Ordering::SeqCst);

        // Serialize data
        let data = bincode::encode_to_vec(value, bincode::config::standard())
            .map_err(|e| PluginError::InitializationError(format!("Failed to serialize context data: {}", e)))?;

        // Create response channel
        let (tx, rx) = oneshot::channel();

        // Register pending request
        {
            let mut pending = self.pending.write().await;
            pending.insert(request_id, tx);
        }

        // Send request
        let msg = IpcMessage::ContextSet {
            key: key.to_string(),
            data,
            request_id,
        };

        {
            let mut sender = self.sender.write().await;
            sender.send(msg).await
                .map_err(|e| PluginError::InitializationError(format!("Failed to send context set request: {}", e)))?;
        }

        // Wait for response
        let response = rx.await
            .map_err(|_| PluginError::InitializationError("Context set request cancelled".to_string()))?;

        // Process response
        match response {
            IpcMessage::ContextSetResponse { success, error, .. } => {
                if success {
                    Ok(())
                } else {
                    Err(PluginError::InitializationError(
                        error.unwrap_or_else(|| "Failed to set context data".to_string())
                    ))
                }
            }
            _ => Err(PluginError::InitializationError("Unexpected response type".to_string())),
        }
    }

    /// Check if a context key exists
    ///
    /// # Arguments
    /// * `key` - The context key to check
    ///
    /// # Returns
    /// `true` if the key exists and plugin has permission, `false` otherwise
    pub async fn has(&self, key: &str) -> Result<bool, PluginError> {
        let request_id = self.request_counter.fetch_add(1, Ordering::SeqCst);

        // Create response channel
        let (tx, rx) = oneshot::channel();

        // Register pending request
        {
            let mut pending = self.pending.write().await;
            pending.insert(request_id, tx);
        }

        // Send request
        let msg = IpcMessage::ContextHas {
            key: key.to_string(),
            request_id,
        };

        {
            let mut sender = self.sender.write().await;
            sender.send(msg).await
                .map_err(|e| PluginError::InitializationError(format!("Failed to send context has request: {}", e)))?;
        }

        // Wait for response
        let response = rx.await
            .map_err(|_| PluginError::InitializationError("Context has request cancelled".to_string()))?;

        // Process response
        match response {
            IpcMessage::ContextHasResponse { exists, .. } => Ok(exists),
            _ => Err(PluginError::InitializationError("Unexpected response type".to_string())),
        }
    }

    /// Handle an IPC response message
    ///
    /// This should be called by the IPC receiver when a context response arrives
    pub async fn handle_response(&self, msg: IpcMessage) -> Result<(), PluginError> {
        let request_id = match &msg {
            IpcMessage::ContextGetResponse { request_id, .. } => *request_id,
            IpcMessage::ContextSetResponse { request_id, .. } => *request_id,
            IpcMessage::ContextHasResponse { request_id, .. } => *request_id,
            _ => return Err(PluginError::InitializationError("Not a context response".to_string())),
        };

        // Find and remove pending request
        let tx = {
            let mut pending = self.pending.write().await;
            pending.remove(&request_id)
        };

        if let Some(tx) = tx {
            // Send response to waiting caller
            let _ = tx.send(msg); // Ignore error if receiver dropped
        }

        Ok(())
    }
}

/// Builder for creating context proxies with specific permissions
pub struct ContextProxyBuilder {
    sender: Option<Box<dyn ContextProxySender>>,
}

impl ContextProxyBuilder {
    pub fn new() -> Self {
        Self { sender: None }
    }

    pub fn with_sender(mut self, sender: Box<dyn ContextProxySender>) -> Self {
        self.sender = Some(sender);
        self
    }

    pub fn build(self) -> Result<ContextProxy, PluginError> {
        let sender = self.sender
            .ok_or_else(|| PluginError::InitializationError("Sender not set".to_string()))?;

        Ok(ContextProxy::new(sender))
    }
}

impl Default for ContextProxyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct MockSender {
        sent: Arc<RwLock<Vec<IpcMessage>>>,
    }

    #[async_trait]
    impl ContextProxySender for MockSender {
        async fn send(&mut self, msg: IpcMessage) -> Result<(), IpcError> {
            let mut sent = self.sent.write().await;
            sent.push(msg);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_context_proxy_get() {
        let sent = Arc::new(RwLock::new(Vec::new()));
        let sender = Box::new(MockSender { sent: sent.clone() });
        let proxy = ContextProxy::new(sender);

        // Start a get request (will hang waiting for response in real use)
        let handle = tokio::spawn(async move {
            proxy.get::<i32>("test_key").await
        });

        // Verify message was sent
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        let messages = sent.read().await;
        assert_eq!(messages.len(), 1);

        match &messages[0] {
            IpcMessage::ContextGet { key, request_id } => {
                assert_eq!(key, "test_key");
                assert_eq!(*request_id, 1);
            }
            _ => panic!("Wrong message type"),
        }

        // Note: Can't easily test the response without a full mock IPC system
        drop(handle);
    }
}

