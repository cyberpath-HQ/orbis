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
fn test_builtin_component_with_invalid_attribute() {
    // Container with INVALID attribute (message is only for Alert)
    // NOTE: With fragment support, the parser now accepts any attributes and
    // falls back to fragment syntax. Semantic validation should catch invalid
    // attributes on built-in components.
    //
    // This is by design: fragments allow any attributes, and distinguishing
    // between built-in components and fragments at parse time would require
    // complex negative lookaheads. Semantic analysis is the right place to
    // validate built-in component attributes.
    let input = r#"template {
        <Container message="This would fail at semantic validation" />
    }"#;
    
    let result = parse_file(input);
    // Parser accepts it (as potential fragment usage), semantic validator would reject
    assert!(result.is_ok(), "Parser accepts any attributes, semantic validation catches errors");
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
fn test_fragment_usage_as_component() {
    // User-defined fragments can be used like components
    let input = r#"template {
        <UserCard user={state.user} />
    }"#;
    
    let result = parse_file(input);
    assert!(result.is_ok(), "Should parse user-defined fragment usage");
}

#[test]
fn test_fragment_with_slot_content() {
    // Fragments can have slot content with named slots (Astro-like)
    let input = r#"template {
        <Modal title="Confirm">
            <Text content="Main content" />
            <Container slot="footer">
                <Button label="OK" />
            </Container>
        </Modal>
    }"#;
    
    let result = parse_file(input);
    assert!(result.is_ok(), "Should parse fragment with slot content");
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
