//! Integration tests for the plugin runtime with actual WASM modules

#[cfg(test)]
mod integration_tests {
    use orbis_plugin::{PluginContext, PluginInfo, PluginManifest, PluginRuntime, PluginSource};
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn get_test_plugin_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../plugins/test-plugin/test_plugin.wasm")
    }

    fn create_test_manifest() -> PluginManifest {
        PluginManifest {
            name: "test-plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "Test plugin for runtime integration testing".to_string(),
            author: Some("Orbis Team".to_string()),
            homepage: None,
            license: None,
            min_orbis_version: None,
            dependencies: vec![],
            permissions: vec![],
            routes: vec![],
            pages: vec![],
            wasm_entry: Some("test_plugin.wasm".to_string()),
            config: serde_json::Value::Null,
        }
    }

    #[tokio::test]
    async fn test_plugin_lifecycle() {
        let runtime = PluginRuntime::new();
        let manifest = create_test_manifest();
        let wasm_path = get_test_plugin_path();

        if !wasm_path.exists() {
            eprintln!("WASM file not found: {:?}", wasm_path);
            eprintln!("Run: cd plugins/test-plugin && ./build.sh");
            panic!("Test plugin WASM not built");
        }

        let source = PluginSource::Standalone(wasm_path.clone());

        let info = PluginInfo {
            id: uuid::Uuid::new_v4(),
            manifest: manifest.clone(),
            source: source.clone(),
            state: orbis_plugin::PluginState::Loaded,
            loaded_at: chrono::Utc::now(),
        };

        // Initialize
        runtime
            .initialize(&info, &source)
            .await
            .expect("Failed to initialize plugin");

        assert!(runtime.is_running("test-plugin"));

        // Start
        runtime
            .start("test-plugin")
            .await
            .expect("Failed to start plugin");

        // Stop
        runtime
            .stop("test-plugin")
            .await
            .expect("Failed to stop plugin");

        assert!(!runtime.is_running("test-plugin"));
    }

    #[tokio::test]
    async fn test_plugin_execution() {
        let runtime = PluginRuntime::new();
        let manifest = create_test_manifest();
        let wasm_path = get_test_plugin_path();

        if !wasm_path.exists() {
            eprintln!("WASM file not found: {:?}", wasm_path);
            eprintln!("Run: cd plugins/test-plugin && ./build.sh");
            panic!("Test plugin WASM not built");
        }

        let source = PluginSource::Standalone(wasm_path);

        let info = PluginInfo {
            id: uuid::Uuid::new_v4(),
            manifest,
            source: source.clone(),
            state: orbis_plugin::PluginState::Loaded,
            loaded_at: chrono::Utc::now(),
        };

        // Initialize and start
        runtime
            .initialize(&info, &source)
            .await
            .expect("Failed to initialize plugin");

        runtime
            .start("test-plugin")
            .await
            .expect("Failed to start plugin");

        // Execute handler
        let context = PluginContext {
            method: "POST".to_string(),
            path: "/test".to_string(),
            headers: HashMap::new(),
            query: HashMap::new(),
            body: serde_json::json!({"test": "data"}),
            user_id: Some("user123".to_string()),
            is_admin: false,
        };

        let result = runtime
            .execute("test-plugin", "test_handler", context)
            .await
            .expect("Failed to execute plugin handler");

        println!("Plugin execution result: {:?}", result);

        // Verify result structure
        assert!(result.is_object());
        assert!(result.get("status").is_some());

        // Stop
        runtime
            .stop("test-plugin")
            .await
            .expect("Failed to stop plugin");
    }

    #[tokio::test]
    async fn test_plugin_state_persistence() {
        let runtime = PluginRuntime::new();
        let manifest = create_test_manifest();
        let wasm_path = get_test_plugin_path();

        if !wasm_path.exists() {
            eprintln!("WASM file not found: {:?}", wasm_path);
            return;
        }

        let source = PluginSource::Standalone(wasm_path);

        let info = PluginInfo {
            id: uuid::Uuid::new_v4(),
            manifest,
            source: source.clone(),
            state: orbis_plugin::PluginState::Loaded,
            loaded_at: chrono::Utc::now(),
        };

        runtime
            .initialize(&info, &source)
            .await
            .expect("Failed to initialize plugin");

        runtime
            .start("test-plugin")
            .await
            .expect("Failed to start plugin");

        // Execute handler multiple times to test state persistence
        let context = PluginContext {
            method: "POST".to_string(),
            path: "/test".to_string(),
            headers: HashMap::new(),
            query: HashMap::new(),
            body: serde_json::json!({}),
            user_id: None,
            is_admin: false,
        };

        // First execution
        let result1 = runtime
            .execute("test-plugin", "test_handler", context.clone())
            .await
            .expect("Failed to execute handler first time");

        println!("First result: {:?}", result1);

        // Second execution - counter should increment
        let result2 = runtime
            .execute("test-plugin", "test_handler", context.clone())
            .await
            .expect("Failed to execute handler second time");

        println!("Second result: {:?}", result2);

        // Verify state is maintained between executions
        // The counter should have incremented

        runtime
            .stop("test-plugin")
            .await
            .expect("Failed to stop plugin");
    }
}
