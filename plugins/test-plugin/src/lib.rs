//! Integration test plugin for Orbis runtime
//!
//! This is a minimal test plugin to verify the runtime works correctly.

/// Initialize the plugin
#[no_mangle]
pub extern "C" fn init() {
    // Empty init function
}

/// Cleanup the plugin
#[no_mangle]
pub extern "C" fn cleanup() {
    // Empty cleanup function
}

/// Simple allocate function for WASM (using global allocator)
#[no_mangle]
pub extern "C" fn allocate(size: i32) -> *mut u8 {
    let mut buf = Vec::with_capacity(size as usize);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

/// Simple deallocate function for WASM
#[no_mangle]
pub extern "C" fn deallocate(ptr: *mut u8, size: i32) {
    unsafe {
        let _ = Vec::from_raw_parts(ptr, 0, size as usize);
    }
}

/// Test handler that processes the context and returns a response
#[no_mangle]
pub extern "C" fn test_handler(ptr: i32, len: i32) -> i32 {
    // Read input
    let input_bytes = unsafe {
        std::slice::from_raw_parts(ptr as *const u8, len as usize)
    };
    
    // Simple echo response
    let response = format!(
        r#"{{"status":"success","data":{{"received_bytes":{},"echo":"ok"}}}}"#,
        input_bytes.len()
    );
    
    let response_bytes = response.as_bytes();
    let response_len = response_bytes.len() as u32;
    
    // Allocate memory for length + data
    let total_size = 4 + response_bytes.len();
    let result_ptr = allocate(total_size as i32);
    
    unsafe {
        // Write length
        *(result_ptr as *mut u32) = response_len;
        
        // Write data
        let data_ptr = result_ptr.add(4);
        std::ptr::copy_nonoverlapping(response_bytes.as_ptr(), data_ptr, response_bytes.len());
    }
    
    result_ptr as i32
}
