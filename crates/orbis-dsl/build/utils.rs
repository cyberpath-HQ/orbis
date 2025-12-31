// =============================================================================
// ORBIS DSL BUILD SYSTEM - Utility Functions
// =============================================================================
//
// This module contains string manipulation and helper utilities used throughout
// the build system for generating grammar rules and documentation.
//
// =============================================================================
//
// NOTE: This file is designed to be used with `include!` in build.rs.
// All dependencies must be brought into scope by the including file.
//
// Required imports in build.rs before include:
// - heck::{ToKebabCase, ToLowerCamelCase, ToShoutyKebabCase, ToShoutySnakeCase,
//          ToSnakeCase, ToTrainCase, ToUpperCamelCase}
//
// =============================================================================

/// Generates all case variations for a keyword to support case-insensitive parsing.
///
/// This enables developers to use any casing style they prefer:
/// - `className`, `classname`, `class_name`, `class-name`, `ClassName`
///
/// The Pest parser will match any of these variations.
///
/// # Example
/// ```ignore
/// generate_all_variants("fieldName")
/// // Returns: ["FIELD-NAME", "FIELD_NAME", "Field-Name", "FieldName",
/// //           "field-name", "fieldName", "field_name"]
/// ```
///
/// # Arguments
/// * `keyword` - The canonical keyword name (typically in camelCase)
///
/// # Returns
/// A sorted, deduplicated vector of all case variations
pub fn generate_all_variants(keyword: &str) -> Vec<String> {
    let mut variants = Vec::new();

    // snake_case: field_name
    variants.push(keyword.to_snake_case());
    
    // SCREAMING_SNAKE_CASE: FIELD_NAME
    variants.push(keyword.to_shouty_snake_case());
    
    // camelCase: fieldName
    variants.push(keyword.to_lower_camel_case());
    
    // PascalCase: FieldName
    variants.push(keyword.to_upper_camel_case());
    
    // kebab-case: field-name
    variants.push(keyword.to_kebab_case());
    
    // SCREAMING-KEBAB-CASE: FIELD-NAME
    variants.push(keyword.to_shouty_kebab_case());
    
    // Train-Case: Field-Name
    variants.push(keyword.to_train_case());

    // Sort and remove duplicates (important for single-word keywords)
    variants.sort();
    variants.dedup();
    
    variants
}

/// Replaces content between two markers in a string.
///
/// This is used to update the auto-generated sections of .pest grammar files
/// without touching the hand-written sections. The markers define a region
/// that will be completely replaced with new content.
///
/// # Example
/// ```ignore
/// let content = "// @start\nold content\n// @end\nmanual content";
/// let new_content = "// @start\nnew content\n// @end";
/// let result = replace_section(&content, "// @start", "// @end", &new_content);
/// // result = "new content\nmanual content"
/// ```
///
/// # Arguments
/// * `content` - The original file content
/// * `start_marker` - The marker indicating the start of the replaceable section
/// * `end_marker` - The marker indicating the end of the replaceable section
/// * `replacement` - The new content to insert (should include markers)
///
/// # Returns
/// The modified content with the section replaced, or original content with
/// replacement appended if markers are not found.
pub fn replace_section(
    content: &str,
    start_marker: &str,
    end_marker: &str,
    replacement: &str,
) -> String {
    if let Some(start_pos) = content.find(start_marker) {
        if let Some(end_pos) = content.find(end_marker) {
            // Found both markers - replace the section between them
            let before = &content[..start_pos];
            let after = &content[end_pos + end_marker.len()..];
            return format!("{}{}{}", before, replacement, after);
        }
    }
    
    // Markers not found - append the replacement
    format!("{}\n{}", content, replacement)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_variants_multi_word() {
        let variants = generate_all_variants("fieldName");
        assert!(variants.contains(&"field_name".to_string()));
        assert!(variants.contains(&"FIELD_NAME".to_string()));
        assert!(variants.contains(&"fieldName".to_string()));
        assert!(variants.contains(&"FieldName".to_string()));
        assert!(variants.contains(&"field-name".to_string()));
    }

    #[test]
    fn test_generate_variants_single_word() {
        let variants = generate_all_variants("click");
        // Single word should have fewer unique variants
        assert!(variants.contains(&"click".to_string()));
        assert!(variants.contains(&"CLICK".to_string()));
        assert!(variants.contains(&"Click".to_string()));
    }

    #[test]
    fn test_replace_section() {
        let content = "before\n// @start\nold\n// @end\nafter";
        let result = replace_section(content, "// @start", "// @end", "// @start\nnew\n// @end");
        assert!(result.contains("new"));
        assert!(result.contains("after"));
        assert!(!result.contains("old"));
    }
}
