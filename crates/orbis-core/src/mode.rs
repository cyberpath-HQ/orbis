//! Application modes and run configurations.

use serde::{Deserialize, Serialize};

/// The application mode determines how Orbis runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AppMode {
    /// Standalone mode: runs locally without network requirements.
    /// Authentication is optional.
    #[default]
    Standalone,

    /// Client-server mode: requires network connectivity.
    /// Authentication is mandatory.
    ClientServer,
}

impl AppMode {
    /// Returns true if authentication is mandatory for this mode.
    #[must_use]
    pub const fn requires_auth(&self) -> bool {
        matches!(self, Self::ClientServer)
    }

    /// Returns true if this is standalone mode.
    #[must_use]
    pub const fn is_standalone(&self) -> bool {
        matches!(self, Self::Standalone)
    }

    /// Returns true if this is client-server mode.
    #[must_use]
    pub const fn is_client_server(&self) -> bool {
        matches!(self, Self::ClientServer)
    }
}

impl std::str::FromStr for AppMode {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "standalone" => Ok(Self::Standalone),
            "client-server" | "clientserver" | "client_server" => Ok(Self::ClientServer),
            _ => Err(crate::Error::config(format!(
                "Invalid app mode: '{}'. Expected 'standalone' or 'client-server'",
                s
            ))),
        }
    }
}

impl std::fmt::Display for AppMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Standalone => write!(f, "standalone"),
            Self::ClientServer => write!(f, "client-server"),
        }
    }
}

/// The run mode determines which component to run in client-server mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RunMode {
    /// Run as a server (API gateway).
    Server,

    /// Run as a client (connects to server).
    #[default]
    Client,
}

impl RunMode {
    /// Returns true if running as server.
    #[must_use]
    pub const fn is_server(&self) -> bool {
        matches!(self, Self::Server)
    }

    /// Returns true if running as client.
    #[must_use]
    pub const fn is_client(&self) -> bool {
        matches!(self, Self::Client)
    }
}

impl std::str::FromStr for RunMode {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "server" => Ok(Self::Server),
            "client" => Ok(Self::Client),
            _ => Err(crate::Error::config(format!(
                "Invalid run mode: '{}'. Expected 'server' or 'client'",
                s
            ))),
        }
    }
}

impl std::fmt::Display for RunMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Server => write!(f, "server"),
            Self::Client => write!(f, "client"),
        }
    }
}
