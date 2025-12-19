//! My First Plugin - A starter plugin template

use orbis_plugin_api::sdk::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

// Zero-boilerplate plugin initialization
orbis_plugin!();

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub content: String,
}

/// Get welcome message
fn get_data_impl(_ctx: Context) -> Result<Response> {
    let message = Message {
        content: "Hello from My First Plugin!".to_string(),
    };
    
    log::info!("Sending welcome message");
    
    Response::json(&message)
}

/// Create a new message
fn create_message_impl(ctx: Context) -> Result<Response> {
    let message: Message = ctx.body_as()?;
    
    log::info!("Received message: {}", message.content);
    
    // Store in state
    state::set("last_message", &message)?;
    
    Response::json(&json!({
        "success": true,
        "message": message
    }))
}

// Export handlers with wrap_handler! macro
wrap_handler!(get_data, get_data_impl);
wrap_handler!(create_message, create_message_impl);

