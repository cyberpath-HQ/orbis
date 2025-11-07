/// Network isolation and restrictions for plugins
/// 
/// Each plugin gets its own network namespace with isolated firewall rules.
/// The host system can access everything, each plugin is restricted independently.
use crate::PluginError;
use std::net::IpAddr;
use tracing::{info, warn, debug};

// Re-export NetworkTarget from core network_types module
pub use crate::network_types::NetworkTarget;


/// Network isolation configuration per plugin
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Enable network namespace isolation (MUST be true for isolation to work)
    pub enable_namespace: bool,
    
    /// Allowed network targets (domains, IPs, IP:port pairs)
    pub allowed_targets: Vec<NetworkTarget>,
    
    /// Allow loopback (127.0.0.1) - for inter-plugin communication via host
    pub allow_loopback: bool,
    
    /// Allow all outbound connections (bypasses whitelist)
    pub allow_all_outbound: bool,
    
    /// Allow DNS resolution (port 53) - needed for domain-based rules
    pub allow_dns: bool,
    
    /// Maximum connections (tracked via conntrack)
    pub max_connections: u32,
    
    /// Enable connection tracking
    pub enable_conntrack: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            enable_namespace: true,
            allowed_targets: Vec::new(),
            allow_loopback: true,
            allow_all_outbound: false,
            allow_dns: true, // Usually needed for domain resolution
            max_connections: 50,
            enable_conntrack: true,
        }
    }
}

impl NetworkConfig {
    /// Create a permissive network config (for plugins that need broad access)
    pub fn permissive() -> Self {
        Self {
            enable_namespace: true, // Still isolated, but allows everything
            allowed_targets: Vec::new(),
            allow_loopback: true,
            allow_all_outbound: true,
            allow_dns: true,
            max_connections: 100,
            enable_conntrack: true,
        }
    }
    
    /// Create a restrictive network config (completely isolated)
    pub fn restrictive() -> Self {
        Self {
            enable_namespace: true,
            allowed_targets: Vec::new(),
            allow_loopback: true,
            allow_all_outbound: false,
            allow_dns: false, // No external DNS
            max_connections: 10,
            enable_conntrack: true,
        }
    }
    
    /// Allow access to a domain (e.g., "www.google.com" or "*.example.com")
    pub fn allow_domain(&mut self, domain: impl Into<String>) {
        let target = NetworkTarget::domain(domain);
        if !self.allowed_targets.contains(&target) {
            self.allowed_targets.push(target);
        }
    }
    
    /// Allow access to an IP address (any port)
    pub fn allow_ip(&mut self, ip: IpAddr) {
        let target = NetworkTarget::ip(ip);
        if !self.allowed_targets.contains(&target) {
            self.allowed_targets.push(target);
        }
    }
    
    /// Allow access to a specific IP:port pair
    pub fn allow_ip_port(&mut self, ip: IpAddr, port: u16) {
        let target = NetworkTarget::ip_port(ip, port);
        if !self.allowed_targets.contains(&target) {
            self.allowed_targets.push(target);
        }
    }
    
    /// Allow access to an IP with port range
    pub fn allow_ip_port_range(&mut self, ip: IpAddr, start_port: u16, end_port: u16) {
        let target = NetworkTarget::ip_port_range(ip, start_port, end_port);
        if !self.allowed_targets.contains(&target) {
            self.allowed_targets.push(target);
        }
    }
    
    /// Add a network target
    pub fn allow_target(&mut self, target: NetworkTarget) {
        if !self.allowed_targets.contains(&target) {
            self.allowed_targets.push(target);
        }
    }
}

