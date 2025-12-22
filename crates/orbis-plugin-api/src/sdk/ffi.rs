//! FFI declarations and memory management.
//!
//! This module handles all the low-level FFI details so plugin developers don't have to.

use core::slice;

// ============================================================================
// Host function declarations
// ============================================================================

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
unsafe extern "C" {
    // State management
    pub fn state_get(key_ptr: i32, key_len: i32) -> i32;
    pub fn state_set(key_ptr: i32, key_len: i32, value_ptr: i32, value_len: i32) -> i32;
    pub fn state_remove(key_ptr: i32, key_len: i32) -> i32;

    // Logging
    pub fn log(level: i32, ptr: i32, len: i32);

    // Database (new)
    pub fn db_query(query_ptr: i32, query_len: i32, params_ptr: i32, params_len: i32) -> i32;
    pub fn db_execute(query_ptr: i32, query_len: i32, params_ptr: i32, params_len: i32) -> i32;

    // HTTP (new)
    pub fn http_request(
        method_ptr: i32,
        method_len: i32,
        url_ptr: i32,
        url_len: i32,
        headers_ptr: i32,
        headers_len: i32,
        body_ptr: i32,
        body_len: i32,
    ) -> i32;

    // Events (new)
    pub fn emit_event(event_ptr: i32, event_len: i32, payload_ptr: i32, payload_len: i32) -> i32;

    // Config (new)
    pub fn get_config(key_ptr: i32, key_len: i32) -> i32;

    // Crypto (new)
    pub fn crypto_hash(algorithm: i32, data_ptr: i32, data_len: i32) -> i32;
    pub fn crypto_random(len: i32) -> i32;
}

/// Shadow implementation of the log function for non-WASM targets
#[cfg(not(target_arch = "wasm32"))]
pub fn log(level: i32, ptr: i32, len: i32) {
    let message = unsafe {
        let slice = slice::from_raw_parts(ptr as *const u8, len as usize);
        std::str::from_utf8(slice).unwrap_or("<invalid utf8>")
    };
    let level_str = match level {
        0 => "ERROR",
        1 => "WARN",
        2 => "INFO",
        3 => "DEBUG",
        _ => "TRACE",
    };
    eprintln!("[{}] {}", level_str, message);
}

// ============================================================================
// Memory management - Plugin side
// ============================================================================

/// Internal allocator for WASM - exported through orbis_plugin! macro
pub fn allocate_internal(size: i32) -> *mut u8 {
    let layout = core::alloc::Layout::from_size_align(size as usize, 1).unwrap();
    // SAFETY: We're using the global allocator which is valid in WASM
    unsafe { std::alloc::alloc(layout) }
}

/// Internal deallocator - exported through orbis_plugin! macro
pub fn deallocate_internal(ptr: *mut u8, size: i32) {
    if ptr.is_null() {
        return;
    }
    let layout = core::alloc::Layout::from_size_align(size as usize, 1).unwrap();
    // SAFETY: ptr was allocated by allocate() with the same layout
    unsafe { std::alloc::dealloc(ptr, layout) }
}

// ============================================================================
// Memory utilities
// ============================================================================

/// Read bytes from a raw pointer with length
///
/// # Safety
/// Caller must ensure ptr is valid for len bytes
#[inline]
pub unsafe fn read_bytes(ptr: *const u8, len: usize) -> Vec<u8> {
    if ptr.is_null() || len == 0 {
        return Vec::new();
    }
    unsafe { slice::from_raw_parts(ptr, len).to_vec() }
}

/// Read a length-prefixed value from a pointer
///
/// Format: [4 bytes u32 LE length][data]
///
/// # Safety
/// Caller must ensure ptr points to valid length-prefixed data
#[inline]
pub unsafe fn read_length_prefixed(ptr: i32) -> Vec<u8> {
    if ptr == 0 {
        return Vec::new();
    }
    
    let ptr = ptr as *const u8;
    
    unsafe {
        // Read the length prefix
        let len_bytes = slice::from_raw_parts(ptr, 4);
        let len = u32::from_le_bytes([len_bytes[0], len_bytes[1], len_bytes[2], len_bytes[3]]);
        
        // Validate length (prevent reading huge invalid values)
        // Max 10MB for safety
        const MAX_LENGTH: u32 = 10 * 1024 * 1024;
        if len > MAX_LENGTH {
            log(0, "Invalid length in read_length_prefixed".as_ptr() as i32, "Invalid length in read_length_prefixed".len() as i32);
            return Vec::new();
        }
        
        // Read the actual data
        if len == 0 {
            return Vec::new();
        }
        
        let data_ptr = ptr.add(4);
        let data_slice = slice::from_raw_parts(data_ptr, len as usize);
        
        // Use Vec::from to safely copy the data
        Vec::from(data_slice)
    }
}

/// Write data with length prefix
///
/// Returns pointer to allocated memory containing: [4 bytes length][data]
#[inline]
pub fn write_length_prefixed(data: &[u8]) -> *mut u8 {
    let len = data.len() as u32;
    let total_size = 4 + data.len();
    let ptr = allocate_internal(total_size as i32);

    unsafe {
        // Write length prefix
        *(ptr as *mut u32) = len;
        // Write data
        let data_ptr = ptr.add(4);
        core::ptr::copy_nonoverlapping(data.as_ptr(), data_ptr, data.len());
    }

    ptr
}

