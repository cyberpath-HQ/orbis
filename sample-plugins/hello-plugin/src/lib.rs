//! Sample Hello World plugin for Orbis

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
}

/// Get the plugin manifest.
/// This function is called to retrieve the embedded manifest.
#[no_mangle]
pub extern "C" fn get_manifest() -> *const u8 {
    let manifest = PluginManifest {
        name: "Hello-Plugin".to_string(),
        version: "0.1.0".to_string(),
        description: "A simple hello world plugin demonstrating Orbis plugin system".to_string(),
        author: "Orbis Team".to_string(),
    };
    
    let json = serde_json::to_string(&manifest).unwrap();
    let bytes = json.into_bytes();
    let ptr = bytes.as_ptr();
    std::mem::forget(bytes); // Prevent deallocation
    ptr
}

/// Initialize the plugin.
#[no_mangle]
pub extern "C" fn init() -> i32 {
    0 // Success
}

/// Execute the plugin's main functionality.
#[no_mangle]
pub extern "C" fn execute() -> i32 {
    // Plugin logic here
    0 // Success
}

/// Clean up plugin resources.
#[no_mangle]
pub extern "C" fn cleanup() -> i32 {
    0 // Success
}