/// Apply network isolation to current process
/// 
/// This MUST be called AFTER the network namespace is created by the namespace module.
/// Each plugin gets its own network namespace with:
/// - Its own loopback interface
/// - Its own veth pair for internet access (optional)
/// - Its own iptables rules (completely independent)
#[cfg(target_os = "linux")]
pub fn apply_network_isolation(config: &NetworkConfig, plugin_name: &str) -> Result<(), PluginError> {
    if !config.enable_namespace {
        warn!("Network namespace isolation DISABLED for plugin: {} - this plugin will share the host's network!", plugin_name);
        return Ok(());
    }
    
    info!("Applying per-plugin network isolation for: {}", plugin_name);
    
    // 1. Set up loopback interface (needed for local communication)
    if config.allow_loopback {
        setup_loopback_interface()?;
    }
    
    // 2. Set up veth pair for internet access (if not completely isolated)
    if config.allow_all_outbound || !config.allowed_targets.is_empty() || config.allow_dns {
        setup_veth_pair(plugin_name)?;
    }
    
    // 3. Configure plugin-specific firewall rules
    configure_plugin_firewall(config, plugin_name)?;
    
    info!("Network isolation applied for plugin '{}': {} allowed targets", 
          plugin_name, config.allowed_targets.len());
    Ok(())
}

/// Setup loopback interface in network namespace
#[cfg(target_os = "linux")]
fn setup_loopback_interface() -> Result<(), PluginError> {
    use std::process::Command;
    
    // Bring up loopback interface
    let output = Command::new("ip")
        .args(&["link", "set", "lo", "up"])
        .output();
    
    match output {
        Ok(out) if out.status.success() => {
            info!("Loopback interface configured");
            Ok(())
        }
        Ok(out) => {
            warn!("Failed to set up loopback: {}", String::from_utf8_lossy(&out.stderr));
            // Don't fail - plugin might still work
            Ok(())
        }
        Err(e) => {
            warn!("Failed to execute ip command: {} - loopback not configured", e);
            Ok(())
        }
    }
}

/// Setup veth pair to connect plugin namespace to host
/// 
/// Creates a virtual ethernet pair: veth-<plugin> (in host) <-> eth0 (in plugin namespace)
#[cfg(target_os = "linux")]
fn setup_veth_pair(plugin_name: &str) -> Result<(), PluginError> {
    use std::process::Command;
    
    let veth_host = format!("veth-{}", plugin_name);
    let veth_plugin = "eth0"; // Inside the namespace
    
    debug!("Setting up veth pair: {} <-> {}", veth_host, veth_plugin);
    
    // Create veth pair (executed from within the namespace)
    // Note: This is a simplified version. In production, you'd create the veth
    // from the host namespace and move one end into the plugin namespace.
    
    // For now, we'll use a simpler approach: configure the existing network
    // The actual veth pair creation should be done from outside the namespace
    
    warn!("veth pair setup is simplified - production should use netns from host");
    
    // Configure routing inside the namespace
    let _ = Command::new("ip")
        .args(&["route", "add", "default", "via", "169.254.1.1"])
        .output();
    
    Ok(())
}

