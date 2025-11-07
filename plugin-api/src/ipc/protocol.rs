/// IPC Protocol definitions for plugin communication
use serde::{Deserialize, Serialize};
use std::fmt;

/// Messages exchanged between server and plugin processes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(bincode::Encode, bincode::Decode)]
pub enum IpcMessage {
    // ========== Server → Plugin ==========
    
    /// Initialize the plugin with context
    Initialize {
        context_data: Vec<u8>, // Serialized PluginContext
    },
    
    /// Execute a hook
    ExecuteHook {
        hook_name: String,
        data: Vec<u8>, // Serialized hook data
        timeout_ms: u64,
    },
    
    /// Request plugin to register its hooks
    RegisterHooksRequest,
    
    /// Shutdown the plugin gracefully
    Shutdown {
        grace_period_ms: u64,
    },
    
    /// Health check ping
    Ping,
    
    // ========== Plugin → Server ==========
    
    /// Response to Initialize
    InitializeResponse {
        success: bool,
        error: Option<String>,
    },
    
    /// Response to ExecuteHook
    HookResponse {
        result: Vec<u8>, // Serialized hook result
        error: Option<String>,
    },
    
    /// Register hooks with the server
    RegisterHooks {
        hooks: Vec<HookRegistration>,
    },
    
    /// Acknowledge shutdown
    ShutdownAck,
    
    /// Response to Ping
    Pong,
    
    /// Log message from plugin
    LogMessage {
        level: LogLevel,
        message: String,
        plugin_name: String,
    },
    
    /// Report resource usage
    ResourceUsage {
        heap_bytes: usize,
        cpu_time_ms: u64,
    },

    // ========== Context Access (Plugin → Server) ==========

    /// Request to access context data
    ContextGet {
        key: String,
        request_id: u64,
    },

    /// Request to set context data
    ContextSet {
        key: String,
        data: Vec<u8>, // Serialized data
        request_id: u64,
    },

    /// Check if context key exists
    ContextHas {
        key: String,
        request_id: u64,
    },

    // ========== Context Responses (Server → Plugin) ==========

    /// Response to ContextGet
    ContextGetResponse {
        request_id: u64,
        data: Option<Vec<u8>>, // Serialized data
        error: Option<String>,
    },

    /// Response to ContextSet
    ContextSetResponse {
        request_id: u64,
        success: bool,
        error: Option<String>,
    },

    /// Response to ContextHas
    ContextHasResponse {
        request_id: u64,
        exists: bool,
    },
    
    // ========== Metrics & Monitoring (Server → Plugin / Plugin → Server) ==========
    
    /// Request current metrics from plugin
    MetricsRequest {
        request_id: u64,
    },
    
    /// Response with current metrics
    MetricsResponse {
        request_id: u64,
        metrics: Option<Vec<u8>>, // Serialized PluginMetrics
        error: Option<String>,
    },
    
    /// Notify server of termination event
    TerminationEvent {
        event_data: Vec<u8>, // Serialized TerminationEvent
    },
    
    /// Plugin is about to be terminated (warning from server)
    TerminationWarning {
        reason: String,
        grace_period_ms: u64,
    },
}

/// Hook registration information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(bincode::Encode, bincode::Decode)]
pub struct HookRegistration {
    pub name: String,
    pub priority: u8,
    pub timeout_ms: Option<u64>,
}

/// Log levels for plugin messages
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[derive(bincode::Encode, bincode::Decode)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

/// IPC-specific errors
#[derive(Debug, thiserror::Error)]
pub enum IpcError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Encode error: {0}")]
    EncodeError(#[from] bincode::error::EncodeError),

    #[error("Decode error: {0}")]
    DecodeError(#[from] bincode::error::DecodeError),

    #[error("Connection closed")]
    ConnectionClosed,
    
    #[error("Timeout after {0}ms")]
    Timeout(u64),
    
    #[error("Protocol error: {0}")]
    Protocol(String),
    
    #[error("Channel error: {0}")]
    ChannelError(String),
}

/// Message framing for length-prefixed protocol
/// 
/// Format: [4-byte length][message bytes]
#[derive(Debug)]
pub struct MessageFrame {
    pub data: Vec<u8>,
}

impl MessageFrame {
    /// Create frame from message
    pub fn from_message(msg: &IpcMessage) -> Result<Self, IpcError> {
        let data = bincode::encode_to_vec(msg, bincode::config::standard())?;
        Ok(Self { data })
    }
    
    /// Parse message from frame
    pub fn to_message(&self) -> Result<IpcMessage, IpcError> {
        let (msg, _len) = bincode::decode_from_slice(&self.data, bincode::config::standard())?;
        Ok(msg)
    }
    
    /// Encode frame with length prefix
    pub fn encode(&self) -> Vec<u8> {
        let len = self.data.len() as u32;
        let mut buf = Vec::with_capacity(4 + self.data.len());
        buf.extend_from_slice(&len.to_be_bytes());
        buf.extend_from_slice(&self.data);
        buf
    }
    
    /// Get expected total size (including length prefix)
    pub fn total_size(&self) -> usize {
        4 + self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_message_serialization() {
        let msg = IpcMessage::Ping;
        let frame = MessageFrame::from_message(&msg).unwrap();
        let encoded = frame.encode();
        
        // Length should be first 4 bytes
        assert_eq!(encoded.len(), frame.total_size());
        
        // Decode and verify
        let decoded = frame.to_message().unwrap();
        matches!(decoded, IpcMessage::Ping);
    }
    
    #[test]
    fn test_hook_registration() {
        let hooks = vec![
            HookRegistration {
                name: "before_request".to_string(),
                priority: 5,
                timeout_ms: Some(1000),
            },
        ];
        
        let msg = IpcMessage::RegisterHooks { hooks };
        let frame = MessageFrame::from_message(&msg).unwrap();
        let decoded = frame.to_message().unwrap();
        
        match decoded {
            IpcMessage::RegisterHooks { hooks } => {
                assert_eq!(hooks.len(), 1);
                assert_eq!(hooks[0].name, "before_request");
            }
            _ => panic!("Wrong message type"),
        }
    }
}

