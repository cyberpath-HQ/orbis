//! CLI argument definitions using clap.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Orbis - Extensible asset management platform
#[derive(Parser, Debug)]
#[command(name = "orbis")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Application mode: standalone or client-server
    #[arg(
        long,
        env = "ORBIS_MODE",
        default_value = "standalone",
        help = "Application mode (standalone, client-server)"
    )]
    pub mode: String,

    /// Run mode: server or client (only for client-server mode)
    #[arg(
        long,
        env = "ORBIS_RUN_MODE",
        default_value = "client",
        help = "Run mode in client-server mode (server, client)"
    )]
    pub run_mode: String,

    /// Path to configuration file
    #[arg(short, long, env = "ORBIS_CONFIG", help = "Path to configuration file")]
    pub config: Option<PathBuf>,

    /// Active profile name
    #[arg(
        short,
        long,
        env = "ORBIS_PROFILE",
        help = "Active profile name for client mode"
    )]
    pub profile: Option<String>,

    // Server configuration
    /// Server host address
    #[arg(
        long,
        env = "ORBIS_SERVER_HOST",
        default_value = "127.0.0.1",
        help = "Server host address"
    )]
    pub server_host: String,

    /// Server port
    #[arg(
        long,
        env = "ORBIS_SERVER_PORT",
        default_value = "8000",
        help = "Server port"
    )]
    pub server_port: u16,

    /// Server URL (for client mode)
    #[arg(long, env = "ORBIS_SERVER_URL", help = "Server URL for client mode")]
    pub server_url: Option<String>,

    /// Request timeout in seconds
    #[arg(
        long,
        env = "ORBIS_REQUEST_TIMEOUT",
        default_value = "30",
        help = "Request timeout in seconds"
    )]
    pub request_timeout: u64,

    // Database configuration
    /// Database URL
    #[arg(long, env = "ORBIS_DB_URL", help = "Database connection URL")]
    pub db_url: Option<String>,

    /// Database host
    #[arg(long, env = "ORBIS_DB_HOST", help = "Database host")]
    pub db_host: Option<String>,

    /// Database port
    #[arg(long, env = "ORBIS_DB_PORT", help = "Database port")]
    pub db_port: Option<u16>,

    /// Database user
    #[arg(long, env = "ORBIS_DB_USER", help = "Database user")]
    pub db_user: Option<String>,

    /// Database password
    #[arg(long, env = "ORBIS_DB_PASSWORD", help = "Database password")]
    pub db_password: Option<String>,

    /// Database name
    #[arg(long, env = "ORBIS_DB_NAME", help = "Database name")]
    pub db_name: Option<String>,

    /// Database schema
    #[arg(long, env = "ORBIS_DB_SCHEMA", help = "Database schema")]
    pub db_schema: Option<String>,

    /// Database backend (postgres, sqlite)
    #[arg(
        long,
        env = "ORBIS_DB_BACKEND",
        default_value = "sqlite",
        help = "Database backend (postgres, sqlite)"
    )]
    pub db_backend: String,

    /// SQLite database path (for sqlite backend)
    #[arg(long, env = "ORBIS_DB_PATH", help = "SQLite database file path")]
    pub db_path: Option<PathBuf>,

    /// Maximum database connections
    #[arg(
        long,
        env = "ORBIS_DB_MAX_CONNECTIONS",
        default_value = "10",
        help = "Maximum database connections"
    )]
    pub db_max_connections: u32,

    /// Minimum database connections
    #[arg(
        long,
        env = "ORBIS_DB_MIN_CONNECTIONS",
        default_value = "2",
        help = "Minimum database connections"
    )]
    pub db_min_connections: u32,

    /// Database connection timeout in milliseconds
    #[arg(
        long,
        env = "ORBIS_DB_CONNECT_TIMEOUT_MS",
        default_value = "5000",
        help = "Database connection timeout (ms)"
    )]
    pub db_connect_timeout_ms: u64,

    /// Database acquire timeout in milliseconds
    #[arg(
        long,
        env = "ORBIS_DB_ACQUIRE_TIMEOUT_MS",
        default_value = "5000",
        help = "Database pool acquire timeout (ms)"
    )]
    pub db_acquire_timeout_ms: u64,

    /// Database idle timeout in milliseconds
    #[arg(
        long,
        env = "ORBIS_DB_IDLE_TIMEOUT_MS",
        default_value = "10000",
        help = "Database connection idle timeout (ms)"
    )]
    pub db_idle_timeout_ms: u64,

    /// Database max lifetime in milliseconds
    #[arg(
        long,
        env = "ORBIS_DB_MAX_LIFETIME_MS",
        default_value = "60000",
        help = "Database connection max lifetime (ms)"
    )]
    pub db_max_lifetime_ms: u64,

    /// Run database migrations on startup
    #[arg(
        long,
        env = "ORBIS_DB_RUN_MIGRATIONS",
        default_value = "true",
        help = "Run database migrations on startup"
    )]
    pub db_run_migrations: bool,

    // TLS configuration
    /// Enable TLS
    #[arg(long, env = "ORBIS_TLS_ENABLED", help = "Enable TLS")]
    pub tls_enabled: bool,

    /// TLS certificate path
    #[arg(long, env = "ORBIS_TLS_CERT_PATH", help = "TLS certificate file path")]
    pub tls_cert_path: Option<PathBuf>,

    /// TLS key path
    #[arg(long, env = "ORBIS_TLS_KEY_PATH", help = "TLS private key file path")]
    pub tls_key_path: Option<PathBuf>,

    /// TLS CA certificate path
    #[arg(long, env = "ORBIS_TLS_CA_PATH", help = "TLS CA certificate file path")]
    pub tls_ca_path: Option<PathBuf>,

    /// Verify TLS certificates
    #[arg(
        long,
        env = "ORBIS_TLS_VERIFY",
        default_value = "true",
        help = "Verify TLS certificates"
    )]
    pub tls_verify: bool,

    // Logging configuration
    /// Log level
    #[arg(
        long,
        env = "ORBIS_LOG",
        default_value = "info",
        help = "Log level (trace, debug, info, warn, error)"
    )]
    pub log_level: String,

    /// Log format
    #[arg(
        long,
        env = "ORBIS_LOG_FORMAT",
        default_value = "pretty",
        help = "Log format (pretty, json, compact)"
    )]
    pub log_format: String,

    /// Log file path
    #[arg(long, env = "ORBIS_LOG_FILE", help = "Log file path")]
    pub log_file: Option<PathBuf>,

    // Authentication configuration
    /// Enable authentication
    #[arg(
        long,
        env = "ORBIS_AUTH_ENABLED",
        help = "Enable authentication (mandatory in client-server mode)"
    )]
    pub auth_enabled: bool,

    /// JWT secret for token signing
    #[arg(long, env = "ORBIS_JWT_SECRET", help = "JWT secret for token signing")]
    pub jwt_secret: Option<String>,

    /// JWT token expiry in seconds
    #[arg(
        long,
        env = "ORBIS_JWT_EXPIRY_SECONDS",
        help = "JWT token expiry in seconds"
    )]
    pub jwt_expiry_seconds: Option<u64>,

    // Directory configuration
    /// Profiles directory
    #[arg(
        long,
        env = "ORBIS_PROFILES_DIR",
        help = "Directory for connection profiles"
    )]
    pub profiles_dir: Option<PathBuf>,

    /// Plugins directory
    #[arg(long, env = "ORBIS_PLUGINS_DIR", help = "Directory for plugins")]
    pub plugins_dir: Option<PathBuf>,

    /// Data directory
    #[arg(long, env = "ORBIS_DATA_DIR", help = "Data directory")]
    pub data_dir: Option<PathBuf>,

    /// Subcommand
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Available subcommands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the server
    Serve {
        /// Daemonize the server
        #[arg(short, long)]
        daemon: bool,
    },

    /// Manage profiles
    Profile {
        #[command(subcommand)]
        action: ProfileCommands,
    },

    /// Database operations
    Db {
        #[command(subcommand)]
        action: DbCommands,
    },

    /// Plugin management
    Plugin {
        #[command(subcommand)]
        action: PluginCommands,
    },

    /// Generate configuration file
    Config {
        /// Output path
        #[arg(short, long)]
        output: PathBuf,
    },
}

