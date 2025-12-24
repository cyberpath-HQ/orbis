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
                @mount => [
                    console.log("mounted")
                ]
                @unmount => [
                    console.log("cleanup")
                ]
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
                <Button label="Click" @click => [
                    state.count = state.count + 1
                ] />
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
    @mount => [
        console.log("Page mounted")
    ]
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
                @click => [
                    state.count = state.count + 1,
                    toast.show("Clicked!", level: success)
                ]
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
                        success => [
                            state.data = $response.body
                        ]
                        error => [
                            toast.show("Error", level: error)
                        ]
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
            @mount => [console.log("mounted")]
            
            @watch(state.count) => [
                console.log("Count changed")
            ]
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
            @watch(state.searchQuery, debounce: 300) => [
                api.call("search", query: state.searchQuery)
            ]
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
            @watch(state.firstName, state.lastName) => [
                console.log("Name changed")
            ]
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
            @watch(state.theme, immediate: true) => [
                console.log("Theme applied")
            ]
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
    @mount => [
        api.call("users") {
            success => [state.users = $response.data]
            error => [toast.show("Failed to load", level: error)]
        }
    ]
    
    @watch(state.searchQuery, debounce: 300) => [
        api.call("search", query: state.searchQuery) {
            success => [state.users = $response.data]
        }
    ]
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
                <UserCard user={user} @onClick => [state.selectedUser = user]>
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
}
