/// Filesystem isolation for plugins
use std::path::Path;
use tracing::{info, warn};
use crate::PluginError;

/// Filesystem isolation configuration
#[derive(Debug, Clone)]
pub struct FilesystemConfig {
    /// Root directory for chroot
    pub root_dir: Option<String>,
    /// Make root read-only
    pub readonly_root: bool,
}

impl Default for FilesystemConfig {
    fn default() -> Self {
        Self {
            root_dir: None,
            readonly_root: true,
        }
    }
}

/// Apply filesystem isolation (chroot)
#[cfg(target_os = "linux")]
pub fn apply_filesystem_isolation(config: &FilesystemConfig) -> Result<(), PluginError> {
    use nix::unistd::chroot;
    use std::env;

    if let Some(root_dir) = &config.root_dir {
        info!("Applying filesystem isolation with root: {}", root_dir);

        let root_path = Path::new(root_dir);
        if !root_path.exists() {
            return Err(PluginError::LoadError(format!(
                "Chroot directory does not exist: {}",
                root_dir
            )));
        }

        // Change to the new root directory
        env::set_current_dir(root_path)
            .map_err(|e| PluginError::LoadError(format!("Failed to chdir to {}: {}", root_dir, e)))?;

        // Chroot to the new root
        chroot(root_path)
            .map_err(|e| PluginError::LoadError(format!("Failed to chroot to {}: {}", root_dir, e)))?;

        // Change to / in the new root
        env::set_current_dir("/")
            .map_err(|e| PluginError::LoadError(format!("Failed to chdir to / after chroot: {}", e)))?;

        info!("Filesystem chrooted to: {}", root_dir);

        if config.readonly_root {
            // Remount root as read-only
            use std::process::Command;
            match Command::new("mount")
                .args(&["-o", "remount,ro", "/"])
                .output() {
                Ok(output) if output.status.success() => {
                    info!("Root filesystem remounted as read-only");
                }
                Ok(output) => {
                    warn!("Failed to remount root as read-only: {}",
                          String::from_utf8_lossy(&output.stderr));
                }
                Err(e) => {
                    warn!("Failed to execute mount command: {}", e);
                }
            }
        }
    } else {
        info!("Filesystem isolation disabled (no root_dir specified)");
    }

    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn apply_filesystem_isolation(_config: &FilesystemConfig) -> Result<(), PluginError> {
    warn!("Filesystem isolation not available on this platform");
    Ok(())
}

/// Create a minimal root filesystem for plugins
pub fn create_minimal_rootfs(target: &Path) -> Result<(), PluginError> {
    info!("Creating minimal rootfs at: {}", target.display());

    // TODO: Create essential directories:
    // /bin, /lib, /lib64, /usr, /tmp, /proc, /sys, /dev

    warn!("create_minimal_rootfs not yet implemented");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filesystem_config_default() {
        let config = FilesystemConfig::default();
        assert!(config.root_dir.is_none());
        assert!(config.readonly_root);
    }
}

