//! Document Management
//!
//! This module handles document state, content updates, and rope-based
//! text manipulation for efficient incremental updates.

use ropey::Rope;
use tower_lsp::lsp_types::{Position, Range, TextDocumentContentChangeEvent, Url};

/// A document being tracked by the language server
#[derive(Debug, Clone)]
pub struct Document {
    /// Document URI
    pub uri: Url,
    /// Document content as a rope for efficient manipulation
    pub content: Rope,
    /// Document version for change tracking
    pub version: i32,
    /// Language ID (should be "orbis")
    pub language_id: String,
}

impl Document {
    /// Create a new document from content
    pub fn new(uri: Url, content: String, version: i32, language_id: String) -> Self {
        Self {
            uri,
            content: Rope::from_str(&content),
            version,
            language_id,
        }
    }

    /// Get document content as a string
    pub fn text(&self) -> String {
        self.content.to_string()
    }

    /// Get a specific line (0-indexed)
    pub fn get_line(&self, line: usize) -> Option<String> {
        if line >= self.content.len_lines() {
            None
        } else {
            Some(self.content.line(line).to_string())
        }
    }

    /// Get number of lines
    pub fn line_count(&self) -> usize {
        self.content.len_lines()
    }

    /// Apply incremental changes to the document
    pub fn apply_changes(&mut self, changes: Vec<TextDocumentContentChangeEvent>, version: i32) {
        for change in changes {
            if let Some(range) = change.range {
                // Incremental update
                let start_idx = self.position_to_offset(&range.start);
                let end_idx = self.position_to_offset(&range.end);

                if let (Some(start), Some(end)) = (start_idx, end_idx) {
                    self.content.remove(start..end);
                    self.content.insert(start, &change.text);
                }
            } else {
                // Full content replacement
                self.content = Rope::from_str(&change.text);
            }
        }
        self.version = version;
    }

    /// Convert LSP Position to byte offset
    pub fn position_to_offset(&self, pos: &Position) -> Option<usize> {
        let line = pos.line as usize;
        if line >= self.content.len_lines() {
            return None;
        }

        let line_start = self.content.line_to_char(line);
        let col = pos.character as usize;

        // Handle UTF-16 code units (LSP uses UTF-16)
        let line_text = self.content.line(line);
        let mut char_offset = 0;
        let mut utf16_offset = 0;

        for ch in line_text.chars() {
            if utf16_offset >= col {
                break;
            }
            char_offset += 1;
            utf16_offset += ch.len_utf16();
        }

        Some(line_start + char_offset)
    }

    /// Convert byte offset to LSP Position
    pub fn offset_to_position(&self, offset: usize) -> Position {
        let offset = offset.min(self.content.len_chars());
        let line = self.content.char_to_line(offset);
        let line_start = self.content.line_to_char(line);

        // Convert char offset to UTF-16 code units
        let mut utf16_col = 0;
        for ch in self.content.slice(line_start..offset).chars() {
            utf16_col += ch.len_utf16();
        }

        Position {
            line: line as u32,
            character: utf16_col as u32,
        }
    }

    /// Get the word at a given position
    pub fn word_at_position(&self, pos: &Position) -> Option<(String, Range)> {
        let offset = self.position_to_offset(pos)?;
        let text = self.text();
        let bytes = text.as_bytes();

        if offset >= bytes.len() {
            return None;
        }

        // Find word boundaries (including dots for method calls)
        let mut start = offset;
        let mut end = offset;

        // Move start backwards to word boundary
        while start > 0 {
            let ch = text.chars().nth(start - 1)?;
            if !is_word_char(ch) && ch != '.' {
                break;
            }
            start -= 1;
        }

        // Move end forwards to word boundary
        while end < text.len() {
            let ch = text.chars().nth(end)?;
            if !is_word_char(ch) && ch != '.' {
                break;
            }
            end += 1;
        }

        if start == end {
            return None;
        }

        let word = text[start..end].to_string();
        let start_pos = self.offset_to_position(start);
        let end_pos = self.offset_to_position(end);

        Some((
            word,
            Range {
                start: start_pos,
                end: end_pos,
            },
        ))
    }

    /// Check if a position is inside a string literal
    /// This is a simple heuristic that counts quotes before the position
    pub fn is_in_string(&self, pos: &Position) -> bool {
        let line = pos.line as usize;
        let col = pos.character as usize;
        
        let line_text = self.get_line(line).unwrap_or_default();
        if col >= line_text.len() {
            return false;
        }
        
        // Count unescaped quotes before position
        let prefix = &line_text[..col];
        let mut in_string = false;
        let mut prev_char = ' ';
        
        for ch in prefix.chars() {
            if ch == '"' && prev_char != '\\' {
                in_string = !in_string;
            }
            prev_char = ch;
        }
        
        in_string
    }