/// Write a string/bytes and return ptr as i32 (for returning from handlers)
#[inline]
pub fn return_bytes(data: &[u8]) -> i32 {
    write_length_prefixed(data) as i32
}

/// Write JSON and return ptr as i32
#[inline]
pub fn return_json<T: serde::Serialize>(value: &T) -> Result<i32, serde_json::Error> {
    let json = serde_json::to_vec(value)?;
    Ok(return_bytes(&json))
}

// ============================================================================
// Handler wrapper macro
// ============================================================================

/// Wraps a handler function to handle FFI details automatically
///
/// Converts: `fn(Context) -> Result<Response>` into `extern "C" fn(i32, i32) -> i32`
///
/// # Usage
///
/// ```rust,ignore
/// // Define your handler function
/// fn my_handler_impl(ctx: Context) -> Result<Response> {
///     Ok(Response::json(&json!({"status": "ok"}))?)
/// }
///
/// // Wrap it for FFI export
/// wrap_handler!(my_handler, my_handler_impl);
/// ```
#[macro_export]
macro_rules! wrap_handler {
    ($export_name:ident, $handler_fn:ident) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn $export_name(ctx_ptr: i32, ctx_len: i32) -> i32 {
            use $crate::sdk::prelude::*;

            // Deserialize context
            let ctx = match Context::from_raw(ctx_ptr, ctx_len) {
                Ok(c) => c,
                Err(e) => {
                    // Log the error using the logging FFI
                    let error_message = format!("Failed to parse context: {}", e);
                    unsafe { $crate::sdk::ffi::log(0, error_message.as_ptr() as i32, error_message.len() as i32); }
                    return Response::error(400, &format!("Invalid context: {}", e))
                        .to_raw()
                        .unwrap_or(0);
                }
            };

            // Call the actual handler
            match $handler_fn(ctx) {
                Ok(response) => response.to_raw().unwrap_or(0),
                Err(e) => {
                    let error_message = format!("Handler error: {}", e);
                    unsafe { $crate::sdk::ffi::log(0, error_message.as_ptr() as i32, error_message.len() as i32); }
                    Response::error(500, &e.to_string())
                        .to_raw()
                        .unwrap_or(0)
                }
            }
        }
    };
}

/// Define a complete plugin with minimal boilerplate
///
/// # Example
///
/// ```rust,ignore
/// use orbis_plugin_api::prelude::*;
///
/// orbis_plugin! {
///     init: || {
///         log::info!("Plugin starting!");
///         state::set("initialized", &true)?;
///         Ok(())
///     },
///     cleanup: || {
///         log::info!("Plugin stopping!");
///         Ok(())
///     }
/// }
/// ```
#[macro_export]
macro_rules! orbis_plugin {
    // With init and cleanup
    (
        init: $init:expr,
        cleanup: $cleanup:expr $(,)?
    ) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn init() -> i32 {
            let init_fn: fn() -> $crate::sdk::Result<()> = $init;
            match init_fn() {
                Ok(()) => 1,
                Err(e) => {
                    let error_message = format!("Init failed: {}", e);
                    unsafe { $crate::sdk::ffi::log(0, error_message.as_ptr() as i32, error_message.len() as i32); }
                    0
                }
            }
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn cleanup() -> i32 {
            let cleanup_fn: fn() -> $crate::sdk::Result<()> = $cleanup;
            match cleanup_fn() {
                Ok(()) => 1,
                Err(e) => {
                    let error_message = format!("Cleanup failed: {}", e);
                    unsafe { $crate::sdk::ffi::log(0, error_message.as_ptr() as i32, error_message.len() as i32); }
                    0
                }
            }
        }
    };

    // With only init
    (init: $init:expr $(,)?) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn init() -> i32 {
            let init_fn: fn() -> $crate::sdk::Result<()> = $init;
            match init_fn() {
                Ok(()) => 1,
                Err(e) => {
                    let error_message = format!("Init failed: {}", e);
                    unsafe { $crate::sdk::ffi::log(0, error_message.as_ptr() as i32, error_message.len() as i32); }
                    0
                }
            }
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn cleanup() -> i32 {
            1 // No-op cleanup
        }
    };

    // No init or cleanup (just lifecycle stubs)
    () => {
        #[unsafe(no_mangle)]
        pub extern "C" fn init() -> i32 {
            1
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn cleanup() -> i32 {
            1
        }
        
        // Memory management functions - defined directly to avoid optimization issues
        #[unsafe(no_mangle)]
        #[inline(never)]
        pub extern "C" fn allocate(size: i32) -> *mut u8 {
            use core::alloc::Layout;
            let layout = Layout::from_size_align(size as usize, 1).unwrap();
            // SAFETY: We're using the global allocator which is valid in WASM
            unsafe { std::alloc::alloc(layout) }
        }
        
        #[unsafe(no_mangle)]
        #[inline(never)]
        pub extern "C" fn deallocate(ptr: *mut u8, size: i32) {
            if ptr.is_null() {
                return;
            }
            use core::alloc::Layout;
            let layout = Layout::from_size_align(size as usize, 1).unwrap();
            // SAFETY: ptr was allocated by allocate() with the same layout
            unsafe { std::alloc::dealloc(ptr, layout) }
        }
    };
}

pub use orbis_plugin;
pub use wrap_handler;
