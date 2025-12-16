//! Plugin loader for loading plugins from various sources.

use super::PluginManifest;
use std::path::PathBuf;

/// Plugin source location and flavor.
#[derive(Debug, Clone)]
pub enum PluginSource {
    /// Packed: ZIP archive containing WASM, manifest.json, and assets.
    Packed(PathBuf),

    /// Unpacked: Folder containing WASM, manifest.json, and assets.
    Unpacked(PathBuf),

    /// Standalone: Single WASM file with embedded manifest.
    Standalone(PathBuf),

    /// Remote URL (for future use).
    Remote(String),
}

impl PluginSource {
    /// Create a plugin source from a path.
    ///
    /// # Errors
    ///
    /// Returns an error if the path is invalid.
    pub fn from_path(path: &PathBuf) -> orbis_core::Result<Self> {
        if !path.exists() {
            return Err(orbis_core::Error::plugin(format!(
                "Plugin path does not exist: {:?}",
                path
            )));
        }

        if path.is_dir() {
            // Unpacked: directory containing plugin files
            Ok(Self::Unpacked(path.clone()))
        } else if let Some(ext) = path.extension() {
            match ext.to_str() {
                Some("wasm") => Ok(Self::Standalone(path.clone())),
                Some("zip") => Ok(Self::Packed(path.clone())),
                _ => Err(orbis_core::Error::plugin(format!(
                    "Unsupported plugin file type: {:?}. Expected .wasm or .zip",
                    ext
                ))),
            }
        } else {
            Err(orbis_core::Error::plugin(
                "Cannot determine plugin type from path",
            ))
        }
    }

    /// Get the manifest path for this source.
    /// Returns None for standalone plugins (manifest is embedded in WASM).
    #[must_use]
    pub fn manifest_path(&self) -> Option<PathBuf> {
        match self {
            Self::Unpacked(dir) => Some(dir.join("manifest.json")),
            Self::Packed(_) => None, // Will be extracted from ZIP
            Self::Standalone(_) => None, // Manifest embedded in WASM
            Self::Remote(_) => None,
        }
    }

    /// Get the plugin WASM entry point.
    #[must_use]
    pub fn entry_point(&self, manifest: &PluginManifest) -> Option<PathBuf> {
        match self {
            Self::Unpacked(dir) => {
                if let Some(wasm_entry) = &manifest.wasm_entry {
                    Some(dir.join(wasm_entry))
                } else {
                    // Default to plugin.wasm
                    Some(dir.join("plugin.wasm"))
                }
            }
            Self::Standalone(path) => Some(path.clone()),
            Self::Packed(_) => None, // Will be extracted from ZIP
            Self::Remote(_) => None,
        }
    }
}

impl Default for PluginSource {
    fn default() -> Self {
        Self::Unpacked(PathBuf::new())
    }
}

/// Plugin loader for loading plugin manifests and code.
pub struct PluginLoader;

impl PluginLoader {
    /// Create a new plugin loader.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Load a plugin manifest from a source.
    ///
    /// For Standalone plugins or when manifest is embedded, extracts it from WASM custom section.
    /// For Packed plugins, extracts manifest.json from ZIP.
    /// For Unpacked plugins, reads manifest.json from directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the manifest cannot be loaded.
    pub fn load_manifest(&self, source: &PluginSource) -> orbis_core::Result<PluginManifest> {
        match source {
            PluginSource::Unpacked(dir) => {
                let manifest_path = dir.join("manifest.json");
                
                // Try to load external manifest first
                if manifest_path.exists() {
                    let content = std::fs::read_to_string(&manifest_path).map_err(|e| {
                        orbis_core::Error::plugin(format!("Failed to read manifest: {}", e))
                    })?;
                    
                    let manifest: PluginManifest = serde_json::from_str(&content).map_err(|e| {
                        orbis_core::Error::plugin(format!("Failed to parse manifest: {}", e))
                    })?;
                    
                    return Ok(manifest);
                }
                
                // Fallback to embedded manifest in WASM
                // Try common names first
                let wasm_path = dir.join("plugin.wasm");
                if wasm_path.exists() {
                    return self.extract_embedded_manifest(&wasm_path);
                }
                
                // Look for any .wasm file in the directory
                if let Ok(entries) = std::fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                            return self.extract_embedded_manifest(&path);
                        }
                    }
                }
                
