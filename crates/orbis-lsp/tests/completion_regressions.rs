use orbis_lsp::analysis::Analyzer;
use orbis_lsp::completion::get_completions;
use orbis_lsp::document::Document;
use tower_lsp::lsp_types::Position;
use tower_lsp::lsp_types::Url;

fn create_document_with_cursor(raw: &str) -> (Document, Position) {
    let mut content = raw.to_string();
    let cursor_idx = content
        .find("/*caret*/")
        .expect("Test content must include /*caret*/ marker");

    let before = &content[..cursor_idx];
    let line = before.lines().count() as u32 - 1;
    let character = before
        .rsplit('\n')
        .next()
        .map(|line| line.chars().count() as u32)
        .unwrap_or(0);

    content = content.replace("/*caret*/", "");

    let document = Document::new(
        Url::parse("file:///test.orbis").unwrap(),
        content,
        1,
        "orbis".to_string(),
    );

    (document, Position { line, character })
}

#[test]
fn completes_state_variables_after_state_prefix() {
    let raw = r#"state {
    count = 0
    name = "Ada"
}

page {
    id: "test"
}

hooks {
    @mount => {
        console.log(state./*caret*/)
    }
}
"#;

    let (document, position) = create_document_with_cursor(raw);
    let analysis = Analyzer::analyze(&document.text(), document.version);
    assert!(
        !analysis.symbols.state_vars.is_empty(),
        "Analyzer should collect state variables, found none"
    );
    let context = document.get_context(&position);
    let items = get_completions(&context, &analysis.symbols);

    let labels: Vec<_> = items.iter().map(|i| i.label.as_str()).collect();

    assert!(
        labels.contains(&"count") && labels.contains(&"name"),
        "state.<var> should suggest declared state variables, got {:?}",
        labels
    );
}

#[test]
fn completes_console_methods_after_console_prefix() {
    let raw = r#"hooks {
    @mount => {
        console./*caret*/
    }
}
"#;

    let (document, position) = create_document_with_cursor(raw);
    let analysis = Analyzer::analyze(&document.text(), document.version);
    let context = document.get_context(&position);
    let items = get_completions(&context, &analysis.symbols);

    let labels: Vec<_> = items.iter().map(|i| i.label.as_str()).collect();
    let expected = ["console.log", "console.info", "console.warn", "console.error", "console.debug"];

    for method in expected {
        assert!(
            labels.contains(&method),
            "console.<method> should include {} suggestions, got {:?}",
            method,
            labels
        );
    }
}

#[test]
fn completes_component_attributes_after_event_handler() {
    let raw = r#"template {
    <Form @submit => { console.log("x") }>
        <Button /*caret*/ />
    </Form>
}
"#;

    let (document, position) = create_document_with_cursor(raw);
    let analysis = Analyzer::analyze(&document.text(), document.version);
    let context = document.get_context(&position);
    let items = get_completions(&context, &analysis.symbols);

    let labels: Vec<_> = items.iter().map(|i| i.label.as_str()).collect();

    assert!(
        labels.contains(&"label") && labels.contains(&"variant"),
        "Component attribute completions should include Button props after event handler, got {:?}",
        labels
    );
}
