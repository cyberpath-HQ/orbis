//! Plugin hot reload watcher.
//!
//! Watches the plugins directory for changes and notifies subscribers
//! when plugins are added, modified, or removed.

use notify::{
    Config as NotifyConfig,
    Event,
    EventKind,
    RecommendedWatcher,
    RecursiveMode,
    Watcher,
};
use std::{
    collections::HashSet,
    path::{
        Path,
        PathBuf,
    },
    sync::{
        Arc,
        atomic::{
            AtomicBool,
            Ordering,
        },
    },
    time::Duration,
};
use tokio::sync::mpsc;
use tracing::{
    debug,
    error,
    info,
    warn,
};

/// Plugin change event types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginChangeKind {
    /// Plugin was added or installed.
    Added,
    /// Plugin files were modified.
    Modified,
    /// Plugin was removed or uninstalled.
    Removed,
}

/// A plugin change event.
#[derive(Debug, Clone)]
pub struct PluginChangeEvent {
    /// The kind of change.
    pub kind: PluginChangeKind,
    /// The path that changed.
    pub path: PathBuf,
    /// The plugin ID (if determinable from path).
    pub plugin_id: Option<String>,
}

impl PluginChangeEvent {
    /// Create a new plugin change event.
    #[must_use]
    pub fn new(kind: PluginChangeKind, path: PathBuf) -> Self {
        let plugin_id = Self::extract_plugin_id(&path);
        Self {
            kind,
            path,
            plugin_id,
        }
    }

    /// Extract plugin ID from path.
    ///
    /// Assumes plugins are organized as:
    /// - `plugins/<plugin-id>/...` (unpacked)
    /// - `plugins/<plugin-id>.wasm` (standalone)
    /// - `plugins/<plugin-id>.zip` (packed)
    fn extract_plugin_id(path: &Path) -> Option<String> {
        // Get the first component after "plugins" directory
        let components: Vec<_> = path.components().collect();

        for (i, comp) in components.iter().enumerate() {
            if let std::path::Component::Normal(name) = comp {
                if name.to_str() == Some("plugins") && i.wrapping_add(1) < components.len() {
                    if let std::path::Component::Normal(next) = &components[i.wrapping_add(1)] {
                        let name = next.to_string_lossy().to_string();
                        // Remove extension if present
                        return Some(
                            name.trim_end_matches(".wasm")
                                .trim_end_matches(".zip")
                                .to_string(),
                        );
                    }
                }
            }
        }

        // Fallback: use file stem
        path.file_stem()
            .and_then(|s| s.to_str())
            .map(String::from)
    }
}

/// Plugin watcher configuration.
#[derive(Debug, Clone)]
pub struct WatcherConfig {
    /// Directory to watch.
    pub watch_dir: PathBuf,
    /// Debounce duration for file events.
    pub debounce_duration: Duration,
    /// Whether to watch recursively.
    pub recursive: bool,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            watch_dir: PathBuf::from("./plugins"),
            debounce_duration: Duration::from_millis(500),
            recursive: true,
        }
    }
}

/// Plugin file system watcher.
///
/// Watches the plugins directory and emits events when plugins change.
pub struct PluginWatcher {
    /// Configuration.
    config: WatcherConfig,
    /// Whether the watcher is running.
    running: Arc<AtomicBool>,
    /// Shutdown signal sender.
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl PluginWatcher {
    /// Create a new plugin watcher.
    #[must_use]
    pub fn new(config: WatcherConfig) -> Self {
        Self {
            config,
            running: Arc::new(AtomicBool::new(false)),
            shutdown_tx: None,
        }
    }

    /// Create with default configuration.
    #[must_use]
    pub fn with_default_config() -> Self {
        Self::new(WatcherConfig::default())
    }

    /// Check if the watcher is currently running.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Start watching for plugin changes.
    ///
    /// Returns a receiver for plugin change events.
    ///
    /// # Errors
    ///
    /// Returns an error if the watcher cannot be started.
    pub fn start(
        &mut self,
    ) -> orbis_core::Result<mpsc::UnboundedReceiver<PluginChangeEvent>> {
        if self.is_running() {
            return Err(orbis_core::Error::plugin("Plugin watcher is already running"));
        }

        // Ensure watch directory exists
        if !self.config.watch_dir.exists() {
            std::fs::create_dir_all(&self.config.watch_dir).map_err(|e| {
                orbis_core::Error::plugin(format!(
                    "Failed to create plugins directory: {}",
                    e
                ))
            })?;
        }

        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

        let watch_dir = self.config.watch_dir.clone();
        let debounce_duration = self.config.debounce_duration;
        let recursive = self.config.recursive;
        let running = self.running.clone();

        // Spawn watcher in a blocking task since notify uses std channels
        std::thread::spawn(move || {
            let result = Self::run_watcher(
                watch_dir,
                debounce_duration,
                recursive,
                event_tx,
                shutdown_rx,
                running,
            );

            if let Err(e) = result {
                error!("Plugin watcher error: {}", e);
            }
        });

        self.running.store(true, Ordering::SeqCst);
        self.shutdown_tx = Some(shutdown_tx);

        info!("Plugin watcher started for: {:?}", self.config.watch_dir);

        Ok(event_rx)
    }

    /// Stop the watcher.
    pub fn stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        self.running.store(false, Ordering::SeqCst);
        info!("Plugin watcher stopped");
    }

