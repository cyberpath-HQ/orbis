//! Example of creating a complete plugin manifest with UI pages.

use orbis_plugin_api::*;
use std::collections::HashMap;

fn main() {
    // Create a simple plugin manifest
    let manifest = PluginManifest {
        name: "example-plugin".to_string(),
        version: "1.0.0".to_string(),
        description: "An example plugin demonstrating the API".to_string(),
        author: Some("Plugin Developer".to_string()),
        homepage: Some("https://example.com".to_string()),
        license: Some("MIT".to_string()),
        min_orbis_version: Some("0.1.0".to_string()),
        dependencies: vec![],
        permissions: vec![
            PluginPermission::DatabaseRead,
            PluginPermission::Network,
        ],
        routes: vec![
            PluginRoute {
                method: "GET".to_string(),
                path: "/api/data".to_string(),
                handler: "get_data".to_string(),
                description: Some("Fetch data from the plugin".to_string()),
                requires_auth: true,
                permissions: vec![],
                rate_limit: Some(60),
            },
        ],
        pages: vec![create_dashboard_page()],
        wasm_entry: Some("plugin.wasm".to_string()),
        config: serde_json::json!({}),
    };

    // Validate the manifest
    match manifest.validate() {
        Ok(()) => println!("✓ Manifest is valid"),
        Err(e) => eprintln!("✗ Manifest validation failed: {}", e),
    }

    // Serialize to JSON
    match serde_json::to_string_pretty(&manifest) {
        Ok(json) => println!("\nManifest JSON:\n{}", json),
        Err(e) => eprintln!("Failed to serialize manifest: {}", e),
    }
}

fn create_dashboard_page() -> PageDefinition {
    let mut state = HashMap::new();
    
    // Define state fields
    state.insert(
        "loading".to_string(),
        StateFieldDefinition {
            field_type: StateFieldType::Boolean,
            default: Some(serde_json::json!(false)),
            nullable: false,
            description: Some("Loading state".to_string()),
        },
    );
    
    state.insert(
        "data".to_string(),
        StateFieldDefinition {
            field_type: StateFieldType::Array,
            default: Some(serde_json::json!([])),
            nullable: false,
            description: Some("Data from API".to_string()),
        },
    );

    PageDefinition {
        route: "/dashboard".to_string(),
        title: "Dashboard".to_string(),
        icon: Some("LayoutDashboard".to_string()),
        description: Some("Plugin dashboard with data visualization".to_string()),
        show_in_menu: true,
        menu_order: 0,
        parent_route: None,
        requires_auth: true,
        permissions: vec![],
        roles: vec![],
        state,
        computed: HashMap::new(),
        sections: vec![
            // Container with header
            ComponentSchema {
                component_type: "Container".to_string(),
                id: Some("main".to_string()),
                class_name: Some("p-6".to_string()),
                style: None,
                visible: None,
                children: vec![
                    // Header
                    ComponentSchema {
                        component_type: "Text".to_string(),
                        id: None,
                        class_name: Some("text-2xl font-bold mb-4".to_string()),
                        style: None,
                        visible: None,
                        children: vec![],
                        events: None,
                        props: {
                            let mut props = HashMap::new();
                            props.insert("text".to_string(), serde_json::json!("Dashboard"));
                            props
                        },
                    },
                    // Loading indicator
                    ComponentSchema {
                        component_type: "Text".to_string(),
                        id: None,
                        class_name: None,
                        style: None,
                        visible: Some(serde_json::json!("${state.loading}")),
                        children: vec![],
                        events: None,
                        props: {
                            let mut props = HashMap::new();
                            props.insert("text".to_string(), serde_json::json!("Loading..."));
                            props
                        },
                    },
                    // Data table
                    ComponentSchema {
                        component_type: "Table".to_string(),
                        id: Some("dataTable".to_string()),
                        class_name: None,
                        style: None,
                        visible: Some(serde_json::json!("${!state.loading}")),
                        children: vec![],
                        events: None,
                        props: {
                            let mut props = HashMap::new();
                            props.insert("dataSource".to_string(), serde_json::json!("state:data"));
                            props.insert("columns".to_string(), serde_json::json!([
                                {
                                    "key": "id",
                                    "label": "ID",
                                    "sortable": true
                                },
                                {
                                    "key": "name",
                                    "label": "Name",
                                    "sortable": true
                                },
                                {
                                    "key": "value",
                                    "label": "Value",
                                    "sortable": false
                                }
                            ]));
                            props
                        },
                    },
                ],
                events: None,
                props: HashMap::new(),
            },
        ],
        actions: HashMap::new(),
        hooks: Some(PageLifecycleHooks {
            on_mount: vec![
                Action::CallApi {
                    name: Some("fetchData".to_string()),
                    api: "/api/plugins/example-plugin/api/data".to_string(),
                    method: Some("GET".to_string()),
                    args_from_state: vec![],
                    map_args: vec![],
                    body: None,
                    on_success: vec![
                        Action::UpdateState {
                            path: "data".to_string(),
                            value: None,
                            from: Some("response.data".to_string()),
                            merge: false,
                        },
                    ],
                    on_error: vec![
                        Action::ShowToast {
                            level: ToastLevel::Error,
                            message: "Failed to load data".to_string(),
                            title: Some("Error".to_string()),
                            duration: Some(5000),
                        },
                    ],
                    on_finally: vec![
                        Action::UpdateState {
                            path: "loading".to_string(),
                            value: Some(serde_json::json!(false)),
                            from: None,
                            merge: false,
                        },
                    ],
                },
            ],
            on_unmount: vec![],
            on_params_change: vec![],
            on_query_change: vec![],
        }),
        dialogs: vec![],
    }
}
