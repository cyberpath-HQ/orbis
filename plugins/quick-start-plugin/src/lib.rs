use orbis_plugin_api::sdk::prelude::*;
use serde_json::json;

// Initialize the plugin with zero boilerplate
orbis_plugin!();

/// Get personalized greeting
fn get_greeting_impl(ctx: Context) -> Result<Response> {
    // Get name from query parameter or use "World"
    let name = ctx.query_param("name").unwrap_or("World");
    
    log::info!("Greeting requested for: {}", name);
    
    Response::json(&json!({
        "message": format!("Hello, {}!", name),
    }))
}

/// Increment click counter
fn increment_count_impl(_ctx: Context) -> Result<Response> {
    // Get current count
    let count: i32 = state::get("count")?.unwrap_or(0);
    let new_count = count + 1;
    
    // Save new count
    state::set("count", &new_count)?;
    
    log::info!("Count incremented to: {}", new_count);
    
    Response::json(&json!({
        "count": new_count,
        "message": format!("Clicked {} times!", new_count)
    }))
}

// Export handlers for FFI
wrap_handler!(get_greeting, get_greeting_impl);
wrap_handler!(increment_count, increment_count_impl);