                Err(orbis_core::Error::plugin(
                    "No manifest.json found and no .wasm file with embedded manifest"
                ))
            }
            
            PluginSource::Packed(zip_path) => {
                self.load_manifest_from_zip(zip_path)
            }
            
            PluginSource::Standalone(wasm_path) => {
                // For standalone, manifest MUST be embedded in WASM
                self.extract_embedded_manifest(wasm_path)
            }
            
            PluginSource::Remote(_) => {
                Err(orbis_core::Error::plugin("Remote plugins not yet supported"))
            }
        }
    }
    
    /// Extract manifest from ZIP archive.
    fn load_manifest_from_zip(&self, zip_path: &PathBuf) -> orbis_core::Result<PluginManifest> {
        use std::io::Read;
        
        let file = std::fs::File::open(zip_path).map_err(|e| {
            orbis_core::Error::plugin(format!("Failed to open ZIP file: {}", e))
        })?;
        
        let mut archive = zip::ZipArchive::new(file).map_err(|e| {
            orbis_core::Error::plugin(format!("Failed to read ZIP archive: {}", e))
        })?;
        
        // Try to find manifest.json in the archive
        if let Ok(mut file) = archive.by_name("manifest.json") {
            let mut content = String::new();
            file.read_to_string(&mut content).map_err(|e| {
                orbis_core::Error::plugin(format!("Failed to read manifest from ZIP: {}", e))
            })?;
            
            let manifest: PluginManifest = serde_json::from_str(&content).map_err(|e| {
                orbis_core::Error::plugin(format!("Failed to parse manifest: {}", e))
            })?;
            
            return Ok(manifest);
        }
        
        // Try in subdirectory (common pattern)
        if let Ok(mut file) = archive.by_name("plugin/manifest.json") {
            let mut content = String::new();
            file.read_to_string(&mut content).map_err(|e| {
                orbis_core::Error::plugin(format!("Failed to read manifest from ZIP: {}", e))
            })?;
            
            let manifest: PluginManifest = serde_json::from_str(&content).map_err(|e| {
                orbis_core::Error::plugin(format!("Failed to parse manifest: {}", e))
            })?;
            
            return Ok(manifest);
        }
        
        // Fallback: try to find WASM file and extract embedded manifest
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| {
                orbis_core::Error::plugin(format!("Failed to access ZIP entry: {}", e))
            })?;
            
            if file.name().ends_with(".wasm") {
                let mut wasm_bytes = Vec::new();
                file.read_to_end(&mut wasm_bytes).map_err(|e| {
                    orbis_core::Error::plugin(format!("Failed to read WASM from ZIP: {}", e))
                })?;
                
                return self.extract_embedded_manifest_from_bytes(&wasm_bytes);
            }
        }
        
        Err(orbis_core::Error::plugin(
            "No manifest.json or .wasm file found in ZIP archive"
        ))
    }
    
    /// Extract embedded manifest from WASM file.
    fn extract_embedded_manifest(&self, wasm_path: &PathBuf) -> orbis_core::Result<PluginManifest> {
        let wasm_bytes = std::fs::read(wasm_path).map_err(|e| {
            orbis_core::Error::plugin(format!("Failed to read WASM file: {}", e))
        })?;
        
        self.extract_embedded_manifest_from_bytes(&wasm_bytes)
    }
    
    /// Extract embedded manifest from WASM bytes.
    /// Looks for a custom section named "manifest" containing the JSON manifest.
    fn extract_embedded_manifest_from_bytes(&self, wasm_bytes: &[u8]) -> orbis_core::Result<PluginManifest> {
        use wasmparser::{Parser, Payload};
        
        // Parse WASM module using wasmparser
        for payload in Parser::new(0).parse_all(wasm_bytes) {
            let payload = payload.map_err(|e| {
                orbis_core::Error::plugin(format!("Failed to parse WASM: {}", e))
            })?;
            
            // Look for custom section named "manifest"
            if let Payload::CustomSection(reader) = payload {
                if reader.name() == "manifest" {
                    // Found manifest section!
                    let manifest_json = reader.data();
                    let manifest_str = std::str::from_utf8(manifest_json).map_err(|_| {
                        orbis_core::Error::plugin("Manifest section is not valid UTF-8")
                    })?;
                    
                    let manifest: PluginManifest = serde_json::from_str(manifest_str).map_err(|e| {
                        orbis_core::Error::plugin(format!("Failed to parse embedded manifest: {}", e))
                    })?;
                    
                    return Ok(manifest);
                }
            }
        }
        
        Err(orbis_core::Error::plugin(
            "No embedded manifest found in WASM custom section"
        ))
    }
    
    /// Read a LEB128-encoded unsigned integer.
    /// This is kept for potential future use but not needed with wasmparser.
    #[allow(dead_code)]
    fn read_leb128(&self, bytes: &[u8]) -> Result<(usize, usize), ()> {
        let mut result = 0usize;
        let mut shift = 0;
        let mut pos = 0;
        
        loop {
            if pos >= bytes.len() || shift >= 64 {
                return Err(());
            }
            
            let byte = bytes[pos];
            pos += 1;
            
            result |= ((byte & 0x7F) as usize) << shift;
            
            if byte & 0x80 == 0 {
                break;
            }
            
            shift += 7;
        }
        
        Ok((result, pos))
    }

    /// Load plugin WASM code.
    ///
    /// # Errors
    ///
    /// Returns an error if the code cannot be loaded.
    pub fn load_code(&self, source: &PluginSource, manifest: &PluginManifest) -> orbis_core::Result<Vec<u8>> {
        match source {
            PluginSource::Unpacked(dir) => {
                let wasm_path = if let Some(entry) = &manifest.wasm_entry {
                    dir.join(entry)
                } else {
                    dir.join("plugin.wasm")
                };
                
                if !wasm_path.exists() {
                    return Err(orbis_core::Error::plugin(format!(
                        "WASM file not found: {:?}",
                        wasm_path
                    )));
                }
                
                std::fs::read(&wasm_path).map_err(|e| {
                    orbis_core::Error::plugin(format!("Failed to read WASM file: {}", e))
                })
            }
            
            PluginSource::Standalone(wasm_path) => {
                std::fs::read(wasm_path).map_err(|e| {
                    orbis_core::Error::plugin(format!("Failed to read WASM file: {}", e))
                })
            }
            
            PluginSource::Packed(zip_path) => {
                self.load_wasm_from_zip(zip_path, manifest)
            }
            
            PluginSource::Remote(_) => {
                Err(orbis_core::Error::plugin("Remote plugins not yet supported"))
            }
        }
    }
    
    /// Load WASM from ZIP archive.
    fn load_wasm_from_zip(&self, zip_path: &PathBuf, manifest: &PluginManifest) -> orbis_core::Result<Vec<u8>> {
        use std::io::Read;
        
        let file = std::fs::File::open(zip_path).map_err(|e| {
            orbis_core::Error::plugin(format!("Failed to open ZIP file: {}", e))
        })?;
        
        let mut archive = zip::ZipArchive::new(file).map_err(|e| {
            orbis_core::Error::plugin(format!("Failed to read ZIP archive: {}", e))
        })?;
        
        // Determine WASM filename
        let wasm_name = manifest.wasm_entry.as_deref().unwrap_or("plugin.wasm");
        
        // Try to find the WASM file
        let mut wasm_file = if let Ok(file) = archive.by_name(wasm_name) {
            file
        } else if let Ok(file) = archive.by_name(&format!("plugin/{}", wasm_name)) {
            file
        } else {
            return Err(orbis_core::Error::plugin(format!("WASM file '{}' not found in ZIP", wasm_name)));
        };
        
        let mut wasm_bytes = Vec::new();
        wasm_file.read_to_end(&mut wasm_bytes).map_err(|e| {
            orbis_core::Error::plugin(format!("Failed to read WASM from ZIP: {}", e))
        })?;
        
        Ok(wasm_bytes)
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}
