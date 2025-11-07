/// Server-side context manager for handling IPC context requests
/// 
/// This module manages context data access from isolated plugins via IPC,
/// enforcing permissions based on plugin requirements.

use crate::{
    PluginError, PluginContext, ContextKey, PredefinedContextKey,
    ContextPermissions, ContextPermissionChecker,
};
use crate::requirements::ContextAccessLevel;
use crate::ipc::protocol::IpcMessage;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{debug, warn, error};

/// Server-side context manager
/// 
/// Handles context access requests from plugins via IPC,
/// checking permissions before allowing access.
pub struct ContextManager {
    /// The actual plugin context with shared data
    context: Arc<PluginContext>,
    
    /// Plugin permissions (plugin_name -> permissions)
    permissions: Arc<RwLock<HashMap<String, ContextPermissions>>>,
}

impl ContextManager {
    /// Create a new context manager
    pub fn new(context: Arc<PluginContext>) -> Self {
        Self {
            context,
            permissions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a plugin's context permissions
    pub async fn register_plugin_permissions(
        &self,
        plugin_name: String,
        permissions: ContextPermissions,
    ) {
        let mut perms = self.permissions.write().await;
        perms.insert(plugin_name, permissions);
    }
    
    /// Remove a plugin's permissions (on unload)
    pub async fn unregister_plugin(&self, plugin_name: &str) {
        let mut perms = self.permissions.write().await;
        perms.remove(plugin_name);
    }
    
    /// Handle a context access request from a plugin
    /// 
    /// Returns an IPC response message to send back to the plugin
    pub async fn handle_context_request(
        &self,
        plugin_name: &str,
        request: IpcMessage,
    ) -> IpcMessage {
        match request {
            IpcMessage::ContextGet { key, request_id } => {
                self.handle_get(plugin_name, &key, request_id).await
            }
            IpcMessage::ContextSet { key, data, request_id } => {
                self.handle_set(plugin_name, &key, data, request_id).await
            }
            IpcMessage::ContextHas { key, request_id } => {
                self.handle_has(plugin_name, &key, request_id).await
            }
            _ => {
                error!("Unexpected context request type");
                IpcMessage::ContextGetResponse {
                    request_id: 0,
                    data: None,
                    error: Some("Invalid request type".to_string()),
                }
            }
        }
    }
    
    /// Handle ContextGet request
    async fn handle_get(
        &self,
        plugin_name: &str,
        key: &str,
        request_id: u64,
    ) -> IpcMessage {
        debug!("Plugin '{}' requesting context key '{}'", plugin_name, key);
        
        // Check permission
        if let Err(e) = self.check_permission(plugin_name, key, ContextAccessLevel::Read).await {
            warn!("Permission denied for plugin '{}' to read context '{}': {}", plugin_name, key, e);
            return IpcMessage::ContextGetResponse {
                request_id,
                data: None,
                error: Some(format!("Permission denied: {}", e)),
            };
        }
        
        // Get the context key
        let context_key = ContextKey::from_string(key);
        
        // Try to get the data (we need to serialize it)
        // Note: This is a simplified version - in practice, you'd need type information
        // or a serialization wrapper around context data
        match self.get_serialized_context(&context_key).await {
            Ok(data) => {
                debug!("Context '{}' retrieved for plugin '{}'", key, plugin_name);
                IpcMessage::ContextGetResponse {
                    request_id,
                    data: Some(data),
                    error: None,
                }
            }
            Err(e) => {
                warn!("Failed to get context '{}' for plugin '{}': {}", key, plugin_name, e);
                IpcMessage::ContextGetResponse {
                    request_id,
                    data: None,
                    error: Some(e.to_string()),
                }
            }
        }
    }
    
    /// Handle ContextSet request
    async fn handle_set(
        &self,
        plugin_name: &str,
        key: &str,
        data: Vec<u8>,
        request_id: u64,
    ) -> IpcMessage {
        debug!("Plugin '{}' setting context key '{}'", plugin_name, key);
        
        // Check permission
        if let Err(e) = self.check_permission(plugin_name, key, ContextAccessLevel::ReadWrite).await {
            warn!("Permission denied for plugin '{}' to write context '{}': {}", plugin_name, key, e);
            return IpcMessage::ContextSetResponse {
                request_id,
                success: false,
                error: Some(format!("Permission denied: {}", e)),
            };
        }
        
        // Get the context key
        let context_key = ContextKey::from_string(key);
        
        // Set the data
        match self.set_serialized_context(&context_key, data).await {
            Ok(()) => {
                debug!("Context '{}' set by plugin '{}'", key, plugin_name);
                IpcMessage::ContextSetResponse {
                    request_id,
                    success: true,
                    error: None,
                }
            }
            Err(e) => {
                warn!("Failed to set context '{}' for plugin '{}': {}", key, plugin_name, e);
                IpcMessage::ContextSetResponse {
                    request_id,
                    success: false,
                    error: Some(e.to_string()),
                }
            }
        }
    }
    
    /// Handle ContextHas request
    async fn handle_has(
        &self,
        plugin_name: &str,
        key: &str,
        request_id: u64,
    ) -> IpcMessage {
        debug!("Plugin '{}' checking context key '{}'", plugin_name, key);
        
        // Check permission (read permission required to check existence)
        let has_permission = self
            .check_permission(plugin_name, key, ContextAccessLevel::Read)
            .await
            .is_ok();
        
        if !has_permission {
            // Return false if no permission (don't reveal existence)
            return IpcMessage::ContextHasResponse {
                request_id,
                exists: false,
            };
        }
        
        let context_key = ContextKey::from_string(key);
        
        // Check if the key exists
        // Note: This requires a has_key method on PluginContext
        let exists = self.context_key_exists(&context_key).await;
        
        IpcMessage::ContextHasResponse {
            request_id,
            exists,
        }
    }
    
    /// Check if a plugin has permission to access a context key
    async fn check_permission(
        &self,
        plugin_name: &str,
        key: &str,
        required_access: ContextAccessLevel,
    ) -> Result<(), PluginError> {
        let perms = self.permissions.read().await;
        
        let plugin_perms = perms.get(plugin_name).ok_or_else(|| {
            PluginError::InitializationError(format!("Plugin '{}' not registered", plugin_name))
        })?;
        
        if !plugin_perms.is_allowed(key, required_access) {
            return Err(PluginError::InitializationError(format!(
                "Plugin '{}' does not have {:?} permission for context '{}'",
                plugin_name, required_access, key
            )));
        }
        
        Ok(())
    }
    
    /// Get serialized context data
    /// 
    /// This is a placeholder - in practice, you'd need a wrapper type
    /// that knows how to serialize the context data
    async fn get_serialized_context(&self, _key: &ContextKey) -> Result<Vec<u8>, PluginError> {
        // For now, return an error indicating this needs implementation
        // In practice, you'd need to:
        // 1. Store a serialization function with each context entry
        // 2. Or use a wrapper type that implements Serialize
        // 3. Or use a type registry
        
        Err(PluginError::InitializationError(
            "Context serialization not yet implemented - requires type wrapper".to_string()
        ))
    }
    
    /// Set context data from serialized bytes
    async fn set_serialized_context(&self, _key: &ContextKey, _data: Vec<u8>) -> Result<(), PluginError> {
        // Similar to get_serialized_context, this needs implementation
        Err(PluginError::InitializationError(
            "Context deserialization not yet implemented - requires type wrapper".to_string()
        ))
    }
    
    /// Check if a context key exists
    async fn context_key_exists(&self, _key: &ContextKey) -> bool {
        // This would need a has_key method on PluginContext
        // For now, return false
        false
    }
}

/// Permission checker implementation for ContextManager
pub struct ContextManagerPermissionChecker {
    permissions: Arc<RwLock<HashMap<String, ContextPermissions>>>,
}

impl ContextManagerPermissionChecker {
    pub fn new(permissions: Arc<RwLock<HashMap<String, ContextPermissions>>>) -> Self {
        Self { permissions }
    }
}

impl ContextPermissionChecker for ContextManagerPermissionChecker {
    fn check_permission(
        &self,
        plugin_name: &str,
        key: &str,
        access_level: ContextAccessLevel,
    ) -> Result<(), PluginError> {
        // This is a synchronous wrapper around async permission checking
        // In practice, you might want to use a different approach
        // For now, we'll use a blocking call (not ideal but works)
        
        let perms = self.permissions.blocking_read();
        
        let plugin_perms = perms.get(plugin_name).ok_or_else(|| {
            PluginError::InitializationError(format!("Plugin '{}' not registered", plugin_name))
        })?;
        
        if !plugin_perms.is_allowed(key, access_level) {
            return Err(PluginError::InitializationError(format!(
                "Plugin '{}' does not have {:?} permission for context '{}'",
                plugin_name, access_level, key
            )));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ContextPermission, ContextPermissions};
    
    #[tokio::test]
    async fn test_permission_checking() {
        let context = Arc::new(PluginContext::new());
        let manager = ContextManager::new(context);
        
        // Register permissions for a plugin
        let permissions = ContextPermissions::allow(vec![
            ContextPermission::read("database_connection"),
            ContextPermission::read_write("http_router"),
        ]);
        
        manager.register_plugin_permissions("test_plugin".to_string(), permissions).await;
        
        // Test read permission (should succeed)
        assert!(manager.check_permission("test_plugin", "database_connection", ContextAccessLevel::Read).await.is_ok());
        
        // Test write permission on read-only key (should fail)
        assert!(manager.check_permission("test_plugin", "database_connection", ContextAccessLevel::ReadWrite).await.is_err());
        
        // Test read-write permission (should succeed)
        assert!(manager.check_permission("test_plugin", "http_router", ContextAccessLevel::Read).await.is_ok());
        assert!(manager.check_permission("test_plugin", "http_router", ContextAccessLevel::ReadWrite).await.is_ok());
        
        // Test permission on unregistered key (should fail)
        assert!(manager.check_permission("test_plugin", "unknown_key", ContextAccessLevel::Read).await.is_err());
    }
}

