use orbis_plugin::{PluginLoader, PluginSource};
use std::path::PathBuf;

fn get_samples_dir() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.parent().unwrap().parent().unwrap().join("sample-plugins/hello-plugin")
}

#[test]
fn test_unpacked_external_manifest() {
    let loader = PluginLoader::new();
    let path = get_samples_dir().join("unpacked-external");
    let source = PluginSource::from_path(&path).expect("Failed to create source");
    
    let manifest = loader.load_manifest(&source).expect("Failed to load manifest");
    
    assert_eq!(manifest.name, "Hello Plugin");
    assert_eq!(manifest.version, "0.1.0");
    assert_eq!(manifest.description, "A simple hello world plugin demonstrating Orbis plugin system");
    assert_eq!(manifest.author, Some("Orbis Team".to_string()));
}

#[test]
fn test_unpacked_embedded_manifest() {
    let loader = PluginLoader::new();
    let path = get_samples_dir().join("unpacked-embedded");
    let source = PluginSource::from_path(&path).expect("Failed to create source");
    
    let manifest = loader.load_manifest(&source).expect("Failed to load manifest");
    
    assert_eq!(manifest.name, "Hello Plugin");
    assert_eq!(manifest.version, "0.1.0");
}

#[test]
fn test_standalone_manifest() {
    let loader = PluginLoader::new();
    let path = get_samples_dir().join("standalone.wasm");
    let source = PluginSource::from_path(&path).expect("Failed to create source");
    
    let manifest = loader.load_manifest(&source).expect("Failed to load manifest");
    
    assert_eq!(manifest.name, "Hello Plugin");
    assert_eq!(manifest.version, "0.1.0");
}

#[test]
fn test_packed_external_manifest() {
    let loader = PluginLoader::new();
    let path = get_samples_dir().join("packed-external.zip");
    let source = PluginSource::from_path(&path).expect("Failed to create source");
    
    let manifest = loader.load_manifest(&source).expect("Failed to load manifest");
    
    assert_eq!(manifest.name, "Hello Plugin");
    assert_eq!(manifest.version, "0.1.0");
}

#[test]
fn test_packed_embedded_manifest() {
    let loader = PluginLoader::new();
    let path = get_samples_dir().join("packed-embedded.zip");
    let source = PluginSource::from_path(&path).expect("Failed to create source");
    
    let manifest = loader.load_manifest(&source).expect("Failed to load manifest");
    
    assert_eq!(manifest.name, "Hello Plugin");
    assert_eq!(manifest.version, "0.1.0");
}

#[test]
fn test_load_wasm_code_unpacked() {
    let loader = PluginLoader::new();
    let path = get_samples_dir().join("unpacked-external");
    let source = PluginSource::from_path(&path).expect("Failed to create source");
    let manifest = loader.load_manifest(&source).expect("Failed to load manifest");
    
    let code = loader.load_code(&source, &manifest).expect("Failed to load WASM code");
    
    // Verify WASM magic number
    assert_eq!(&code[0..4], b"\0asm");
}

#[test]
fn test_load_wasm_code_standalone() {
    let loader = PluginLoader::new();
    let path = get_samples_dir().join("standalone.wasm");
    let source = PluginSource::from_path(&path).expect("Failed to create source");
    let manifest = loader.load_manifest(&source).expect("Failed to load manifest");
    
    let code = loader.load_code(&source, &manifest).expect("Failed to load WASM code");
    
    // Verify WASM magic number
    assert_eq!(&code[0..4], b"\0asm");
}

#[test]
fn test_load_wasm_code_packed() {
    let loader = PluginLoader::new();
    let path = get_samples_dir().join("packed-external.zip");
    let source = PluginSource::from_path(&path).expect("Failed to create source");
    let manifest = loader.load_manifest(&source).expect("Failed to load manifest");
    
    let code = loader.load_code(&source, &manifest).expect("Failed to load WASM code");
    
    // Verify WASM magic number
    assert_eq!(&code[0..4], b"\0asm");
}
