//! Logging configuration.

use crate::Cli;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::Level;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Log output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    /// Pretty-printed format (default).
    #[default]
    Pretty,

    /// JSON format.
    Json,

    /// Compact format.
    Compact,
}

impl std::str::FromStr for LogFormat {
    type Err = orbis_core::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pretty" => Ok(Self::Pretty),
            "json" => Ok(Self::Json),
            "compact" => Ok(Self::Compact),
            _ => Err(orbis_core::Error::config(format!(
                "Invalid log format: '{}'. Expected 'pretty', 'json', or 'compact'",
                s
            ))),
        }
    }
}

/// Logging configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    /// Log level.
    pub level: String,

    /// Log format.
    pub format: LogFormat,

    /// Log file path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<PathBuf>,

    /// Include file and line numbers in logs.
    pub include_file_line: bool,

    /// Include target (module path) in logs.
    pub include_target: bool,

    /// Include thread IDs in logs.
    pub include_thread_id: bool,

    /// Include span events in logs.
    pub include_span_events: bool,
}

impl LogConfig {
    /// Create log config from CLI arguments.
    pub fn from_cli(cli: &Cli, file_config: Option<&LogConfig>) -> Self {
        Self {
            level: cli.log_level.clone(),
            format: cli.log_format.parse().unwrap_or_else(|_| {
                file_config.map(|c| c.format).unwrap_or_default()
            }),
            file: cli.log_file.clone().or_else(|| {
                file_config.and_then(|c| c.file.clone())
            }),
            include_file_line: file_config.is_some_and(|c| c.include_file_line),
            include_target: file_config.map(|c| c.include_target).unwrap_or(true),
            include_thread_id: file_config.is_some_and(|c| c.include_thread_id),
            include_span_events: file_config.is_some_and(|c| c.include_span_events),
        }
    }

    /// Initialize the tracing subscriber.
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails.
    pub fn init(&self) -> orbis_core::Result<()> {
        let filter = EnvFilter::try_new(&self.level).map_err(|e| {
            orbis_core::Error::config(format!("Invalid log level '{}': {}", self.level, e))
        })?;

        let subscriber = tracing_subscriber::registry().with(filter);

        match self.format {
            LogFormat::Pretty => {
                let layer = fmt::layer()
                    .with_file(self.include_file_line)
                    .with_line_number(self.include_file_line)
                    .with_target(self.include_target)
                    .with_thread_ids(self.include_thread_id)
                    .pretty();

                subscriber.with(layer).try_init().map_err(|e| {
                    orbis_core::Error::config(format!("Failed to initialize logging: {}", e))
                })?;
            }
            LogFormat::Json => {
                let layer = fmt::layer()
                    .with_file(self.include_file_line)
                    .with_line_number(self.include_file_line)
                    .with_target(self.include_target)
                    .with_thread_ids(self.include_thread_id)
                    .json();

                subscriber.with(layer).try_init().map_err(|e| {
                    orbis_core::Error::config(format!("Failed to initialize logging: {}", e))
                })?;
            }
            LogFormat::Compact => {
                let layer = fmt::layer()
                    .with_file(self.include_file_line)
                    .with_line_number(self.include_file_line)
                    .with_target(self.include_target)
                    .with_thread_ids(self.include_thread_id)
                    .compact();

                subscriber.with(layer).try_init().map_err(|e| {
                    orbis_core::Error::config(format!("Failed to initialize logging: {}", e))
                })?;
            }
        }

        Ok(())
    }

    /// Get the log level as tracing Level.
    #[must_use]
    pub fn tracing_level(&self) -> Level {
        match self.level.to_lowercase().as_str() {
            "trace" => Level::TRACE,
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" | "warning" => Level::WARN,
            "error" => Level::ERROR,
            _ => Level::INFO,
        }
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Pretty,
            file: None,
            include_file_line: false,
            include_target: true,
            include_thread_id: false,
            include_span_events: false,
        }
    }
}
