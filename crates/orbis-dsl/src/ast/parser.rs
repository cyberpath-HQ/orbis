//! Enhanced AST Parser with Import Resolution
//!
//! This module provides a high-level parsing API that:
//! - Parses source files to AST immediately
//! - Resolves and loads imported files
//! - Sorts imports by dependency order (minimum common file first)
//! - Returns a flattened, easily traversable AST
//!
//! # Example
//!
//! ```rust,ignore
//! use orbis_dsl::ast::{OrbisParser, ParseOptions};
//!
//! let parser = OrbisParser::new();
//! let result = parser.parse_file("./page.orbis")?;
//!
//! // Access the main file's AST
//! println!("{:#?}", result.ast);
//!
//! // Access resolved imports with their ASTs
//! for (path, import_ast) in &result.resolved_imports {
//!     println!("Import: {} -> {} elements", path, import_ast.elements.len());
//! }
//! ```

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

use super::builder::{parse_to_ast_with_path, BuildError, BuildErrorKind, BuildResult};
use super::node::{AstFile, ImportStatement, Span};

// ============================================================================
// PARSE OPTIONS
// ============================================================================

/// Options for parsing Orbis DSL files
#[derive(Debug, Clone, Default)]
pub struct ParseOptions {
    /// Whether to resolve and load imports
    pub resolve_imports: bool,
    /// Maximum depth for recursive import resolution (default: 10)
    pub max_import_depth: usize,
    /// Base directory for resolving relative imports
    pub base_dir: Option<PathBuf>,
    /// Whether to fail on missing imports (default: true)
    pub fail_on_missing_import: bool,
}

impl ParseOptions {
    /// Create default options with import resolution enabled
    pub fn with_imports() -> Self {
        Self {
            resolve_imports: true,
            max_import_depth: 10,
            base_dir: None,
            fail_on_missing_import: true,
        }
    }

    /// Create options for single-file parsing (no import resolution)
    pub fn single_file() -> Self {
        Self {
            resolve_imports: false,
            max_import_depth: 0,
            base_dir: None,
            fail_on_missing_import: false,
        }
    }

    /// Set the base directory for import resolution
    pub fn with_base_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.base_dir = Some(dir.into());
        self
    }

    /// Set max import depth
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_import_depth = depth;
        self
    }
}

// ============================================================================
// PARSE RESULT
// ============================================================================

/// Result of parsing an Orbis DSL file with resolved imports
#[derive(Debug, Clone)]
pub struct ParseResult {
    /// The main file's AST
    pub ast: AstFile,
    /// Resolved imports in dependency order (minimum common file first)
    /// Key is the canonical path, value is the parsed AST
    pub resolved_imports: Vec<(String, AstFile)>,
    /// Import graph for debugging/visualization
    pub import_graph: ImportGraph,
    /// Any warnings generated during parsing
    pub warnings: Vec<ParseWarning>,
}

impl ParseResult {
    /// Get the AST for a specific import path
    pub fn get_import(&self, path: &str) -> Option<&AstFile> {
        self.resolved_imports
            .iter()
            .find(|(p, _)| p == path)
            .map(|(_, ast)| ast)
    }

    /// Get all ASTs in dependency order (imports first, main file last)
    pub fn all_asts_ordered(&self) -> Vec<&AstFile> {
        let mut result: Vec<&AstFile> = self.resolved_imports.iter().map(|(_, ast)| ast).collect();
        result.push(&self.ast);
        result
    }

    /// Convert the entire parse result to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

impl serde::Serialize for ParseResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ParseResult", 3)?;
        state.serialize_field("ast", &self.ast)?;
        state.serialize_field("resolved_imports", &self.resolved_imports)?;
        state.serialize_field("warnings", &self.warnings)?;
        state.end()
    }
}

/// Warning generated during parsing (non-fatal issues)
#[derive(Debug, Clone, serde::Serialize)]
pub struct ParseWarning {
    pub message: String,
    pub path: Option<String>,
    pub span: Option<Span>,
}

// ============================================================================
// IMPORT GRAPH
// ============================================================================

