// filepath: /home/ebalo/Desktop/projects/rust/orbis-assets/plugin-api/src/hook.rs
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, RwLock};
use std::any::Any;
use std::fmt::Debug;
use async_trait::async_trait;
use crate::{PluginContext, PluginError};

/// Priority level for hook execution (0 = lowest, 255 = highest)
/// Higher priority handles execute first
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
pub struct HookPriority(u8);

impl HookPriority {
    pub const LOWEST: Self = Self(u8::MIN);
    pub const LOW: Self = Self(10);
    pub const NORMAL: Self = Self(50);
    pub const HIGH: Self = Self(100);
    pub const HIGHEST: Self = Self(u8::MAX);

    /// Create a new HookPriority with the given value
    pub fn new(priority: u8) -> Self {
        Self(priority)
    }

    pub fn value(&self) -> u8 {
        self.0
    }
}

impl Ord for HookPriority {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse order: higher values come first
        other.0.cmp(&self.0)
    }
}

impl Default for HookPriority {
    fn default() -> Self {
        Self::NORMAL
    }
}

/// Hook handle identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HandleId(usize);

/// Hook handler trait - handles transform data and can access plugin context
/// The output of one handle becomes the input of the next
#[async_trait(?Send)]
pub trait HookHandle<T: Clone>: Send + Sync {
    /// Handle the data, transforming it for the next handle in the chain
    /// Has access to the plugin context for any needed operations
    async fn handle(&self, data: T, context: &PluginContext) -> Result<T, PluginError>;
}

impl<T: Clone> Debug for dyn HookHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HookHandle")
    }
}

/// Implementation for async closures
#[async_trait(?Send)]
impl<T, F, Fut> HookHandle<T> for F
where
    F: Fn(T, &PluginContext) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<T, PluginError>>,
    T: Clone + Send + Sync + 'static,
{
    async fn handle(&self, data: T, context: &PluginContext) -> Result<T, PluginError> {
        self(data, context).await
    }
}

/// Wrapper for hook handles with priority and metadata
struct PrioritizedHandle<T: Clone> {
    priority: HookPriority,
    handle: Arc<dyn HookHandle<T>>,
    id: HandleId,
    name: Option<String>,
}

impl<T: Clone> PrioritizedHandle<T> {
    fn new(
        priority: HookPriority,
        handle: Arc<dyn HookHandle<T>>,
        id: HandleId,
        name: Option<String>,
    ) -> Self {
        Self {
            priority,
            handle,
            id,
            name,
        }
    }
}

/// Hook that manages and executes handles in priority order
/// Handles are executed as a chain where each handle's output becomes the next's input
pub struct Hook<T: Clone + 'static> {
    name: String,
    handles: RwLock<BTreeMap<(HookPriority, HandleId), PrioritizedHandle<T>>>,
    next_id: RwLock<usize>,
}

