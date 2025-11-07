use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::PluginError;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ContextKey {
    /// Predefined context keys, strongly typed
    Predefined(PredefinedContextKey),
    /// Custom context keys, identified by a static string
    Custom(&'static str),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PredefinedContextKey {
    /// Database connection (shared, read-only for most plugins)
    DatabaseConnection,
    
    /// HTTP router for registering routes (read-write for route registration)
    HttpRouter,
    
    /// Configuration data (read-only)
    Configuration,
    
    /// Shared cache (read-write)
    Cache,
    
    /// Event bus for pub/sub (read-write)
    EventBus,
    
    /// Metrics collector (write-only for most plugins)
    Metrics,
}

impl PredefinedContextKey {
    /// Convert to string key
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DatabaseConnection => "database_connection",
            Self::HttpRouter => "http_router",
            Self::Configuration => "configuration",
            Self::Cache => "cache",
            Self::EventBus => "event_bus",
            Self::Metrics => "metrics",
        }
    }
    
    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "database_connection" => Some(Self::DatabaseConnection),
            "http_router" => Some(Self::HttpRouter),
            "configuration" => Some(Self::Configuration),
            "cache" => Some(Self::Cache),
            "event_bus" => Some(Self::EventBus),
            "metrics" => Some(Self::Metrics),
            _ => None,
        }
    }
}

impl std::fmt::Display for PredefinedContextKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl ContextKey {
    /// Convert to string representation
    pub fn as_string(&self) -> String {
        match self {
            Self::Predefined(key) => key.as_str().to_string(),
            Self::Custom(key) => (*key).to_string(),
        }
    }
    
    /// Parse from string
    pub fn from_string(s: &str) -> Self {
        if let Some(predefined) = PredefinedContextKey::from_str(s) {
            Self::Predefined(predefined)
        } else {
            // For runtime strings, we need to leak them or use a different approach
            // In practice, custom keys should be &'static str
            Self::Custom(Box::leak(s.to_string().into_boxed_str()))
        }
    }
}

/// Shared context that can be passed to plugins
pub struct PluginContext {
    shared_objects: RwLock<HashMap<ContextKey, Arc<dyn Any + Send + Sync>>>,
    /// Optional permission checker (for server-side enforcement)
    permission_checker: Option<Arc<dyn ContextPermissionChecker>>,
}

/// Trait for checking context access permissions
pub trait ContextPermissionChecker: Send + Sync {
    /// Check if the current plugin has permission to access a context key
    fn check_permission(&self, plugin_name: &str, key: &str, access_level: crate::requirements::ContextAccessLevel) -> Result<(), PluginError>;
}

impl PluginContext {
    pub fn new() -> Self {
        Self {
            shared_objects: RwLock::new(HashMap::new()),
            permission_checker: None,
        }
    }
    
    /// Create a new context with permission checking
    pub fn with_permission_checker(checker: Arc<dyn ContextPermissionChecker>) -> Self {
        Self {
            shared_objects: RwLock::new(HashMap::new()),
            permission_checker: Some(checker),
        }
    }
    
    /// Set the permission checker
    pub fn set_permission_checker(&mut self, checker: Arc<dyn ContextPermissionChecker>) {
        self.permission_checker = Some(checker);
    }

    /// Share an object with plugins
    ///
    /// # Arguments
    ///
    /// * `object` - The object to share, wrapped in an Arc
    ///
    /// # Errors
    ///
    /// Returns `PluginError::InitializationError` if the lock cannot be acquired
    pub fn share<T: Any + Send + Sync + 'static>(&self, key: ContextKey, object: Arc<T>) -> Result<(), PluginError> {
        let mut objects = self
            .shared_objects
            .write()
            .map_err(|e| PluginError::InitializationError(format!("Failed to acquire write lock: {}", e)))?;
        objects.insert(key, object);
        Ok(())
    }

    /// Get a shared object from the context
    pub fn get<T: Any + Send + Sync + 'static>(&self, key: ContextKey) -> Result<Arc<T>, PluginError> {
        let objects = self
            .shared_objects
            .read()
            .map_err(|e| PluginError::InitializationError(format!("Failed to acquire read lock: {}", e)))?;

        objects
            .get(&key)
            .and_then(|obj| obj.clone().downcast::<T>().ok())
            .ok_or_else(|| {
                PluginError::InitializationError(format!(
                    "Shared object of type {} not found",
                    std::any::type_name::<T>()
                ))
            })
    }
    
    /// Get a shared object with permission checking
    pub fn get_with_permission<T: Any + Send + Sync + 'static>(
        &self,
        key: ContextKey,
        plugin_name: &str,
    ) -> Result<Arc<T>, PluginError> {
        // Check permissions if checker is set
        if let Some(checker) = &self.permission_checker {
            checker.check_permission(plugin_name, &key.as_string(), crate::requirements::ContextAccessLevel::Read)?;
        }
        
        self.get(key)
    }
    
    /// Share an object with permission checking
    pub fn share_with_permission<T: Any + Send + Sync + 'static>(
        &self,
        key: ContextKey,
        object: Arc<T>,
        plugin_name: &str,
    ) -> Result<(), PluginError> {
        // Check permissions if checker is set
        if let Some(checker) = &self.permission_checker {
            checker.check_permission(plugin_name, &key.as_string(), crate::requirements::ContextAccessLevel::ReadWrite)?;
        }
        
        self.share(key, object)
    }

    /// Check if a type is shared
    pub fn has<T: Any + Send + Sync + 'static>(&self, key: ContextKey) -> bool {
        self.shared_objects
            .read()
            .map(|objects| objects.contains_key(&key))
            .unwrap_or(false)
    }

    /// Remove a shared object
    pub fn remove<T: Any + Send + Sync + 'static>(&self, key: ContextKey) -> Result<(), PluginError> {
        let mut objects = self
            .shared_objects
            .write()
            .map_err(|e| PluginError::InitializationError(format!("Failed to acquire write lock: {}", e)))?;
        objects.remove(&key);
        Ok(())
    }
}

impl Default for PluginContext {
    fn default() -> Self { Self::new() }
}

/// Trait for objects that can be shared with plugins
pub trait SharedContext: Any + Send + Sync {}

/// Blanket implementation
impl<T: Any + Send + Sync> SharedContext for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestData {
        value: i32,
    }

    #[test]
    fn test_share_and_get() {
        let context = PluginContext::new();
        let data = Arc::new(TestData {
            value: 42,
        });

        context
            .share(ContextKey::Custom("test"), data.clone())
            .unwrap();

        let retrieved = context.get::<TestData>(ContextKey::Custom("test")).unwrap();
        assert_eq!(retrieved.value, 42);
    }

    #[test]
    fn test_has() {
        let context = PluginContext::new();
        assert!(!context.has::<TestData>(ContextKey::Custom("test")));

        let data = Arc::new(TestData {
            value: 42,
        });
        context.share(ContextKey::Custom("test"), data).unwrap();

        assert!(context.has::<TestData>(ContextKey::Custom("test")));
    }

    #[test]
    fn test_remove() {
        let context = PluginContext::new();
        let data = Arc::new(TestData {
            value: 42,
        });

        context.share(ContextKey::Custom("test"), data).unwrap();
        assert!(context.has::<TestData>(ContextKey::Custom("test")));

        context
            .remove::<TestData>(ContextKey::Custom("test"))
            .unwrap();
        assert!(!context.has::<TestData>(ContextKey::Custom("test")));
    }
}
