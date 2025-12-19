//! Sample Hello World plugin for Orbis

use orbis_plugin_api::sdk::prelude::*;
use serde_json::json;

// Zero-boilerplate plugin initialization
orbis_plugin!();

/// Get a personalized greeting
fn get_greeting_impl(ctx: Context) -> Result<Response> {
    let name = ctx.query_param("name").unwrap_or("World");
    
    log::info!("Greeting requested for: {}", name);
    
    Response::json(&json!({
        "message": format!("Hello, {}!", name)
    }))
}

/// Get plugin information
fn get_info_impl(_ctx: Context) -> Result<Response> {
    Response::json(&json!({
        "name": "Hello Plugin",
        "version": "0.1.0",
        "description": "A simple hello world plugin demonstrating Orbis plugin system",
        "author": "Orbis Team"
    }))
}

// Export handlers with wrap_handler! macro
wrap_handler!(get_greeting, get_greeting_impl);
wrap_handler!(get_info, get_info_impl);
