/// Bridge module to convert between API types and internal types
use std::sync::Arc;
use crate::{BridgedPlugin, HookRegistry, Plugin, PluginContext, PluginError, ResourceLimits};

/// Wrapper that converts API Plugin to internal Plugin
pub struct PluginBridge {
    inner: Box<dyn Plugin>,
}

impl PluginBridge {
    pub fn new(plugin: Box<dyn Plugin>) -> Self {
        Self { inner: plugin }
    }
}

#[async_trait::async_trait(?Send)]
impl BridgedPlugin for PluginBridge {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn version(&self) -> &str {
        self.inner.version()
    }

    fn author(&self) -> &str {
        self.inner.author()
    }

    fn description(&self) -> Option<&str> {
        self.inner.description()
    }

    fn resource_limits(&self) -> Option<ResourceLimits> {
        self.inner.resource_limits()
    }
    
    fn requirements(&self) -> crate::PluginRequirements {
        self.inner.requirements()
    }

    async fn init(&mut self, context: Arc<PluginContext>) -> Result<(), PluginError> {
        let context_ptr = Arc::into_raw(context.clone()) as *const ();
        self.inner.init(context_ptr).await?;
        // Keep the Arc alive by reconstructing it
        unsafe { Arc::from_raw(context_ptr as *const PluginContext) };
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        self.inner.shutdown().await
    }

    async fn register_hooks(&self, hook_registry: &mut HookRegistry) -> Result<(), PluginError> {
        let registry_ptr = hook_registry as *mut HookRegistry as *mut ();
        self.inner.register_hooks(registry_ptr).await
    }
}

