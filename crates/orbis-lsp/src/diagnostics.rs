//! Diagnostics
//!
//! This module handles converting analysis errors to LSP diagnostics
//! with rich error messages and suggestions.

use tower_lsp::lsp_types::{
    Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity, DiagnosticTag, Location,
    NumberOrString, Position, Range, Url,
};

use crate::analysis::{AnalysisError, AnalysisResult, ErrorSeverity};

/// Convert analysis result to LSP diagnostics
pub fn to_diagnostics(result: &AnalysisResult, uri: &Url) -> Vec<Diagnostic> {
    result
        .errors
        .iter()
        .map(|error| error_to_diagnostic(error, uri))
        .collect()
}

/// Convert a single analysis error to an LSP diagnostic
fn error_to_diagnostic(error: &AnalysisError, uri: &Url) -> Diagnostic {
    let range = error
        .span
        .as_ref()
        .map(|span| Range {
            start: Position {
                line: (span.start_line.saturating_sub(1)) as u32,
                character: (span.start_col.saturating_sub(1)) as u32,
            },
            end: Position {
                line: (span.end_line.saturating_sub(1)) as u32,
                character: (span.end_col.saturating_sub(1)) as u32,
            },
        })
        .unwrap_or(Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 0,
            },
        });

    let severity = match error.severity {
        ErrorSeverity::Error => DiagnosticSeverity::ERROR,
        ErrorSeverity::Warning => DiagnosticSeverity::WARNING,
        ErrorSeverity::Information => DiagnosticSeverity::INFORMATION,
        ErrorSeverity::Hint => DiagnosticSeverity::HINT,
    };

    // Build the message with suggestion if available
    let message = if let Some(suggestion) = &error.suggestion {
        format!("{}\n\n💡 {}", error.message, suggestion)
    } else {
        error.message.clone()
    };

    // Convert related information
    let related_information = if error.related.is_empty() {
        None
    } else {
        Some(
            error
                .related
                .iter()
                .map(|info| DiagnosticRelatedInformation {
                    location: Location {
                        uri: uri.clone(),
                        range: Range {
                            start: Position {
                                line: (info.span.start_line.saturating_sub(1)) as u32,
                                character: (info.span.start_col.saturating_sub(1)) as u32,
                            },
                            end: Position {
                                line: (info.span.end_line.saturating_sub(1)) as u32,
                                character: (info.span.end_col.saturating_sub(1)) as u32,
                            },
                        },
                    },
                    message: info.message.clone(),
                })
                .collect(),
        )
    };

    // Check for deprecation warnings
    let tags = if error.message.contains("deprecated") || error.message.contains("Deprecated") {
        Some(vec![DiagnosticTag::DEPRECATED])
    } else if error.message.contains("Unused") || error.message.contains("unused") {
        Some(vec![DiagnosticTag::UNNECESSARY])
    } else {
        None
    };

    Diagnostic {
        range,
        severity: Some(severity),
        code: Some(NumberOrString::String(error_code(&error.message))),
        code_description: None,
        source: Some("orbis-lsp".to_string()),
        message,
        related_information,
        tags,
        data: None,
    }
}

/// Generate an error code from the message
fn error_code(message: &str) -> String {
    if message.contains("Undefined state") {
        "E0001".to_string()
    } else if message.contains("Undefined fragment") {
        "E0002".to_string()
    } else if message.contains("Unknown component") {
        "E0003".to_string()
    } else if message.contains("Unknown attribute") {
        "E0004".to_string()
    } else if message.contains("Unknown event") {
        "E0005".to_string()
    } else if message.contains("Type mismatch") {
        "E0006".to_string()
    } else if message.contains("Missing required") {
        "E0007".to_string()
    } else if message.contains("Unused") {
        "W0001".to_string()
    } else if message.contains("deprecated") || message.contains("Deprecated") {
        "W0002".to_string()
    } else if message.contains("Expected") {
        "E0100".to_string()
    } else {
        "E0000".to_string()
    }
}

/// Format a parse error from pest into a user-friendly message
pub fn format_parse_error(message: &str, source: &str) -> String {
    // Extract the position if available
    if let Some(pos_start) = message.find("at line") {
        let before_pos = &message[..pos_start].trim();
        let pos_info = &message[pos_start..];

        // Try to extract what was expected
        let expected = extract_expected(before_pos);
        let expected_str = expected.clone().unwrap_or_else(|| "Unexpected token".to_string());
        let fix_hint = expected.unwrap_or_default();

        format!(
            "Syntax Error: {}\n{}{}",
            expected_str,
            pos_info,
            suggest_fix_for_parse_error(&fix_hint, source)
        )
    } else {
        message.to_string()
    }
}

fn extract_expected(message: &str) -> Option<String> {
    if message.contains("Expected") {
        Some(message.to_string())
    } else if message.contains("positives:") {
        // Parse pest error format
        if let Some(start) = message.find("[") {
            if let Some(end) = message.find("]") {
                let items = &message[start + 1..end];
                let items: Vec<_> = items.split(',').map(|s| s.trim()).collect();
                return Some(format!("Expected one of: {}", items.join(", ")));
            }
        }
        None
    } else {
        None
    }
}

fn suggest_fix_for_parse_error(expected: &str, _source: &str) -> String {
    let mut suggestions = Vec::new();

    if expected.contains("attribute") || expected.contains("Attribute") {
        suggestions.push("• Check that attribute names are spelled correctly");
        suggestions.push("• Use camelCase for attribute names (e.g., className, onClick)");
    }

    if expected.contains("component") || expected.contains("Component") {
        suggestions.push("• Component names must start with uppercase letter");
        suggestions.push("• Check that you're using a valid Orbis component");
    }

    if expected.contains("expression") || expected.contains("Expression") {
        suggestions.push("• Expressions should be wrapped in { }");
        suggestions.push("• Check for unclosed braces or parentheses");
    }

    if expected.contains("string") || expected.contains("String") {
        suggestions.push("• String values must be wrapped in quotes");
        suggestions.push("• Check for unclosed quotation marks");
    }

    if suggestions.is_empty() {
        String::new()
    } else {
        format!("\n\n💡 Suggestions:\n{}", suggestions.join("\n"))
    }
}
