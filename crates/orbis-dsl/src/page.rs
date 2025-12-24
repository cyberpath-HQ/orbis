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
}
