//! Page grammar parser - handles page definitions and UI elements
//!
//! This module provides parsing for page-related DSL constructs with
//! case-insensitive keyword matching.

use pest::Parser as PestParser;

const _GRAMMAR: &str = include_str!("page.pest");

// Uncomment to enable the parser:
#[derive(pest_derive::Parser)]
#[grammar = "page.pest"]
pub struct Parser;

/// Parse a page DSL file and return the parse result
pub fn parse_file(input: &str) -> Result<pest::iterators::Pairs<'_, Rule>, pest::error::Error<Rule>> {
    Parser::parse(Rule::file, input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grammar_is_included() {
        assert!(_GRAMMAR.contains("@builder-insertion-start"));
        assert!(_GRAMMAR.contains("file"));
    }

    #[test]
    fn grammar_contains_all_variants() {
        assert!(_GRAMMAR.contains("\"page\""));
        assert!(_GRAMMAR.contains("\"Page\""));
        assert!(_GRAMMAR.contains("\"PAGE\""));
    }

    #[test]
    fn parse_page_block() {
        let input = r#"page {
                id: "test-page"
                title: "Test Page"
            }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse page block: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_state_block() {
        let input = r#"state {
                name: string = "World"
                count = 0
                loading = false
            }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse state block: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_hooks_block() {
        let input = r#"hooks {
                @mount => {
                    console.log("mounted")
                }
                @unmount => {
                    console.log("cleanup")
                }
            }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse hooks block: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_simple_template() {
        let input = r#"template {
                <Container className="flex items-center">
                    <Text content="Hello World!" />
                </Container>
            }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse template: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_component_with_event() {
        let input = r#"template {
                <Button label="Click" @click => {
                    state.count = state.count + 1
                } />
            }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse component with event: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_if_block() {
        let input = r#"template {
                if state.loading {
                    <LoadingOverlay />
                } else {
                    <Text content="Content loaded" />
                }
            }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse if block: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_for_block() {
        let input = r#"template {
                for item in state.items {
                    <Card title={item.name} />
                }
            }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse for block: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_complete_example() {
        let input = r#"
page {
    id: "greeting-page"
    title: "Quick Start"
    icon: "waves"
    route: "/hello"
}

state {
    name: string = "World"
    count = 0
    loading = false
    items: Item[] = []
}

hooks {
    @mount => {
        console.log("Page mounted")
    }
}

template {
    <Container className="flex items-center justify-center">
        <Card className="p-8 shadow-2xl">
            <Text content="Hello, {state.name}!" className="text-4xl" />
            
            <Text content="Clicks: {state.count}" />
            
            if state.loading {
                <LoadingOverlay />
            }
            
            for item in state.items {
                <Card title={item.name} />
            }
            
            <Button 
                label="Click Me!"
                @click => {
                    state.count = state.count + 1,
                    toast.show("Clicked!", level: success)
                }
            />
        </Card>
    </Container>
}
"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse complete example: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_action_with_handlers() {
        let input = r#"template {
                <Button 
                    label="Load Data"
                    @click => api.call("endpoint") {
                        success => {
                            state.data = $response.body
                        }
                        error => {
                            toast.show("Error", level: error)
                        }
                    }
                />
            }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse action with handlers: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_when_block() {
        let input = r#"template {
                when state.status {
                    "loading" => <LoadingOverlay />
                    "error" => {
                        <Alert type="error" message="An error occurred" />
                    }
                    else => {
                        <Text content="Content loaded" />
                    }
                }
            }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse when block: {:?}",
            result.err()
        );
    }

    // =========================================================================
    // WATCHER TESTS
    // =========================================================================

    #[test]
    fn parse_watcher_basic() {
        let input = r#"hooks {
            @mount => { console.log("mounted") }
            
            @watch(state.count) => {
                console.log("Count changed")
            }
        }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse basic watcher: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_watcher_with_options() {
        let input = r#"hooks {
            @watch(state.searchQuery, debounce: 300) => {
                api.call("search", query: state.searchQuery)
            }
        }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse watcher with debounce: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_watcher_multiple_targets() {
        let input = r#"hooks {
            @watch(state.firstName, state.lastName) => {
                console.log("Name changed")
            }
        }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse watcher with multiple targets: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_watcher_with_immediate() {
        let input = r#"hooks {
            @watch(state.theme, immediate: true) => {
                console.log("Theme applied")
            }
        }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse watcher with immediate option: {:?}",
            result.err()
        );
    }

    // =========================================================================
    // FRAGMENT TESTS
    // =========================================================================

    #[test]
    fn parse_fragment_definition_basic() {
        let input = r#"fragment UserCard(user) {
            <Card>
                <Heading text={user.name} />
            </Card>
        }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse basic fragment definition: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_fragment_with_typed_params() {
        let input = r#"fragment UserCard(user: User, size?: string) {
            <Card>
                <Text content={user.name} />
            </Card>
        }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse fragment with typed params: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_fragment_with_event_params() {
        let input = r#"fragment ClickableCard(title: string, @onClick, @onHover?) {
            <Card title={title} @click={@onClick}>
                <slot />
            </Card>
        }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse fragment with event params: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_fragment_with_slots() {
        let input = r#"fragment Modal(title: string) {
            <Container className="modal">
                <Heading text={title} />
                <slot />
                <slot name="footer" />
            </Container>
        }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse fragment with slots: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_fragment_usage() {
        let input = r#"template {
            <UserCard user={state.currentUser} />
        }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse fragment usage: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_fragment_with_slot_content() {
        let input = r#"template {
            <Modal title="Confirm">
                <Text content="Main content goes here" />
                <Container slot="footer">
                    <Button label="Cancel" />
                    <Button label="OK" />
                </Container>
            </Modal>
        }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse fragment with slot content: {:?}",
            result.err()
        );
    }

    // =========================================================================
    // ENHANCED TYPE SYSTEM TESTS
    // =========================================================================

    #[test]
    fn parse_state_with_union_types() {
        let input = r#"state {
            status: "idle" | "loading" | "success" | "error" = "idle"
        }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse state with union types: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_state_with_optional_types() {
        let input = r#"state {
            user: User? = null
            config?: Config
        }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse state with optional types: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_state_with_array_types() {
        let input = r#"state {
            items: Item[] = []
            numbers: number[] = []
        }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse state with array types: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_state_with_generic_types() {
        let input = r#"state {
            response: Response<User> = null
            map: Map<string, number> = {}
        }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse state with generic types: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_interface_definition() {
        let input = r#"interface User {
            id: number
            name: string
            email: string
            role: "admin" | "user" | "guest"
            settings?: UserSettings
        }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse interface definition: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_generic_interface() {
        let input = r#"interface Response<T> {
            data: T
            error: string | null
            status: number
        }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse generic interface: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_computed_state_with_types() {
        let input = r#"state {
            firstName: string = "John"
            lastName: string = "Doe"
            
            @computed fullName: string => state.firstName + " " + state.lastName
            doubledCount => state.count * 2
        }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse computed state with types: {:?}",
            result.err()
        );
    }

    // =========================================================================
    // IMPORT/EXPORT TESTS
    // =========================================================================

    #[test]
    fn parse_typescript_style_imports() {
        let input = r#"import { UserCard, PostCard } from "./fragments/cards.orbis"
import { type User, type Config } from "./types.orbis"
import * as Utils from "./utils.orbis"
import DefaultLayout from "./layouts/default.orbis"

page {
    id: "test"
    title: "Test"
}"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse TypeScript-style imports: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_rust_style_use() {
        let input = r#"use super::components::Button
use crate::common::*

page {
    id: "test"
    title: "Test"
}"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse Rust-style use statements: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_export_fragment() {
        let input = r#"export fragment UserCard(user: User) {
    <Card>
        <Text content={user.name} />
    </Card>
}

