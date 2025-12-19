//! Orbis Tauri Application
//!
//! Extensible asset management platform supporting:
//! - Standalone mode: Local database with embedded server
//! - Client-Server mode: Connect to remote Orbis server

mod commands;
mod state;

use orbis_config::{init_config, Config};
use orbis_core::AppMode;
use orbis_server::Server;
use tauri::Manager;

/// Application state shared across Tauri commands.
pub use state::{OrbisState, AuthSession};

/// Initialize logging.
fn init_logging(config: &Config) {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&config.log.level));

    if config.log.format == orbis_config::LogFormat::Json {
        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer())
            .init();
    }
}

/// Run the Tauri application.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize configuration from CLI/env
    let config = match init_config() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to initialize configuration: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize logging
    {
        let config = config.read();
        init_logging(&config);
    }

    tracing::info!("Starting Orbis...");

    // Build and run the Tauri application
    tauri::Builder::default()
        // .plugin(tauri_plugin_updater::Builder::new().build())
        // .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let config_clone = config.clone();

            // Initialize async runtime for setup
            tauri::async_runtime::block_on(async move {
                let config = config_clone.read().clone();

                // Initialize application state based on mode
                let state = match config.mode {
                    AppMode::Standalone => {
                        tracing::info!("Running in standalone mode");
                        init_standalone(&config).await
                    }
                    AppMode::ClientServer => {
                        if config.run_mode.is_server() {
                            tracing::info!("Running as server");
                            init_server(&config, &app_handle).await
                        } else {
                            tracing::info!("Running as client");
                            init_client(&config).await
                        }
                    }
                };

                match state {
                    Ok(state) => {
                        app_handle.manage(state);
                    }
                    Err(e) => {
                        tracing::error!("Failed to initialize: {}", e);
                        std::process::exit(1);
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::health_check,
            commands::get_mode,
            commands::get_profile,
            commands::list_profiles,
            commands::create_profile,
            commands::delete_profile,
            commands::switch_profile,
            commands::get_plugins,
            commands::get_plugin_pages,
            commands::get_plugin_info,
            commands::call_plugin_api,
            commands::reload_plugin,
            commands::enable_plugin,
            commands::disable_plugin,
            commands::install_plugin,
            commands::uninstall_plugin,
            commands::start_plugin_watcher,
            commands::stop_plugin_watcher,
            commands::login,
            commands::logout,
            commands::get_session,
            commands::verify_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Initialize standalone mode (local database + embedded server).
async fn init_standalone(config: &Config) -> orbis_core::Result<OrbisState> {
    // Create the server (handles database, auth, plugins)
    let server = Server::new(config.clone()).await?;

    // In standalone mode, run the HTTP server in background for API access
    let server_state = server.state().clone();

    // Start server in background
    let server_config = config.clone();
    tauri::async_runtime::spawn(async move {
        let server = Server::new(server_config).await.expect("Failed to create server");
        if let Err(e) = server.run().await {
            tracing::error!("Server error: {}", e);
        }
    });

    Ok(OrbisState::new_standalone(
        server_state.db().clone(),
        server_state.auth().cloned(),
        server_state.plugins_arc(),
        config.clone(),
    ))
}

/// Initialize server mode (full server with UI).
async fn init_server(
    config: &Config,
    _app_handle: &tauri::AppHandle,
) -> orbis_core::Result<OrbisState> {
    // Same as standalone but emphasizes server role
    init_standalone(config).await
}

/// Initialize client mode (connect to remote server).
async fn init_client(config: &Config) -> orbis_core::Result<OrbisState> {
    // In client mode, we don't have local database or plugins
    // We connect to a remote server
    let server_url = config
        .server
        .url
        .as_ref()
        .ok_or_else(|| orbis_core::Error::config("Server URL is required in client mode"))?;

    tracing::info!("Connecting to server: {}", server_url);

    Ok(OrbisState::new_client(server_url.clone(), config.clone()))
}

