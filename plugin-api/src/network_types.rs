/// Network target types - used in requirements and sandbox
/// 
/// This module is always compiled since it's needed for plugin requirements
/// declaration, but the actual network enforcement is in the sandbox module.

use std::net::IpAddr;
use serde::{Deserialize, Serialize};

/// Network target - can be a domain pattern, IP, or IP:port pair
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NetworkTarget {
    /// Domain pattern (supports wildcards like *.example.com)
    Domain(String),
    /// IP address (any port)
    Ip(IpAddr),
    /// IP and port pair
    IpPort(IpAddr, u16),
    /// IP and port range
    IpPortRange(IpAddr, u16, u16),
}

impl NetworkTarget {
    /// Create from domain string
    pub fn domain(domain: impl Into<String>) -> Self {
        Self::Domain(domain.into())
    }
    
    /// Create from IP
    pub fn ip(ip: IpAddr) -> Self {
        Self::Ip(ip)
    }
    
    /// Create from IP:port pair
    pub fn ip_port(ip: IpAddr, port: u16) -> Self {
        Self::IpPort(ip, port)
    }
    
    /// Create from IP and port range
    pub fn ip_port_range(ip: IpAddr, start: u16, end: u16) -> Self {
        Self::IpPortRange(ip, start, end)
    }
}