/// Configure plugin-specific firewall rules
/// 
/// Each plugin gets its OWN iptables rules in its OWN network namespace.
/// These rules are completely independent from other plugins and the host.
#[cfg(target_os = "linux")]
fn configure_plugin_firewall(config: &NetworkConfig, plugin_name: &str) -> Result<(), PluginError> {
    use std::process::Command;
    
    info!("Configuring firewall for plugin: {}", plugin_name);
    
    // If allow all, we're done (default ACCEPT policy)
    if config.allow_all_outbound {
        info!("Plugin '{}' has unrestricted network access", plugin_name);
        return Ok(());
    }
    
    // Set default policy to DROP
    let _ = Command::new("iptables")
        .args(&["-P", "OUTPUT", "DROP"])
        .output();
    
    let _ = Command::new("iptables")
        .args(&["-P", "INPUT", "DROP"])
        .output();
    
    // Allow loopback
    if config.allow_loopback {
        Command::new("iptables")
            .args(&["-A", "OUTPUT", "-o", "lo", "-j", "ACCEPT"])
            .output()
            .map_err(|e| PluginError::LoadError(format!("Failed to configure loopback: {}", e)))?;
        
        Command::new("iptables")
            .args(&["-A", "INPUT", "-i", "lo", "-j", "ACCEPT"])
            .output()
            .map_err(|e| PluginError::LoadError(format!("Failed to configure loopback: {}", e)))?;
    }
    
    // Allow DNS if enabled
    if config.allow_dns {
        // Allow DNS queries (UDP port 53)
        Command::new("iptables")
            .args(&["-A", "OUTPUT", "-p", "udp", "--dport", "53", "-j", "ACCEPT"])
            .output()
            .map_err(|e| PluginError::LoadError(format!("Failed to allow DNS: {}", e)))?;
        
        // Allow DNS responses
        Command::new("iptables")
            .args(&["-A", "INPUT", "-p", "udp", "--sport", "53", "-j", "ACCEPT"])
            .output()
            .map_err(|e| PluginError::LoadError(format!("Failed to allow DNS responses: {}", e)))?;
    }
    
    // Allow established connections (responses to our requests)
    if config.enable_conntrack {
        Command::new("iptables")
            .args(&["-A", "INPUT", "-m", "state", "--state", "ESTABLISHED,RELATED", "-j", "ACCEPT"])
            .output()
            .map_err(|e| PluginError::LoadError(format!("Failed to configure conntrack: {}", e)))?;
    }
    
    // Process allowed targets
    for target in &config.allowed_targets {
        match target {
            NetworkTarget::Domain(domain) => {
                // For domains, we need to resolve them to IPs
                // This is complex - we'll allow and log
                info!("Domain-based filtering for '{}' requires DNS resolution", domain);
                if domain.contains('*') {
                    warn!("Wildcard domain '{}' requires DNS monitoring - allowing DNS", domain);
                }
                // In production, you'd use a DNS proxy or resolve at rule-application time
                // For now, we document this limitation
            }
            
            NetworkTarget::Ip(ip) => {
                // Allow any connection to this IP
                Command::new("iptables")
                    .args(&["-A", "OUTPUT", "-d", &ip.to_string(), "-j", "ACCEPT"])
                    .output()
                    .map_err(|e| PluginError::LoadError(format!("Failed to allow IP {}: {}", ip, e)))?;
                
                debug!("Allowed all ports to IP: {}", ip);
            }
            
            NetworkTarget::IpPort(ip, port) => {
                // Allow TCP to this IP:port
                Command::new("iptables")
                    .args(&["-A", "OUTPUT", "-p", "tcp", "-d", &ip.to_string(), 
                           "--dport", &port.to_string(), "-j", "ACCEPT"])
                    .output()
                    .map_err(|e| PluginError::LoadError(format!("Failed to allow {}:{}: {}", ip, port, e)))?;
                
                // Allow UDP to this IP:port
                Command::new("iptables")
                    .args(&["-A", "OUTPUT", "-p", "udp", "-d", &ip.to_string(), 
                           "--dport", &port.to_string(), "-j", "ACCEPT"])
                    .output()
                    .map_err(|e| PluginError::LoadError(format!("Failed to allow {}:{} UDP: {}", ip, port, e)))?;
                
                debug!("Allowed IP:port {}:{}", ip, port);
            }
            
            NetworkTarget::IpPortRange(ip, start, end) => {
                // Allow TCP port range
                let port_range = format!("{}:{}", start, end);
                Command::new("iptables")
                    .args(&["-A", "OUTPUT", "-p", "tcp", "-d", &ip.to_string(), 
                           "--dport", &port_range, "-j", "ACCEPT"])
                    .output()
                    .map_err(|e| PluginError::LoadError(format!("Failed to allow {}:{}-{}: {}", ip, start, end, e)))?;
                
                // Allow UDP port range
                Command::new("iptables")
                    .args(&["-A", "OUTPUT", "-p", "udp", "-d", &ip.to_string(), 
                           "--dport", &port_range, "-j", "ACCEPT"])
                    .output()
                    .map_err(|e| PluginError::LoadError(format!("Failed to allow {}:{}-{} UDP: {}", ip, start, end, e)))?;
                
                debug!("Allowed IP:port range {}:{}-{}", ip, start, end);
            }
        }
    }
    
    // Log dropped packets (optional, for debugging)
    let _ = Command::new("iptables")
        .args(&["-A", "OUTPUT", "-j", "LOG", "--log-prefix", 
               &format!("[PLUGIN-{}] DROP: ", plugin_name), "--log-level", "4"])
        .output();
    
    info!("Firewall configured for plugin '{}': {} targets allowed", 
          plugin_name, config.allowed_targets.len());
    
    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn apply_network_isolation(_config: &NetworkConfig, _plugin_name: &str) -> Result<(), PluginError> {
    warn!("Network isolation not available on this platform");
    Ok(())
}

/// Resolve a domain to IP addresses (for domain-based rules)
/// 
/// Note: This requires DNS to be functional in the namespace
#[cfg(target_os = "linux")]
pub fn resolve_domain(domain: &str) -> Result<Vec<IpAddr>, PluginError> {
    use std::net::ToSocketAddrs;
    
    // Add a dummy port for resolution
    let address = format!("{}:0", domain);
    
    match address.to_socket_addrs() {
        Ok(addrs) => {
            let ips: Vec<IpAddr> = addrs.map(|addr| addr.ip()).collect();
            if ips.is_empty() {
                Err(PluginError::LoadError(format!("No IPs found for domain: {}", domain)))
            } else {
                Ok(ips)
            }
        }
        Err(e) => {
            Err(PluginError::LoadError(format!("Failed to resolve domain '{}': {}", domain, e)))
        }
    }
}

/// Check if a domain matches a wildcard pattern
pub fn domain_matches_pattern(domain: &str, pattern: &str) -> bool {
    if !pattern.contains('*') {
        // Exact match
        return domain == pattern;
    }
    
    // Wildcard pattern
    if pattern.starts_with("*.") {
        let suffix = &pattern[2..];
        domain.ends_with(suffix) || domain == suffix
    } else if pattern.ends_with(".*") {
        let prefix = &pattern[..pattern.len() - 2];
        domain.starts_with(prefix)
    } else {
        // More complex wildcard - simple contains check
        let parts: Vec<&str> = pattern.split('*').collect();
        let mut pos = 0;
        for part in parts {
            if let Some(found) = domain[pos..].find(part) {
                pos += found + part.len();
            } else {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    
    #[test]
    fn test_network_config_default() {
        let config = NetworkConfig::default();
        assert!(config.enable_namespace);
        assert!(config.allow_loopback);
        assert!(!config.allow_all_outbound);
        assert!(config.allow_dns);
    }
    
    #[test]
    fn test_network_config_permissive() {
        let config = NetworkConfig::permissive();
        assert!(config.enable_namespace); // Still isolated!
        assert!(config.allow_all_outbound);
    }
    
    #[test]
    fn test_allow_ip() {
        let mut config = NetworkConfig::default();
        let ip = IpAddr::from_str("8.8.8.8").unwrap();
        config.allow_ip(ip);
        assert_eq!(config.allowed_targets.len(), 1);
    }
    
    #[test]
    fn test_allow_ip_port() {
        let mut config = NetworkConfig::default();
        let ip = IpAddr::from_str("192.168.1.1").unwrap();
        config.allow_ip_port(ip, 5432);
        assert_eq!(config.allowed_targets.len(), 1);
    }
    
    #[test]
    fn test_allow_domain() {
        let mut config = NetworkConfig::default();
        config.allow_domain("www.google.com");
        config.allow_domain("*.example.com");
        assert_eq!(config.allowed_targets.len(), 2);
    }
    
    #[test]
    fn test_domain_pattern_matching() {
        assert!(domain_matches_pattern("www.google.com", "www.google.com"));
        assert!(domain_matches_pattern("api.example.com", "*.example.com"));
        assert!(domain_matches_pattern("example.com", "*.example.com"));
        assert!(!domain_matches_pattern("google.com", "*.example.com"));
        assert!(domain_matches_pattern("test.api.example.com", "*.example.com"));
    }
    
    #[test]
    fn test_network_target_types() {
        let ip = IpAddr::from_str("1.2.3.4").unwrap();
        
        let target1 = NetworkTarget::domain("test.com");
        let target2 = NetworkTarget::ip(ip);
        let target3 = NetworkTarget::ip_port(ip, 443);
        let target4 = NetworkTarget::ip_port_range(ip, 8000, 9000);
        
        assert!(matches!(target1, NetworkTarget::Domain(_)));
        assert!(matches!(target2, NetworkTarget::Ip(_)));
        assert!(matches!(target3, NetworkTarget::IpPort(_, 443)));
        assert!(matches!(target4, NetworkTarget::IpPortRange(_, 8000, 9000)));
    }
}