/// Graph of import relationships between files
#[derive(Debug, Clone, Default)]
pub struct ImportGraph {
    /// Nodes in the graph (file paths)
    pub nodes: HashSet<String>,
    /// Edges: (from_file, to_file)
    pub edges: Vec<(String, String)>,
}

impl ImportGraph {
    /// Get all files that a given file imports
    pub fn imports_of(&self, file: &str) -> Vec<&str> {
        self.edges
            .iter()
            .filter(|(from, _)| from == file)
            .map(|(_, to)| to.as_str())
            .collect()
    }

    /// Get all files that import a given file
    pub fn importers_of(&self, file: &str) -> Vec<&str> {
        self.edges
            .iter()
            .filter(|(_, to)| to == file)
            .map(|(from, _)| from.as_str())
            .collect()
    }

    /// Topological sort to get files in dependency order
    /// Returns files that are dependencies first (files that are imported by others
    /// but don't import anything themselves come first)
    pub fn topological_sort(&self) -> Vec<String> {
        let mut result = Vec::new();
        let mut out_degree: HashMap<&str, usize> = HashMap::new();
        let mut reverse_adjacency: HashMap<&str, Vec<&str>> = HashMap::new();

        // Initialize
        for node in &self.nodes {
            out_degree.insert(node, 0);
            reverse_adjacency.insert(node, Vec::new());
        }

        // Build reverse graph (pointing from imported to importer)
        // Edge (a, b) means "a imports b", so in reverse graph b -> a
        for (from, to) in &self.edges {
            if let Some(count) = out_degree.get_mut(from.as_str()) {
                *count += 1;
            }
            if let Some(adj) = reverse_adjacency.get_mut(to.as_str()) {
                adj.push(from);
            }
        }

        // Modified Kahn's algorithm on reverse graph
        // Start with nodes that have no outgoing edges (leaf dependencies)
        let mut queue: VecDeque<&str> = out_degree
            .iter()
            .filter(|&(_, &count)| count == 0)
            .map(|(&node, _)| node)
            .collect();

        while let Some(node) = queue.pop_front() {
            result.push(node.to_string());
            if let Some(importers) = reverse_adjacency.get(node) {
                for &importer in importers {
                    if let Some(count) = out_degree.get_mut(importer) {
                        *count -= 1;
                        if *count == 0 {
                            queue.push_back(importer);
                        }
                    }
                }
            }
        }

        result
    }
}

// ============================================================================
// ORBIS PARSER
// ============================================================================

/// High-level parser for Orbis DSL files with import resolution
pub struct OrbisParser {
    options: ParseOptions,
    /// Cache of already-parsed files
    cache: HashMap<PathBuf, AstFile>,
}

impl OrbisParser {
    /// Create a new parser with default options
    pub fn new() -> Self {
        Self {
            options: ParseOptions::default(),
            cache: HashMap::new(),
        }
    }

    /// Create a parser with specific options
    pub fn with_options(options: ParseOptions) -> Self {
        Self {
            options,
            cache: HashMap::new(),
        }
    }

    /// Parse source code directly (no file I/O)
    pub fn parse_source(&mut self, source: &str) -> BuildResult<AstFile> {
        parse_to_ast_with_path(source, "<source>")
    }

    /// Parse source code with a virtual path
    pub fn parse_source_with_path(&mut self, source: &str, path: &str) -> BuildResult<AstFile> {
        parse_to_ast_with_path(source, path)
    }