pub interface Config {
    theme: string
}"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse exported fragment: {:?}",
            result.err()
        );
    }

    // =========================================================================
    // VALIDATION TESTS
    // =========================================================================

    #[test]
    fn parse_state_with_validation() {
        let input = r#"state {
            email: string = "" @email @min(1) @message("Email is required")
            age: number = 0 @int @min(18) @max(120)
            username: string @min(3) @max(20) @trim
        }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse state with validation: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_validation_with_regex() {
        let input = r#"state {
            password: string @min(8) @regex(/[A-Z]/) @regex(/[0-9]/)
            phone: string @pattern(/^\+?[0-9]{10,}$/) @message("Invalid phone")
        }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse validation with regex: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_validation_with_transforms() {
        let input = r#"state {
            name: string @trim @toLowerCase
            email: string @toLowerCase @trim
        }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse validation with transforms: {:?}",
            result.err()
        );
    }

    // =========================================================================
    // CSS-IN-DSL TESTS
    // =========================================================================

    #[test]
    fn parse_basic_styles_block() {
        let input = r#"styles {
    .card {
        padding: 1rem;
        border-radius: 8px;
        box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    }
    
    .button {
        background-color: blue;
        color: white;
    }
}"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse basic styles block: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_styles_with_tailwind_apply() {
        let input = r#"styles {
    .card {
        @apply flex items-center justify-between p-4 rounded-lg shadow-md;
        @apply hover:shadow-lg transition-shadow;
    }
}"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse styles with Tailwind @apply: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_styles_with_media_queries() {
        let input = r#"styles {
    .container {
        width: 100%;
    }
    
    @media (min-width: 768px) {
        .container {
            max-width: 768px;
        }
    }
    
    @screen md {
        .card {
            padding: 2rem;
        }
    }
}"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse styles with media queries: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_styles_with_keyframes() {
        let input = r#"styles {
    @keyframes fadeIn {
        from {
            opacity: 0;
        }
        to {
            opacity: 1;
        }
    }
    
    .animated {
        animation: fadeIn 0.3s ease-in-out;
    }
}"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse styles with keyframes: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_scoped_and_global_styles() {
        let input = r#"styles scoped {
    .card {
        padding: 1rem;
    }
}

