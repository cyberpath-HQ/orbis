use thiserror::Error;

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Failed to load plugin library: {0}")]
    LoadError(String),
    
    #[error("Plugin symbol not found: {0}")]
    SymbolNotFound(String),
    
    #[error("Plugin initialization failed: {0}")]
    InitializationError(String),
    
    #[error("Plugin already loaded: {0}")]
    AlreadyLoaded(String),
    
    #[error("Plugin not found: {0}")]
    NotFound(String),
    
    #[error("Security validation failed: {0}")]
    SecurityError(String),
    
    /// START: Signature related errors
    #[error("Signature related operation failed: {0}")]
    SignatureError(#[from] signer::errors::SignerErrors),
    
    #[error("Public key not in allowed list")]
    InvalidPublicKey,
    
    #[error("Invalid plugin signature")]
    InvalidSignature,
    /// END: Signature related errors
    
    #[error("Resource limit violation: {0}")]
    ResourceLimitError(String),
    
    #[error("Hook execution timeout: {0}")]
    HookTimeoutError(String),
    
    #[error("Hook registration failed: {0}")]
    HookError(String),
    
    #[error("Plugin is not trusted")]
    UntrustedPlugin,
    
    #[error("Plugin is not whitelisted: {0}")]
    NotWhitelisted(String),
    
    #[error("Plugin version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: String, actual: String },
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Plugin ABI incompatible")]
    AbiIncompatible,

    #[error("IPC protocol error: {0}")]
    Protocol(String),

    #[error("IPC timeout: {0}ms")]
    Timeout(u64),

    #[error("IPC error: {0}")]
    IpcError(String),
}

// Convert IpcError to PluginError
impl From<crate::ipc::IpcError> for PluginError {
    fn from(err: crate::ipc::IpcError) -> Self {
        PluginError::IpcError(err.to_string())
    }
}

