use orbis_dsl::page::parse_file;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_component_whitelisting_example() {
    // Path relative to workspace root
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // crates
    path.pop(); // workspace root
    path.push("examples/component-whitelisting-simple.orbis");
    
    let input = fs::read_to_string(&path)
        .expect(&format!("Failed to read example file at {:?}", path));
    
    let result = parse_file(&input);
    
    // Should parse successfully (all components use valid attributes/events)
    assert!(
        result.is_ok(),
        "Component whitelisting example should parse successfully. Error: {:?}",
        result.err()
    );
}
