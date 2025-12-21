//! Logging utilities for plugins.
//!
//! Provides convenient logging macros that work in WASM.
//!
//! # Example
//!
//! ```rust,ignore
//! use orbis_plugin_api::sdk::log;
//!
//! log::info!("Processing request for user {}", user_id);
//! log::error!("Failed to process: {}", error);
//! log::debug!("Debug info: {:?}", data);
//! ```

/// Log level constants matching the host runtime
pub mod level {
    pub const ERROR: i32 = 0;
    pub const WARN: i32 = 1;
    pub const INFO: i32 = 2;
    pub const DEBUG: i32 = 3;
    pub const TRACE: i32 = 4;
}

/// Log a message at the specified level
#[cfg(target_arch = "wasm32")]
#[inline]
pub fn log_at_level(level: i32, message: &str) {
    unsafe {
        super::ffi::log(level, message.as_ptr() as i32, message.len() as i32);
    }
}

/// Log a message (non-WASM stub - prints to stderr)
#[cfg(not(target_arch = "wasm32"))]
#[inline]
pub fn log_at_level(level: i32, message: &str) {
    let level_str = match level {
        0 => "ERROR",
        1 => "WARN",
        2 => "INFO",
        3 => "DEBUG",
        _ => "TRACE",
    };
    eprintln!("[{}] {}", level_str, message);
}

/// Log an error message
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::sdk::log::log_at_level($crate::sdk::log::level::ERROR, &format!($($arg)*))
    };
}

/// Log a warning message
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::sdk::log::log_at_level($crate::sdk::log::level::WARN, &format!($($arg)*))
    };
}

/// Log an info message
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::sdk::log::log_at_level($crate::sdk::log::level::INFO, &format!($($arg)*))
    };
}

/// Log a debug message
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::sdk::log::log_at_level($crate::sdk::log::level::DEBUG, &format!($($arg)*))
    };
}

/// Log a trace message
#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {
        $crate::sdk::log::log_at_level($crate::sdk::log::level::TRACE, &format!($($arg)*))
    };
}

// Re-export macros for convenient access
pub use log_debug as debug;
pub use log_error as error;
pub use log_info as info;
pub use log_trace as trace;
pub use log_warn as warn;
