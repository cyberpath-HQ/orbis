//! FFI declarations and memory management.
//!
//! This module handles all the low-level FFI details so plugin developers don't have to.

use core::slice;

// ============================================================================
// Host function declarations
// ============================================================================

#[cfg(target_arch = "wasm32")]
unsafe extern "C" {
    // State management
    pub fn state_get(key_ptr: *const u8, key_len: i32) -> *const u8;
    pub fn state_set(key_ptr: *const u8, key_len: i32, value_ptr: *const u8, value_len: i32)
        -> i32;
    pub fn state_remove(key_ptr: *const u8, key_len: i32) -> i32;

    // Logging
    pub fn log(level: i32, ptr: *const u8, len: i32);

    // Database (new)
    pub fn db_query(query_ptr: *const u8, query_len: i32, params_ptr: *const u8, params_len: i32) -> *const u8;
    pub fn db_execute(query_ptr: *const u8, query_len: i32, params_ptr: *const u8, params_len: i32) -> i32;

    // HTTP (new)
    pub fn http_request(
        method_ptr: *const u8, method_len: i32,
        url_ptr: *const u8, url_len: i32,
        headers_ptr: *const u8, headers_len: i32,
        body_ptr: *const u8, body_len: i32,
    ) -> *const u8;

    // Events (new)
    pub fn emit_event(event_ptr: *const u8, event_len: i32, payload_ptr: *const u8, payload_len: i32) -> i32;

    // Config (new)
    pub fn get_config(key_ptr: *const u8, key_len: i32) -> *const u8;

    // Crypto (new)
    pub fn crypto_hash(algorithm: i32, data_ptr: *const u8, data_len: i32) -> *const u8;
    pub fn crypto_random(len: i32) -> *const u8;
}

// ============================================================================
// Memory management - Plugin side
// ============================================================================

/// Global allocator for WASM - allocates memory that can be written to by the host
#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn allocate(size: i32) -> *mut u8 {
    let layout = core::alloc::Layout::from_size_align(size as usize, 1).unwrap();
    // SAFETY: We're using the global allocator which is valid in WASM
    unsafe { std::alloc::alloc(layout) }
}

/// Allocate memory (non-WASM stub for compilation)
#[cfg(not(target_arch = "wasm32"))]
pub fn allocate(size: i32) -> *mut u8 {
    let layout = std::alloc::Layout::from_size_align(size as usize, 1).unwrap();
    unsafe { std::alloc::alloc(layout) }
}

/// Deallocate memory previously allocated by `allocate`
#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn deallocate(ptr: *mut u8, size: i32) {
    if ptr.is_null() {
        return;
    }
    let layout = core::alloc::Layout::from_size_align(size as usize, 1).unwrap();
    // SAFETY: ptr was allocated by allocate() with the same layout
    unsafe { std::alloc::dealloc(ptr, layout) }
}

/// Deallocate memory (non-WASM stub)
#[cfg(not(target_arch = "wasm32"))]
pub fn deallocate(ptr: *mut u8, size: i32) {
    if ptr.is_null() {
        return;
    }
    let layout = std::alloc::Layout::from_size_align(size as usize, 1).unwrap();
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
pub unsafe fn read_length_prefixed(ptr: *const u8) -> Vec<u8> {
    if ptr.is_null() {
        return Vec::new();
    }
    unsafe {
        let len = *(ptr as *const u32);
        let data_ptr = ptr.add(4);
        slice::from_raw_parts(data_ptr, len as usize).to_vec()
    }
}

/// Write data with length prefix
///
/// Returns pointer to allocated memory containing: [4 bytes length][data]
#[inline]
pub fn write_length_prefixed(data: &[u8]) -> *mut u8 {
    let len = data.len() as u32;
    let total_size = 4 + data.len();
    let ptr = allocate(total_size as i32);

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
#[macro_export]
macro_rules! wrap_handler {
    ($handler:ident) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn $handler(ctx_ptr: i32, ctx_len: i32) -> i32 {
            use $crate::sdk::prelude::*;

            // Deserialize context
            let ctx = match Context::from_raw(ctx_ptr, ctx_len) {
                Ok(c) => c,
                Err(e) => {
                    log::error!("Failed to parse context: {}", e);
                    return Response::error(400, &format!("Invalid context: {}", e))
                        .to_raw()
                        .unwrap_or(0);
                }
            };

            // Call the actual handler
            match $handler(ctx) {
                Ok(response) => response.to_raw().unwrap_or(0),
                Err(e) => {
                    log::error!("Handler error: {}", e);
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
                    $crate::sdk::log::error!("Init failed: {}", e);
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
                    $crate::sdk::log::error!("Cleanup failed: {}", e);
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
                    $crate::sdk::log::error!("Init failed: {}", e);
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
    };
}

pub use orbis_plugin;
pub use wrap_handler;
