//! TLS configuration.

use orbis_config::TlsConfig;
use rustls::ServerConfig;
use std::fs::File;
use std::io::BufReader;

/// Create TLS server configuration.
///
/// # Errors
///
/// Returns an error if TLS configuration fails.
pub fn create_tls_config(config: &TlsConfig) -> orbis_core::Result<ServerConfig> {
    let cert_path = config.cert_path.as_ref().ok_or_else(|| {
        orbis_core::Error::config("TLS certificate path is required")
    })?;

    let key_path = config.key_path.as_ref().ok_or_else(|| {
        orbis_core::Error::config("TLS key path is required")
    })?;

    // Load certificate chain
    let cert_file = File::open(cert_path).map_err(|e| {
        orbis_core::Error::config(format!("Failed to open certificate file: {}", e))
    })?;
    let mut cert_reader = BufReader::new(cert_file);
    let certs: Vec<_> = rustls_pemfile::certs(&mut cert_reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| orbis_core::Error::config(format!("Failed to parse certificates: {}", e)))?;

    // Load private key
    let key_file = File::open(key_path).map_err(|e| {
        orbis_core::Error::config(format!("Failed to open key file: {}", e))
    })?;
    let mut key_reader = BufReader::new(key_file);

    let key = rustls_pemfile::private_key(&mut key_reader)
        .map_err(|e| orbis_core::Error::config(format!("Failed to parse private key: {}", e)))?
        .ok_or_else(|| orbis_core::Error::config("No private key found in file"))?;

    // Build server config
    let server_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| orbis_core::Error::config(format!("Failed to create TLS config: {}", e)))?;

    Ok(server_config)
}
