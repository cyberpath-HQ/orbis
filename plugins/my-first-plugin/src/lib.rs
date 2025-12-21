//! My First Plugin - A simple Hello World plugin with state persistence

use orbis_plugin_api::sdk::prelude::*;
use serde::{Deserialize, Serialize};

// Zero-boilerplate plugin initialization
orbis_plugin!();

#[derive(Debug, Serialize, Deserialize)]
pub struct GreetRequest {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GreetResponse {
    pub message: String,
    pub name: String,
}

/// Get current greeting (restored from state)
fn get_greeting_impl(_ctx: Context) -> Result<Response> {
    // Try to load saved state
    let saved_name: Option<String> = state::get("saved_name")?;
    let saved_message: Option<String> = state::get("saved_message")?;
    
    let response = GreetResponse {
        name: saved_name.unwrap_or_default(),
        message: saved_message.unwrap_or_else(|| "Hello World!".to_string()),
    };
    
    log::info!("Retrieved greeting: {} for {}", response.message, response.name);
    
    Response::json(&response)
}

/// Create a greeting for a name
fn create_greeting_impl(ctx: Context) -> Result<Response> {
    let mut request: GreetRequest = ctx.body_as()?;
    
    let message = if request.name.is_empty() {
        request.name = "World".to_string();
        format!("Hello {}!", request.name)
    } else {
        format!("Hello {}!", request.name)
    };
    
    log::info!("Creating greeting: {} for {}", message, request.name);
    
    // Persist state
    state::set("saved_name", &request.name)?;
    state::set("saved_message", &message)?;
    
    let response = GreetResponse {
        name: request.name,
        message,
    };
    
    Response::json(&response)
}

// Export handlers with wrap_handler! macro
wrap_handler!(get_greeting, get_greeting_impl);
wrap_handler!(create_greeting, create_greeting_impl);