    /// Parse a file with full import resolution
    pub fn parse_file(&mut self, path: impl AsRef<Path>) -> BuildResult<ParseResult> {
        let path = path.as_ref();
        let canonical = path
            .canonicalize()
            .map_err(|e| BuildError::new(format!("Cannot resolve path: {}", e), None, BuildErrorKind::ParseError))?;

        let base_dir = self
            .options
            .base_dir
            .clone()
            .or_else(|| canonical.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        let source = std::fs::read_to_string(&canonical)
            .map_err(|e| BuildError::new(format!("Cannot read file: {}", e), None, BuildErrorKind::ParseError))?;

        let path_str = canonical.to_string_lossy().to_string();
        let ast = parse_to_ast_with_path(&source, &path_str)?;

        if !self.options.resolve_imports {
            return Ok(ParseResult {
                ast,
                resolved_imports: Vec::new(),
                import_graph: ImportGraph::default(),
                warnings: Vec::new(),
            });
        }

        // Resolve imports
        let mut import_graph = ImportGraph::default();
        let mut resolved_imports: HashMap<String, AstFile> = HashMap::new();
        let mut warnings = Vec::new();

        import_graph.nodes.insert(path_str.clone());

        self.resolve_imports_recursive(
            &ast,
            &path_str,
            &base_dir,
            0,
            &mut import_graph,
            &mut resolved_imports,
            &mut warnings,
        )?;

        // Sort imports by dependency order
        let sorted_paths = import_graph.topological_sort();
        let resolved_imports: Vec<(String, AstFile)> = sorted_paths
            .into_iter()
            .filter(|p| p != &path_str) // Exclude main file
            .filter_map(|p| resolved_imports.remove(&p).map(|ast| (p, ast)))
            .collect();

        Ok(ParseResult {
            ast,
            resolved_imports,
            import_graph,
            warnings,
        })
    }

    fn resolve_imports_recursive(
        &mut self,
        ast: &AstFile,
        current_path: &str,
        base_dir: &Path,
        depth: usize,
        graph: &mut ImportGraph,
        resolved: &mut HashMap<String, AstFile>,
        warnings: &mut Vec<ParseWarning>,
    ) -> BuildResult<()> {
        if depth > self.options.max_import_depth {
            return Err(BuildError::new(
                format!("Maximum import depth ({}) exceeded", self.options.max_import_depth),
                None,
                BuildErrorKind::ParseError,
            ));
        }

        for import in &ast.imports {
            let import_path = match import {
                ImportStatement::TypeScript { path, .. } => path.clone(),
                ImportStatement::Rust { path, .. } => path.join("::"),
            };

            // Resolve relative path
            let resolved_path = self.resolve_import_path(&import_path, current_path, base_dir);

            match resolved_path {
                Ok(resolved_path_str) => {
                    // Add edge to graph
                    graph.edges.push((current_path.to_string(), resolved_path_str.clone()));
                    graph.nodes.insert(resolved_path_str.clone());

                    // Skip if already resolved
                    if resolved.contains_key(&resolved_path_str) {
                        continue;
                    }

                    // Try to load and parse the import
                    match self.load_and_parse_import(&resolved_path_str) {
                        Ok(import_ast) => {
                            // Recursively resolve imports of the import
                            let import_dir = Path::new(&resolved_path_str)
                                .parent()
                                .map(|p| p.to_path_buf())
                                .unwrap_or_else(|| base_dir.to_path_buf());

                            self.resolve_imports_recursive(
                                &import_ast,
                                &resolved_path_str,
                                &import_dir,
                                depth + 1,
                                graph,
                                resolved,
                                warnings,
                            )?;

                            resolved.insert(resolved_path_str, import_ast);
                        }
                        Err(e) => {
                            if self.options.fail_on_missing_import {
                                return Err(e);
                            } else {
                                warnings.push(ParseWarning {
                                    message: format!("Could not resolve import '{}': {}", import_path, e),
                                    path: Some(current_path.to_string()),
                                    span: import.span().cloned(),
                                });
                            }
                        }
                    }
                }
                Err(e) => {
                    if self.options.fail_on_missing_import {
                        return Err(e);
                    } else {
                        warnings.push(ParseWarning {
                            message: format!("Could not resolve import path '{}': {}", import_path, e),
                            path: Some(current_path.to_string()),
                            span: import.span().cloned(),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    fn resolve_import_path(
        &self,
        import_path: &str,
        current_file: &str,
        base_dir: &Path,
    ) -> BuildResult<String> {
        // Handle relative imports
        if import_path.starts_with("./") || import_path.starts_with("../") {
            let current_dir = Path::new(current_file)
                .parent()
                .unwrap_or(base_dir);

            let resolved = current_dir.join(import_path);

            // Try with .orbis extension if not present
            let resolved = if resolved.extension().is_none() {
                resolved.with_extension("orbis")
            } else {
                resolved
            };

            resolved
                .canonicalize()
                .map(|p| p.to_string_lossy().to_string())
                .map_err(|e| {
                    BuildError::new(
                        format!("Cannot resolve import '{}': {}", import_path, e),
                        None,
                        BuildErrorKind::ParseError,
                    )
                })
        } else {
            // Absolute or package import - for now just return as-is
            // In a real implementation, this would resolve to node_modules or a package registry
            Ok(import_path.to_string())
        }
    }

    fn load_and_parse_import(&mut self, path: &str) -> BuildResult<AstFile> {
        let path_buf = PathBuf::from(path);

        // Check cache
        if let Some(ast) = self.cache.get(&path_buf) {
            return Ok(ast.clone());
        }

        // Load and parse
        let source = std::fs::read_to_string(&path_buf).map_err(|e| {
            BuildError::new(
                format!("Cannot read import '{}': {}", path, e),
                None,
                BuildErrorKind::ParseError,
            )
        })?;

        let ast = parse_to_ast_with_path(&source, path)?;

        // Cache
        self.cache.insert(path_buf, ast.clone());

        Ok(ast)
    }

    /// Clear the parse cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl Default for OrbisParser {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// HELPER TRAIT FOR IMPORT STATEMENT
// ============================================================================

impl ImportStatement {
    /// Get the span of the import statement
    pub fn span(&self) -> Option<&Span> {
        match self {
            ImportStatement::TypeScript { span, .. } => Some(span),
            ImportStatement::Rust { span, .. } => Some(span),
        }
    }

    /// Get the import path
    pub fn path(&self) -> String {
        match self {
            ImportStatement::TypeScript { path, .. } => path.clone(),
            ImportStatement::Rust { path, .. } => path.join("::"),
        }
    }
}

// ============================================================================
// CONVENIENCE FUNCTIONS
// ============================================================================

/// Parse source code directly to AST (single file, no imports)
pub fn parse(source: &str) -> BuildResult<AstFile> {
    let mut parser = OrbisParser::with_options(ParseOptions::single_file());
    parser.parse_source(source)
}

/// Parse source code with a path identifier
pub fn parse_with_path(source: &str, path: &str) -> BuildResult<AstFile> {
    let mut parser = OrbisParser::with_options(ParseOptions::single_file());
    parser.parse_source_with_path(source, path)
}

/// Parse a file with full import resolution
pub fn parse_file(path: impl AsRef<Path>) -> BuildResult<ParseResult> {
    let mut parser = OrbisParser::with_options(ParseOptions::with_imports());
    parser.parse_file(path)
}

/// Parse a file without resolving imports
pub fn parse_file_only(path: impl AsRef<Path>) -> BuildResult<ParseResult> {
    let mut parser = OrbisParser::with_options(ParseOptions::single_file());
    parser.parse_file(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_source() {
        let source = r#"
page {
    id: "test-page"
    title: "Test Page"
}
"#;
        let ast = parse(source).unwrap();
        assert!(ast.elements.len() >= 1);
    }

    #[test]
    fn test_parse_with_path() {
        let source = r#"
page { id: "test" }
"#;
        let ast = parse_with_path(source, "/virtual/test.orbis").unwrap();
        assert_eq!(ast.path, Some("/virtual/test.orbis".to_string()));
    }

    #[test]
    fn test_import_graph_topological_sort() {
        let mut graph = ImportGraph::default();
        graph.nodes.insert("a.orbis".to_string());
        graph.nodes.insert("b.orbis".to_string());
        graph.nodes.insert("c.orbis".to_string());
        graph.edges.push(("a.orbis".to_string(), "b.orbis".to_string()));
        graph.edges.push(("a.orbis".to_string(), "c.orbis".to_string()));
        graph.edges.push(("b.orbis".to_string(), "c.orbis".to_string()));

        let sorted = graph.topological_sort();
        // c should come before b, b should come before a
        let c_idx = sorted.iter().position(|x| x == "c.orbis").unwrap();
        let b_idx = sorted.iter().position(|x| x == "b.orbis").unwrap();
        let a_idx = sorted.iter().position(|x| x == "a.orbis").unwrap();

        assert!(c_idx < b_idx);
        assert!(b_idx < a_idx);
    }
}
