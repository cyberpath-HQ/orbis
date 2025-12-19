//! Integration test plugin for Orbis runtime
//!
//! This is a minimal test plugin to verify the runtime works correctly.

use orbis_plugin_api::sdk::prelude::*;
use serde_json::json;

// Zero-boilerplate plugin initialization
orbis_plugin!();

/// Test handler that echoes back request information
fn test_handler_impl(ctx: Context) -> Result<Response> {
    log::info!("Test handler called: {} {}", ctx.method, ctx.path);
    
    Response::json(&json!({
        "status": "success",
        "method": ctx.method,
        "path": ctx.path,
        "message": "Test handler executed successfully"
    }))
}

/// State test handler - demonstrates state management
fn state_test_impl(_ctx: Context) -> Result<Response> {
    let counter: i32 = state::get("counter")?.unwrap_or(0);
    let new_counter = counter + 1;
    state::set("counter", &new_counter)?;
    
    log::info!("State counter incremented to: {}", new_counter);
    
    Response::json(&json!({
        "counter": new_counter,
        "message": "State updated successfully"
    }))
}

// Export handlers with wrap_handler! macro
wrap_handler!(test_handler, test_handler_impl);
wrap_handler!(state_test, state_test_impl);