    /// Run the file watcher (blocking).
    #[allow(clippy::cognitive_complexity)]
    fn run_watcher(
        watch_dir: PathBuf,
        debounce_duration: Duration,
        recursive: bool,
        event_tx: mpsc::UnboundedSender<PluginChangeEvent>,
        mut shutdown_rx: tokio::sync::oneshot::Receiver<()>,
        running: Arc<AtomicBool>,
    ) -> orbis_core::Result<()> {
        // Create a channel for notify events
        let (tx, rx) = std::sync::mpsc::channel();

        // Create watcher
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            },
            NotifyConfig::default().with_poll_interval(debounce_duration),
        )
        .map_err(|e| orbis_core::Error::plugin(format!("Failed to create file watcher: {}", e)))?;

        // Start watching
        let mode = if recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };

        watcher.watch(&watch_dir, mode).map_err(|e| {
            orbis_core::Error::plugin(format!("Failed to watch directory: {}", e))
        })?;

        debug!("Watching directory: {:?}", watch_dir);

        // Track seen events for debouncing
        let mut pending_events: HashSet<PathBuf> = HashSet::new();
        let mut last_event_time = std::time::Instant::now();

        // Process events
        loop {
            // Check for shutdown signal (non-blocking)
            if shutdown_rx.try_recv().is_ok() {
                debug!("Received shutdown signal");
                break;
            }

            // Wait for file events with timeout
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(event) => {
                    // Process the notify event
                    for path in &event.paths {
                        if Self::is_plugin_file(path) {
                            pending_events.insert(path.clone());
                            last_event_time = std::time::Instant::now();
                        }
                    }

                    // Debounce: only emit after quiet period
                    if last_event_time.elapsed() >= debounce_duration && !pending_events.is_empty()
                    {
                        for path in pending_events.drain() {
                            if let Some(change_event) =
                                Self::convert_event(&event.kind, &path)
                            {
                                debug!("Plugin change: {:?}", change_event);
                                if event_tx.send(change_event).is_err() {
                                    // Receiver dropped, stop watching
                                    warn!("Event receiver dropped, stopping watcher");
                                    running.store(false, Ordering::SeqCst);
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // Check for debounced events
                    if last_event_time.elapsed() >= debounce_duration && !pending_events.is_empty()
                    {
                        for path in pending_events.drain() {
                            // Determine kind based on file existence since we lost the original event kind
                            let kind = if path.exists() {
                                PluginChangeKind::Modified
                            } else {
                                PluginChangeKind::Removed
                            };

                            let change_event = PluginChangeEvent::new(kind, path);
                            debug!("Plugin change (debounced): {:?}", change_event);
                            if event_tx.send(change_event).is_err() {
                                warn!("Event receiver dropped, stopping watcher");
                                running.store(false, Ordering::SeqCst);
                                return Ok(());
                            }
                        }
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    warn!("Watcher channel disconnected");
                    break;
                }
            }
        }

        running.store(false, Ordering::SeqCst);
        Ok(())
    }

    /// Convert a notify event to a plugin change event.
    fn convert_event(kind: &EventKind, path: &Path) -> Option<PluginChangeEvent> {
        let change_kind = match kind {
            EventKind::Create(_) => PluginChangeKind::Added,
            EventKind::Modify(_) => PluginChangeKind::Modified,
            EventKind::Remove(_) => PluginChangeKind::Removed,
            _ => return None,
        };

        Some(PluginChangeEvent::new(change_kind, path.to_path_buf()))
    }

    /// Check if a path is a plugin file.
    fn is_plugin_file(path: &Path) -> bool {
        // Check for plugin-related files
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            matches!(ext, "wasm" | "zip" | "json")
        } else {
            // Could be a directory (unpacked plugin)
            path.is_dir()
        }
    }
}

impl Drop for PluginWatcher {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_plugin_id_unpacked() {
        let path = PathBuf::from("plugins/hello-plugin/manifest.json");
        let id = PluginChangeEvent::extract_plugin_id(&path);
        assert_eq!(id, Some("hello-plugin".to_string()));
    }

    #[test]
    fn test_extract_plugin_id_standalone() {
        let path = PathBuf::from("plugins/my-plugin.wasm");
        let id = PluginChangeEvent::extract_plugin_id(&path);
        assert_eq!(id, Some("my-plugin".to_string()));
    }

    #[test]
    fn test_extract_plugin_id_packed() {
        let path = PathBuf::from("plugins/another-plugin.zip");
        let id = PluginChangeEvent::extract_plugin_id(&path);
        assert_eq!(id, Some("another-plugin".to_string()));
    }
}
