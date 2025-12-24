use orbis_dsl::page::parse_file;

#[test]
fn test_attribute_whitelisting_valid() {
    // Container with valid attributes (id, className, visible)
    let input = r#"template {
        <Container id="test" className="my-class" visible="true" />
    }"#;
    
    let result = parse_file(input);
    assert!(result.is_ok(), "Should parse Container with valid attributes");
}

#[test]
fn test_attribute_whitelisting_invalid() {
    // Container with INVALID attribute (message is only for Alert)
    let input = r#"template {
        <Container message="This should fail" />
    }"#;
    
    let result = parse_file(input);
    assert!(result.is_err(), "Should reject Container with 'message' attribute (not whitelisted)");
}

#[test]
fn test_alert_with_message() {
    // Alert with valid message attribute
    let input = r#"template {
        <Alert type="info" message="Hello world" />
    }"#;
    
    let result = parse_file(input);
    assert!(result.is_ok(), "Should parse Alert with message attribute");
}

#[test]
fn test_component_specific_events() {
    // Container with valid events (click, mouseEnter, mouseLeave)
    let input = r#"template {
        <Container @click => [state.count = state.count + 1] />
    }"#;
    
    let result = parse_file(input);
    assert!(result.is_ok(), "Should parse Container with click event");
}

#[test]
fn test_invalid_component_name() {
    // NonExistentComponent is not in the whitelist
    let input = r#"template {
        <NonExistentComponent id="test" />
    }"#;
    
    let result = parse_file(input);
    assert!(result.is_err(), "Should reject non-whitelisted component name");
}

#[test]
fn test_field_with_html_attributes() {
    // Field with HTML form attributes (placeholder, disabled, required, label)
    let input = r#"template {
        <Field 
            fieldName="username" 
            placeholder="Enter username"
            label="Username"
            required="true"
        />
    }"#;
    
    let result = parse_file(input);
    assert!(result.is_ok(), "Should parse Field with HTML form attributes");
}