/// Profile management commands.
#[derive(Subcommand, Debug)]
pub enum ProfileCommands {
    /// List all profiles
    List,

    /// Add a new profile
    Add {
        /// Profile name
        name: String,

        /// Server URL
        #[arg(long)]
        url: String,

        /// Set as default
        #[arg(short, long)]
        default: bool,
    },

    /// Remove a profile
    Remove {
        /// Profile name
        name: String,
    },

    /// Set default profile
    SetDefault {
        /// Profile name
        name: String,
    },

    /// Show profile details
    Show {
        /// Profile name
        name: String,
    },
}

/// Database management commands.
#[derive(Subcommand, Debug)]
pub enum DbCommands {
    /// Run pending migrations
    Migrate,

    /// Revert last migration
    Revert,

    /// Show migration status
    Status,

    /// Create a new migration
    Create {
        /// Migration name
        name: String,
    },
}

/// Plugin management commands.
#[derive(Subcommand, Debug)]
pub enum PluginCommands {
    /// List installed plugins
    List,

    /// Install a plugin
    Install {
        /// Plugin path or URL
        source: String,
    },

    /// Uninstall a plugin
    Uninstall {
        /// Plugin name
        name: String,
    },

    /// Enable a plugin
    Enable {
        /// Plugin name
        name: String,
    },

    /// Disable a plugin
    Disable {
        /// Plugin name
        name: String,
    },

    /// Show plugin details
    Info {
        /// Plugin name
        name: String,
    },
}