impl<T: Clone + 'static> Hook<T> {
    pub fn new(name: String) -> Self {
        Self {
            name,
            handles: RwLock::new(BTreeMap::new()),
            next_id: RwLock::new(0),
        }
    }

    /// Register a handle with a priority
    /// Returns the HandleId that can be used to unregister the handle
    pub fn register_handle(
        &self,
        priority: HookPriority,
        handle: Arc<dyn HookHandle<T>>,
        handle_name: Option<String>,
    ) -> Result<HandleId, PluginError> {
        let mut next_id = self.next_id.write()
            .map_err(|e| PluginError::HookError(format!("Failed to acquire id lock: {}", e)))?;
        let id = HandleId(*next_id);
        *next_id = next_id.wrapping_add(1);

        let prioritized = PrioritizedHandle::new(priority, handle, id, handle_name);

        let mut handles = self.handles.write()
            .map_err(|e| PluginError::HookError(format!("Failed to acquire write lock: {}", e)))?;
        handles.insert((priority, id), prioritized);

        Ok(id)
    }

    /// Unregister a handle by id
    pub fn unregister_handle(&self, priority: HookPriority, id: HandleId) -> Result<(), PluginError> {
        let mut handles = self.handles.write()
            .map_err(|e| PluginError::HookError(format!("Failed to acquire write lock: {}", e)))?;
        handles.remove(&(priority, id));
        Ok(())
    }

    /// Trigger the hook, executing all handles in priority order (highest first)
    /// Each handle's output becomes the input of the next handle
    /// If any handle fails, execution stops and the error is returned
    pub async fn trigger(&self, initial_data: T, context: &PluginContext) -> Result<T, PluginError> {
        let handles = self.handles.read()
            .map_err(|e| PluginError::HookError(format!("Failed to acquire read lock: {}", e)))?;

        let mut data = initial_data;

        // Handles are already sorted by BTreeMap (higher priority first)
        for (_key, prioritized) in handles.iter() {
            tracing::debug!(
                "Executing handle {:?} with priority {} for hook '{}'",
                prioritized.name.as_ref().unwrap_or(&format!("{:?}", prioritized.id)),
                prioritized.priority.value(),
                self.name
            );

            // Execute handle and chain the result
            data = prioritized.handle.handle(data, context).await
                .map_err(|e| {
                    tracing::error!(
                        "Handle {:?} failed for hook '{}': {}",
                        prioritized.name.as_ref().unwrap_or(&format!("{:?}", prioritized.id)),
                        self.name,
                        e
                    );
                    e
                })?;
        }

        Ok(data)
    }

    /// Get the number of registered handles
    pub fn handle_count(&self) -> usize {
        self.handles.read().map(|h| h.len()).unwrap_or(0)
    }

    /// Get the hook name
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl<T: Clone + 'static> Default for Hook<T> {
    fn default() -> Self {
        Self::new("unnamed".to_string())
    }
}

/// Hook identifier for the registry
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HookIdentifier {
    Predefined(PredefinedHook),
    Custom(String),
}

impl std::fmt::Display for HookIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookIdentifier::Predefined(hook) => write!(f, "{:?}", hook),
            HookIdentifier::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Predefined hooks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PredefinedHook {
    OnPluginLoad,
    OnPluginUnload,
    BeforeRequest,
    AfterRequest,
}

