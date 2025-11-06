use axum::Json;
use axum::Router;
use axum::routing::get;
use num_traits::Unsigned;
use rustls_pemfile::{certs, pkcs8_private_keys};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};
use std::{env, fs::File, io::BufReader, str::FromStr, sync::Arc, time::Duration};
use thiserror::Error;
use tokio_rustls::rustls::crypto::CryptoProvider;
use tokio_rustls::rustls::{ServerConfig, crypto};
use tracing::{error, info, warn};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Error, Debug)]
enum AppError {
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),

    #[error("Invalid value for environment variable {0}: {1}")]
    InvalidEnvValue(String, String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sea_orm::DbErr),

    #[error("Server error: {0}")]
    ServerError(#[from] std::io::Error),

    #[error("TLS configuration error: {0}")]
    TlsError(String),
}

#[derive(Serialize, Deserialize)]
struct HealthResponse {
    status: String,
}

/// Parses an environment variable and converts it to the specified unsigned type T.
///
/// # Arguments
/// * `variable_name` - The name of the environment variable to parse.
///
/// # Returns
/// The parsed value of type T.
fn parse_env_var<T>(variable_name: &str) -> Result<T, AppError>
where
    T: Unsigned + FromStr,
    T::Err: std::fmt::Debug,
{
    let value =
        env::var(variable_name).map_err(|_| AppError::MissingEnvVar(variable_name.to_string()))?;

    value
        .parse::<T>()
        .map_err(|e| AppError::InvalidEnvValue(variable_name.to_string(), format!("{:?}", e)))
}

/// Initializes the database connection using environment variables for configuration.
///
/// # Returns
/// A Result containing the DatabaseConnection or a DbErr.
async fn init_db() -> Result<DatabaseConnection, AppError> {
    let db_url = env::var("ORBIS_DB_URL")
        .map_err(|_| AppError::MissingEnvVar("ORBIS_DB_URL".to_string()))?;

    let schema = env::var("ORBIS_DB_SCHEMA").unwrap_or_else(|_| "public".to_owned());

    let mut options = ConnectOptions::new(db_url);
    options
        .max_connections(parse_env_var("ORBIS_DB_MAX_CONNECTIONS")?)
        .min_connections(parse_env_var("ORBIS_DB_MIN_CONNECTIONS")?)
        .connect_timeout(Duration::from_millis(parse_env_var(
            "ORBIS_DB_CONNECT_TIMEOUT_MS",
        )?))
        .acquire_timeout(Duration::from_millis(parse_env_var(
            "ORBIS_DB_ACQUIRE_TIMEOUT_MS",
        )?))
        .idle_timeout(Duration::from_millis(parse_env_var(
            "ORBIS_DB_IDLE_TIMEOUT_MS",
        )?))
        .max_lifetime(Duration::from_millis(parse_env_var(
            "ORBIS_DB_MAX_LIFETIME_MS",
        )?))
        .set_schema_search_path(schema)
        .sqlx_logging(false)
        .test_before_acquire(true);

    Ok(Database::connect(options).await?)
}

/// Health check endpoint handler.
///
/// # Returns
/// A JSON response indicating the health status.
async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "OK".to_owned(),
    })
}

/// Initializes tracing for logging.
fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_env("ORBIS_LOG")
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

/// Initializes SSL/TLS configuration if certificate and key paths are provided.
///
/// # Returns
/// An Option containing the ServerConfig if SSL is configured, or None if not.
fn init_ssl_config() -> Result<Option<Arc<ServerConfig>>, AppError> {
    let cert_path = env::var("ORBIS_TLS_CERT_PATH").ok();
    let key_path = env::var("ORBIS_TLS_KEY_PATH").ok();

    let _ = CryptoProvider::install_default(crypto::ring::default_provider())
        .map_err(|e| AppError::TlsError("Failed to install crypto provider".to_owned()));

    match (cert_path, key_path) {
        (Some(cert_path), Some(key_path)) => {
            info!("Loading TLS configuration");

            let cert_file = File::open(&cert_path).map_err(|e| {
                AppError::TlsError(format!(
                    "Failed to open certificate file {}: {}",
                    cert_path, e
                ))
            })?;
            let key_file = File::open(&key_path).map_err(|e| {
                AppError::TlsError(format!("Failed to open key file {}: {}", key_path, e))
            })?;

            let mut cert_reader = BufReader::new(cert_file);
            let mut key_reader = BufReader::new(key_file);

            let certs = certs(&mut cert_reader)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| AppError::TlsError(format!("Failed to parse certificate: {}", e)))?;

            let mut keys = pkcs8_private_keys(&mut key_reader)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| AppError::TlsError(format!("Failed to parse private key: {}", e)))?;

            if keys.is_empty() {
                return Err(AppError::TlsError(
                    "No private keys found in key file".to_string(),
                ));
            }

            let config = ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(certs, keys.remove(0).into())
                .map_err(|e| AppError::TlsError(format!("Failed to build TLS config: {}", e)))?;

            info!("TLS configuration loaded successfully");
            Ok(Some(Arc::new(config)))
        }
        (None, None) => {
            warn!("TLS not configured, running in HTTP mode");
            Ok(None)
        }
        (Some(_), None) => Err(AppError::TlsError(
            "ORBIS_TLS_CERT_PATH provided but ORBIS_TLS_KEY_PATH is missing".to_string(),
        )),
        (None, Some(_)) => Err(AppError::TlsError(
            "ORBIS_TLS_KEY_PATH provided but ORBIS_TLS_CERT_PATH is missing".to_string(),
        )),
    }
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        error!("{}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<(), AppError> {
    #[cfg(feature = "dev")]
    dotenv::dotenv().ok();

    init_tracing();

    info!("Initializing database connection");
    let _db = init_db().await?;
    info!("Database connection established");

    let app = Router::new().route("/health", get(health_check));

    let addr = format!(
        "{}:{}",
        env::var("ORBIS_SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_owned()),
        env::var("ORBIS_SERVER_PORT").unwrap_or_else(|_| "8000".to_owned())
    );

    let tls_config = init_ssl_config()?;

    match tls_config {
        Some(config) => {
            info!("Starting HTTPS server at https://{}", addr);

            let rustls_config = axum_server::tls_rustls::RustlsConfig::from_config(config);

            // Note: axum_server::bind_rustls requires the address to be in the format `IP:PORT`
            // The syntax `localhost:8000` is NOT supported for TLS binding.
            axum_server::bind_rustls(
                addr.parse().map_err(|e| {
                    AppError::InvalidEnvValue(
                        "ORBIS_SERVER_HOST/ORBIS_SERVER_PORT".to_owned(),
                        format!("{}", e),
                    )
                })?,
                rustls_config,
            )
            .serve(app.into_make_service())
            .await?;
        }
        None => {
            info!("Starting HTTP server at http://{}", addr);
            let listener = tokio::net::TcpListener::bind(addr).await?;
            axum::serve(listener, app).await?;
        }
    }

    Ok(())
}
