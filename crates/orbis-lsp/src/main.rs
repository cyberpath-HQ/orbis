//! Orbis LSP - Language Server for Orbis DSL
//!
//! This is the entry point for the Orbis Language Server. It sets up logging
//! and initializes the LSP server using tower-lsp.
//!
//! # Usage
//!
//! ```bash
//! # Run with default settings
//! orbis-lsp
//!
//! # Run with debug logging
//! RUST_LOG=debug orbis-lsp
//! ```

use tower_lsp::{LspService, Server};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use orbis_lsp::OrbisBackend;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "orbis_lsp=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    tracing::info!("Starting Orbis Language Server");

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(OrbisBackend::new);

    Server::new(stdin, stdout, socket).serve(service).await;
}