    /// Get the context at a given position (for completion)
    pub fn get_context(&self, pos: &Position) -> DocumentContext {
        let line = pos.line as usize;
        let col = pos.character as usize;

        let line_text = self.get_line(line).unwrap_or_default();

        // Get text before cursor on current line
        let prefix = if col <= line_text.len() {
            line_text[..col.min(line_text.len())].to_string()
        } else {
            line_text.clone()
        };

        // Determine context type
        let context_type = determine_context_type(&prefix, &line_text);

        // Get the current word being typed
        let trigger_word = get_current_word(&prefix);

        DocumentContext {
            line: line_text,
            prefix,
            trigger_word,
            context_type,
        }
    }
}

/// Context information for a cursor position
#[derive(Debug, Clone)]
pub struct DocumentContext {
    /// Full line text
    pub line: String,
    /// Text before cursor
    pub prefix: String,
    /// Word currently being typed
    pub trigger_word: String,
    /// Type of context
    pub context_type: ContextType,
}

/// Type of context for completion
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextType {
    /// At top level (page, state, template, etc.)
    TopLevel,
    /// Inside page block
    PageBlock,
    /// Inside state block
    StateBlock,
    /// Inside hooks block
    HooksBlock,
    /// Inside template block
    Template,
    /// Inside a component tag, expecting attribute
    ComponentAttribute { component: String },
    /// Inside a component tag, expecting event
    ComponentEvent { component: String },
    /// Inside expression braces
    Expression,
    /// Inside action body
    ActionBody,
    /// Inside import statement
    Import,
    /// Inside type annotation
    TypeAnnotation,
    /// Inside styles block
    Styles,
    /// Unknown context
    Unknown,
}

fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '$' || ch == '@'
}

fn get_current_word(prefix: &str) -> String {
    let mut word = String::new();
    for ch in prefix.chars().rev() {
        if is_word_char(ch) || ch == '.' {
            word.insert(0, ch);
        } else {
            break;
        }
    }
    word
}

fn determine_context_type(prefix: &str, _full_line: &str) -> ContextType {
    let trimmed = prefix.trim();

    // Check for import context
    if trimmed.starts_with("import ") || trimmed.starts_with("use ") {
        return ContextType::Import;
    }

    // Check if inside expression
    let open_braces = prefix.matches('{').count();
    let close_braces = prefix.matches('}').count();
    if open_braces > close_braces {
        // Check if it's an action body
        if prefix.contains("=>") {
            return ContextType::ActionBody;
        }
        return ContextType::Expression;
    }

    // Check if we're in a component tag
    if let Some(tag_start) = prefix.rfind('<') {
        let after_tag = &prefix[tag_start..];
        // Check if tag is still open (no >)
        if !after_tag.contains('>') || after_tag.rfind('<') > after_tag.rfind('>') {
            // Extract component name
            let tag_content = after_tag.trim_start_matches('<');
            let component = tag_content
                .split(|c: char| c.is_whitespace() || c == '/' || c == '>')
                .next()
                .unwrap_or("")
                .to_string();

            if !component.is_empty() {
                // Check if we're after @ (event)
                if prefix.ends_with('@') || prefix.contains(" @") {
                    return ContextType::ComponentEvent { component };
                }
                // Otherwise expect attribute
                return ContextType::ComponentAttribute { component };
            }
        }
    }

    // Check block contexts by looking for block keywords
    let text_before = prefix.to_lowercase();
    if text_before.contains("template") && open_braces > 0 {
        return ContextType::Template;
    }
    if text_before.contains("state") && open_braces > 0 {
        return ContextType::StateBlock;
    }
    if text_before.contains("hooks") && open_braces > 0 {
        return ContextType::HooksBlock;
    }
    if text_before.contains("page") && open_braces > 0 {
        return ContextType::PageBlock;
    }
    if text_before.contains("styles") && open_braces > 0 {
        return ContextType::Styles;
    }

    // Check for type annotation context
    if prefix.ends_with(':') || prefix.contains(": ") {
        return ContextType::TypeAnnotation;
    }

    // Default to top level
    ContextType::TopLevel
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_text() {
        let doc = Document::new(
            Url::parse("file:///test.orbis").unwrap(),
            "hello\nworld".to_string(),
            1,
            "orbis".to_string(),
        );
        assert_eq!(doc.text(), "hello\nworld");
    }

    #[test]
    fn test_document_lines() {
        let doc = Document::new(
            Url::parse("file:///test.orbis").unwrap(),
            "line1\nline2\nline3".to_string(),
            1,
            "orbis".to_string(),
        );
        assert_eq!(doc.line_count(), 3);
        assert_eq!(doc.get_line(0), Some("line1\n".to_string()));
        assert_eq!(doc.get_line(1), Some("line2\n".to_string()));
        assert_eq!(doc.get_line(2), Some("line3".to_string()));
    }

    #[test]
    fn test_position_conversion() {
        let doc = Document::new(
            Url::parse("file:///test.orbis").unwrap(),
            "hello\nworld".to_string(),
            1,
            "orbis".to_string(),
        );

        let offset = doc.position_to_offset(&Position {
            line: 1,
            character: 2,
        });
        assert_eq!(offset, Some(8)); // "hello\nwo"

        let pos = doc.offset_to_position(8);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.character, 2);
    }
}