/// Registry for managing hooks
/// Plugins can register hooks and handles through this registry
pub struct HookRegistry {
    hooks: RwLock<HashMap<HookIdentifier, Arc<dyn Any + Send + Sync>>>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self {
            hooks: RwLock::new(HashMap::new()),
        }
    }

    /// Register or get a hook with a specific identifier and type
    pub fn register_hook<T: Clone + 'static>(&self, hook_id: HookIdentifier) -> Result<Arc<Hook<T>>, PluginError> {
        let mut hooks = self.hooks.write()
            .map_err(|e| PluginError::HookError(format!("Failed to acquire write lock: {}", e)))?;

        if let Some(hook) = hooks.get(&hook_id) {
            return hook
                .clone()
                .downcast::<Hook<T>>()
                .map_err(|_| PluginError::HookError(format!("Type mismatch for hook '{}'", hook_id)));
        }

        let hook = Arc::new(Hook::<T>::new(hook_id.to_string()));
        hooks.insert(hook_id, hook.clone());
        Ok(hook)
    }

    /// Get an existing hook
    pub fn get_hook<T: Clone + 'static>(&self, hook_id: &HookIdentifier) -> Result<Arc<Hook<T>>, PluginError> {
        let hooks = self.hooks.read()
            .map_err(|e| PluginError::HookError(format!("Failed to acquire read lock: {}", e)))?;

        hooks
            .get(hook_id)
            .and_then(|hook| hook.clone().downcast::<Hook<T>>().ok())
            .ok_or_else(|| {
                PluginError::HookError(format!("Hook '{}' not found", hook_id))
            })
    }

    /// Check if a hook is registered
    pub fn has_hook(&self, hook_id: &HookIdentifier) -> bool {
        self.hooks
            .read()
            .map(|hooks| hooks.contains_key(hook_id))
            .unwrap_or(false)
    }

    /// Get a list of all registered hook identifiers
    pub fn list_hooks(&self) -> Vec<HookIdentifier> {
        self.hooks
            .read()
            .map(|hooks| hooks.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Get the number of registered hooks
    pub fn hook_count(&self) -> usize {
        self.hooks.read().map(|h| h.len()).unwrap_or(0)
    }
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[derive(Debug, Clone)]
    struct TestData {
        value: i32,
    }

    fn mock_context() -> PluginContext {
        PluginContext::new()
    }

    #[test]
    fn test_hook_priority() {
        assert!(HookPriority::HIGHEST > HookPriority::HIGH);
        assert!(HookPriority::HIGH > HookPriority::NORMAL);
        assert!(HookPriority::NORMAL > HookPriority::LOW);
        assert!(HookPriority::LOW > HookPriority::LOWEST);
    }

    #[tokio::test]
    async fn test_hook_chaining() {
        let hook = Hook::<TestData>::new("test_chain".to_string());
        let context = mock_context();

        // Register handles that transform the data
        hook.register_handle(
            HookPriority::HIGH,
            Arc::new(|mut data: TestData, _ctx: &PluginContext| async move {
                data.value += 10;
                Ok(data)
            }),
            Some("add_10".to_string()),
        ).unwrap();

        hook.register_handle(
            HookPriority::NORMAL,
            Arc::new(|mut data: TestData, _ctx: &PluginContext| async move {
                data.value *= 2;
                Ok(data)
            }),
            Some("multiply_2".to_string()),
        ).unwrap();

        hook.register_handle(
            HookPriority::LOW,
            Arc::new(|mut data: TestData, _ctx: &PluginContext| async move {
                data.value -= 5;
                Ok(data)
            }),
            Some("subtract_5".to_string()),
        ).unwrap();

        let initial = TestData { value: 5 };
        let result = hook.trigger(initial, &context).await.unwrap();

        // Expected: (5 + 10) * 2 - 5 = 25
        assert_eq!(result.value, 25);
    }

    #[tokio::test]
    async fn test_hook_failure_stops_chain() {
        let hook = Hook::<TestData>::new("test_failure".to_string());
        let context = mock_context();

        hook.register_handle(
            HookPriority::HIGH,
            Arc::new(|mut data: TestData, _ctx: &PluginContext| async move {
                data.value += 10;
                Ok(data)
            }),
            Some("add_10".to_string()),
        ).unwrap();

        hook.register_handle(
            HookPriority::NORMAL,
            Arc::new(|_data: TestData, _ctx: &PluginContext| async move {
                Err(PluginError::HookError("Intentional failure".to_string()))
            }),
            Some("fail".to_string()),
        ).unwrap();

        hook.register_handle(
            HookPriority::LOW,
            Arc::new(|mut data: TestData, _ctx: &PluginContext| async move {
                data.value *= 100; // Should never execute
                Ok(data)
            }),
            Some("multiply_100".to_string()),
        ).unwrap();

        let initial = TestData { value: 5 };
        let result = hook.trigger(initial, &context).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PluginError::HookError(_)));
    }

    #[test]
    fn test_hook_registry() {
        let registry = HookRegistry::new();
        let hook_id = HookIdentifier::Custom("test_hook".to_string());

        assert!(!registry.has_hook(&hook_id));

        let hook = registry.register_hook::<TestData>(hook_id.clone()).unwrap();
        assert!(registry.has_hook(&hook_id));

        let hook2 = registry.get_hook::<TestData>(&hook_id).unwrap();
        assert_eq!(Arc::as_ptr(&hook), Arc::as_ptr(&hook2));
    }

    #[test]
    fn test_list_hooks() {
        let registry = HookRegistry::new();

        let hook1 = HookIdentifier::Predefined(PredefinedHook::OnPluginLoad);
        let hook2 = HookIdentifier::Custom("custom".to_string());

        registry.register_hook::<TestData>(hook1.clone()).unwrap();
        registry.register_hook::<TestData>(hook2.clone()).unwrap();

        let hooks = registry.list_hooks();
        assert_eq!(hooks.len(), 2);
        assert!(hooks.contains(&hook1));
        assert!(hooks.contains(&hook2));
    }
}