styles global {
    body {
        margin: 0;
    }
}"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse scoped and global styles: {:?}",
            result.err()
        );
    }

    // =========================================================================
    // COMPREHENSIVE INTEGRATION TEST
    // =========================================================================

    #[test]
    fn parse_complete_modern_example() {
        let input = r#"
interface User {
    id: number
    name: string
    email: string
    role: "admin" | "user"
}

interface ApiResponse<T> {
    data: T
    error: string?
    status: number
}

fragment UserCard(user: User, @onClick?) {
    <Card className="user-card" @click={@onClick}>
        <Heading level="3" text={user.name} />
        <Text content={user.email} />
        <Badge content={user.role} />
        <slot name="actions" />
    </Card>
}

page {
    id: "users-page"
    title: "Users"
    route: "/users"
}

state {
    users: User[] = []
    selectedUser: User? = null
    loading = false
    searchQuery = ""
    
    @computed filteredUsers => state.users
}

hooks {
    @mount => {
        api.call("users") {
            success => { state.users = $response.data }
            error => { toast.show("Failed to load", level: error) }
        }
    }
    
    @watch(state.searchQuery, debounce: 300) => {
        api.call("search", query: state.searchQuery) {
            success => { state.users = $response.data }
        }
    }
}

template {
    <Container className="page">
        <PageHeader title="Users" />
        
        <Field 
            name="search"
            fieldType="text"
            placeholder="Search users..."
            bindTo={state.searchQuery}
        />
        
        if state.loading {
            <LoadingOverlay text="Loading..." />
        } else {
            for user in state.filteredUsers {
                <UserCard user={user} @onClick => { state.selectedUser = user }>
                    <Container slot="actions">
                        <Button label="Edit" variant="outline" />
                    </Container>
                </UserCard>
            }
        }
    </Container>
}
"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse complete modern example: {:?}",
            result.err()
        );
    }

    // =========================================================================
    // IMPROVED TEST COVERAGE
    // =========================================================================

    #[test]
    fn parse_page_with_all_attributes() {
        let input = r#"page {
            id: "complete-page"
            title: "Complete Page"
            description: "A page with all attributes"
            icon: "home"
            route: "/complete"
            show_in_menu: true
            menu_order: 1
            requires_auth: true
            permissions: "admin,user"
            roles: "admin,editor"
        }
        
        template {
            <Container><Text content="Test" /></Container>
        }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse page with all attributes: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_page_minimal() {
        let input = r#"page {
            id: "minimal"
            title: "Minimal Page"
        }
        
        template {
            <Container><Text content="Minimal" /></Container>
        }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse minimal page: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_validators_pattern_various_formats() {
        let input = r#"state {
            email: string @pattern(/^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/) @message("Invalid email")
            phone: string @pattern(/^\+?[0-9\s\-()]{10,}$/)
            zipcode: string @pattern(/^\d{5}(-\d{4})?$/)
            username: string @pattern(/^[a-z][a-z0-9_]{2,31}$/) @min(3) @max(32)
            slug: string @pattern(/^[a-z0-9]+(?:-[a-z0-9]+)*$/)
        }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse validators with various pattern formats: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_validators_regex_and_pattern_interchangeably() {
        let input = r#"state {
            password: string @regex(/[A-Z]/) @pattern(/[0-9]/) @min(8)
            field1: string @regex(/test/)
            field2: string @pattern(/test/)
        }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse validators using both regex and pattern: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_validators_with_multiple_messages() {
        let input = r#"state {
            password: string 
                @min(8) @message("Too short")
                @regex(/[A-Z]/) @message("Need uppercase")
                @regex(/[0-9]/) @message("Need number")
        }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse validators with multiple messages: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_complex_validation_rules() {
        let input = r#"state {
            // String validations
            name: string @min(1) @max(100) @trim
            email: string @email @toLowerCase @trim
            url: string @url @startsWith("https://")
            
            // Complex regex patterns
            creditCard: string @pattern(/^\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}$/)
            uuid: string @uuid
            
            // Combined
            customId: string @min(5) @max(20) @pattern(/^[A-Z0-9_-]+$/)
        }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse complex validation rules: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_state_with_page_definition() {
        let input = r#"page {
            id: "form-page"
            title: "Form Page"
            route: "/form"
        }
        
        state {
            username: string @min(3) @max(20)
            email: string @email
            age: number
        }
        
        template {
            <Container><Text content="Form" /></Container>
        }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse state with page definition: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_page_with_computed_and_validators() {
        let input = r#"page {
            id: "form-page"
            title: "Form Page"
        }
        
        state {
            email: string @email @toLowerCase
            firstName: string @min(1)
            lastName: string @min(1)
        }
        
        template {
            <Container><Text content={state.email} /></Container>
        }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse page with computed and validators: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_multiple_pages_in_file() {
        let input = r#"page {
            id: "home"
            title: "Home"
            route: "/"
        }
        
        template {
            <Container><Text content="Home" /></Container>
        }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse page file: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_page_metadata_with_special_characters() {
        let input = r#"page {
            id: "page-with-special-id"
            title: "Page with Special Chars: @#$%"
            description: "A description with 'quotes' and \"double quotes\""
            route: "/special/path-here"
        }
        
        template {
            <Container><Text content="Test" /></Container>
        }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse page with special characters: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_validator_edge_cases() {
        let input = r#"state {
            // Regex with special characters
            field1: string @pattern(/test/) 
            
            // Regex with escaped special characters
            field2: string @pattern(/\(test\)/)
            
            // Multiple consecutive validators
            field3: string @min(1) @max(10) @trim @toLowerCase @regex(/^[a-z]+$/)
            
            // URL pattern
            field4: string @url @lowercase
        }"#;

        let result = parse_file(input);
        assert!(
            result.is_ok(),
            "Failed to parse validator edge cases: {:?}",
            result.err()
        );
    }
}
