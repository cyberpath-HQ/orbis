//! Semantic Tokens Provider
//!
//! This module provides semantic token highlighting for the Orbis DSL.
//! Semantic tokens provide rich syntax highlighting that goes beyond
//! TextMate grammars by using semantic information from the parser.

use tower_lsp::lsp_types::{
    SemanticToken, SemanticTokens, SemanticTokensResult,
};

use crate::analysis::AnalysisResult;
use orbis_dsl::ast::{
    AstFile, TopLevelElement, TemplateContent, ControlFlow, Expression,
    StateDeclaration, ActionItem, Action, Span, HookEntry,
};

/// Semantic token types as indices into the legend
mod token_types {
    pub const NAMESPACE: u32 = 0;
    pub const TYPE: u32 = 1;
    pub const CLASS: u32 = 2;
    pub const FUNCTION: u32 = 3;
    pub const PARAMETER: u32 = 4;
    pub const VARIABLE: u32 = 5;
    pub const PROPERTY: u32 = 6;
    pub const KEYWORD: u32 = 7;
    pub const STRING: u32 = 8;
    pub const NUMBER: u32 = 9;
    pub const OPERATOR: u32 = 10;
    pub const COMMENT: u32 = 11;
    pub const DECORATOR: u32 = 12;
    pub const EVENT: u32 = 13;
    #[allow(dead_code)]
    pub const MACRO: u32 = 14;
}

/// Semantic token modifiers as bit flags
mod token_modifiers {
    pub const DECLARATION: u32 = 1 << 0;
    pub const DEFINITION: u32 = 1 << 1;
    pub const READONLY: u32 = 1 << 2;
    #[allow(dead_code)]
    pub const DEPRECATED: u32 = 1 << 3;
    pub const MODIFICATION: u32 = 1 << 4;
    #[allow(dead_code)]
    pub const DEFAULT_LIBRARY: u32 = 1 << 5;
    #[allow(dead_code)]
    pub const ASYNC: u32 = 1 << 6;
}

/// Token builder for creating semantic tokens
pub struct SemanticTokenBuilder {
    /// Unsorted tokens with absolute positions
    tokens: Vec<(u32, u32, u32, u32, u32)>, // (line, start, length, type, modifiers)
    prev_line: u32,
    prev_start: u32,
}

impl SemanticTokenBuilder {
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            prev_line: 0,
            prev_start: 0,
        }
    }

    /// Push a semantic token with absolute position (before sorting)
    pub fn push(&mut self, line: u32, start: u32, length: u32, token_type: u32, modifiers: u32) {
        // Store absolute position, we'll sort and convert to deltas later
        self.tokens.push((line, start, length, token_type, modifiers));
    }

    /// Build the final semantic tokens result
    pub fn build(mut self) -> SemanticTokensResult {
        // Sort tokens by line, then by start column
        self.tokens.sort_by_key(|t| (t.0, t.1));

        // Convert to semantic token deltas
        let mut semantic_tokens = Vec::new();
        let mut prev_line = 0u32;
        let mut prev_start = 0u32;

        for (line, start, length, token_type, modifiers) in self.tokens {
            let delta_line = line.saturating_sub(prev_line);
            let delta_start = if delta_line == 0 {
                start.saturating_sub(prev_start)
            } else {
                start
            };

            semantic_tokens.push(SemanticToken {
                delta_line,
                delta_start,
                length,
                token_type,
                token_modifiers_bitset: modifiers,
            });

            prev_line = line;
            prev_start = start;
        }

        SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: semantic_tokens,
        })
    }
}

impl Default for SemanticTokenBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Get semantic tokens for the entire document
pub fn get_semantic_tokens(result: &AnalysisResult) -> Option<SemanticTokensResult> {
    let ast = result.ast.as_ref()?;
    let mut builder = SemanticTokenBuilder::new();

    visit_ast(ast, &mut builder);

    Some(builder.build())
}

/// Visit AST and emit semantic tokens
fn visit_ast(ast: &AstFile, builder: &mut SemanticTokenBuilder) {
    // Process imports
    for import in &ast.imports {
        visit_import(import, builder);
    }

    // Process top-level elements
    for element in &ast.elements {
        visit_element(element, builder);
    }
}

/// Visit import statement
fn visit_import(import: &orbis_dsl::ast::ImportStatement, builder: &mut SemanticTokenBuilder) {
    match import {
        orbis_dsl::ast::ImportStatement::TypeScript { clause, span, .. } => {
            // "import" keyword
            emit_token(builder, span, 0, 6, token_types::KEYWORD, 0);

            // Import clause
            match clause {
                orbis_dsl::ast::ImportClause::Named { specifiers } => {
                    for spec in specifiers {
                        emit_token(builder, &spec.span, 0, spec.name.len() as u32, token_types::TYPE, 0);
                    }
                }
                orbis_dsl::ast::ImportClause::Default { name } => {
                    emit_token(builder, span, 7, name.len() as u32, token_types::TYPE, 0);
                }
                orbis_dsl::ast::ImportClause::Namespace { name } => {
                    emit_token(builder, span, 7, name.len() as u32, token_types::NAMESPACE, 0);
                }
            }
        }
        orbis_dsl::ast::ImportStatement::Rust { path, span, .. } => {
            // "use" keyword
            emit_token(builder, span, 0, 3, token_types::KEYWORD, 0);

            // Path segments
            for segment in path {
                emit_token(builder, span, 0, segment.len() as u32, token_types::NAMESPACE, 0);
            }
        }
    }
}

/// Visit top-level element
fn visit_element(element: &TopLevelElement, builder: &mut SemanticTokenBuilder) {
    match element {
        TopLevelElement::Page(page) => {
            // "page" keyword
            emit_token(builder, &page.span, 0, 4, token_types::KEYWORD, token_modifiers::DEFINITION);
        }

        TopLevelElement::State(state) => {
            // "state" keyword
            emit_token(builder, &state.span, 0, 5, token_types::KEYWORD, token_modifiers::DEFINITION);

            for decl in &state.declarations {
                visit_state_declaration(decl, builder);
            }
        }

        TopLevelElement::Hooks(hooks) => {
            // "hooks" keyword
            emit_token(builder, &hooks.span, 0, 5, token_types::KEYWORD, token_modifiers::DEFINITION);

            for entry in &hooks.entries {
                match entry {
                    HookEntry::Lifecycle(hook) => {
                        let decorator = match hook.kind {
                            orbis_dsl::ast::LifecycleHookKind::Mount => "@mount",
                            orbis_dsl::ast::LifecycleHookKind::Unmount => "@unmount",
                        };
                        let line = (hook.span.start_line.saturating_sub(1)) as u32;
                        let col = (hook.span.start_col.saturating_sub(1)) as u32;
                        builder.push(line, col, decorator.len() as u32, token_types::DECORATOR, 0);
                        
                        // Highlight the => arrow (it's after the decorator, typically with space)
                        // Approximate position: decorator length + 1 space
                        builder.push(line, col + decorator.len() as u32 + 1, 2, token_types::OPERATOR, 0);

                        for action in &hook.actions {
                            visit_action_item(action, builder);
                        }
                    }
                    HookEntry::Watcher(watcher) => {
                        // @watch decorator
                        let line = (watcher.span.start_line.saturating_sub(1)) as u32;
                        let col = (watcher.span.start_col.saturating_sub(1)) as u32;
                        builder.push(line, col, 6, token_types::DECORATOR, 0); // @watch
                        
                        // Note: => arrow position is harder to determine for watchers because of variable argument lists
                        // We'll skip precise arrow highlighting for watchers for now

                        for target in &watcher.targets {
                            visit_expression(target, builder);
                        }

                        for action in &watcher.actions {
                            visit_action_item(action, builder);
                        }
                    }
                }
            }
        }

        TopLevelElement::Template(template) => {
            // "template" keyword
            emit_token(builder, &template.span, 0, 8, token_types::KEYWORD, token_modifiers::DEFINITION);

            for content in &template.content {
                visit_template_content(content, builder);
            }
        }

        TopLevelElement::Fragment(frag) => {
            // "fragment" keyword
            emit_token(builder, &frag.span, 0, 8, token_types::KEYWORD, token_modifiers::DEFINITION);

            // Fragment name
            emit_token(
                builder,
                &frag.span,
                9,
                frag.name.len() as u32,
                token_types::FUNCTION,
                token_modifiers::DECLARATION,
            );

            // Parameters
            for param in &frag.params {
                emit_token(
                    builder,
                    &param.span,
                    0,
                    param.name.len() as u32,
                    token_types::PARAMETER,
                    token_modifiers::DECLARATION,
                );
            }

            for content in &frag.body {
                visit_template_content(content, builder);
            }
        }

        TopLevelElement::Interface(iface) => {
            // "interface" keyword
            emit_token(builder, &iface.span, 0, 9, token_types::KEYWORD, token_modifiers::DEFINITION);

            // Interface name
            emit_token(
                builder,
                &iface.span,
                10,
                iface.name.len() as u32,
                token_types::TYPE,
                token_modifiers::DECLARATION,
            );

            for member in &iface.members {
                let line = (member.span.start_line.saturating_sub(1)) as u32;
                builder.push(line, 0, member.name.len() as u32, token_types::PROPERTY, 0);
            }
        }

        TopLevelElement::Styles(styles) => {
            // "styles" keyword
            emit_token(builder, &styles.span, 0, 6, token_types::KEYWORD, token_modifiers::DEFINITION);
        }

        TopLevelElement::Export(export) => {
            // "export" keyword
            emit_token(builder, &export.span, 0, 6, token_types::KEYWORD, 0);

            match &export.item {
                orbis_dsl::ast::ExportableItem::Fragment(frag) => {
                    emit_token(
                        builder,
                        &export.span,
                        7,
                        frag.name.len() as u32,
                        token_types::FUNCTION,
                        token_modifiers::DECLARATION,
                    );
                }
                orbis_dsl::ast::ExportableItem::Interface(iface) => {
                    emit_token(
                        builder,
                        &export.span,
                        7,
                        iface.name.len() as u32,
                        token_types::TYPE,
                        token_modifiers::DECLARATION,
                    );
                }
                orbis_dsl::ast::ExportableItem::Const { name, .. } => {
                    emit_token(
                        builder,
                        &export.span,
                        7,
                        name.len() as u32,
                        token_types::VARIABLE,
                        token_modifiers::READONLY,
                    );
                }
            }
        }

        TopLevelElement::Comment { value, span } => {
            let line = (span.start_line.saturating_sub(1)) as u32;
            let col = (span.start_col.saturating_sub(1)) as u32;
            let length = value.len() as u32 + 4; // include // or /* */
            builder.push(line, col, length, token_types::COMMENT, 0);
        }
    }
}

/// Visit state declaration
fn visit_state_declaration(decl: &StateDeclaration, builder: &mut SemanticTokenBuilder) {
    match decl {
        StateDeclaration::Regular(reg) => {
            let line = (reg.span.start_line.saturating_sub(1)) as u32;
            let col = (reg.span.start_col.saturating_sub(1)) as u32;
            
            // Variable name
            builder.push(
                line,
                col,
                reg.name.len() as u32,
                token_types::VARIABLE,
                token_modifiers::DECLARATION,
            );

            // Type annotation if present
            if let Some(type_ann) = &reg.type_annotation {
                // Highlight the type annotation span
                let type_span = type_ann.span();
                let type_line = (type_span.start_line.saturating_sub(1)) as u32;
                let type_col = (type_span.start_col.saturating_sub(1)) as u32;
                let type_len = (type_span.end_col - type_span.start_col) as u32;
                builder.push(
                    type_line,
                    type_col,
                    type_len,
                    token_types::TYPE,
                    0,
                );
            }

            if let Some(value) = &reg.value {
                visit_expression(value, builder);
            }
        }

        StateDeclaration::Computed(comp) => {
            let line = (comp.span.start_line.saturating_sub(1)) as u32;

            if comp.explicit_computed {
                // @computed decorator
                builder.push(line, 0, 9, token_types::DECORATOR, 0);
            }

            // Variable name
            builder.push(
                line,
                if comp.explicit_computed { 10 } else { 0 },
                comp.name.len() as u32,
                token_types::VARIABLE,
                token_modifiers::DECLARATION | token_modifiers::READONLY,
            );

            visit_expression(&comp.expression, builder);
        }

        StateDeclaration::Validated(val) => {
            let line = (val.span.start_line.saturating_sub(1)) as u32;

            // Variable name
            builder.push(
                line,
                0,
                val.name.len() as u32,
                token_types::VARIABLE,
                token_modifiers::DECLARATION,
            );

            // Validators as decorators
            for validator in &val.validators {
                let v_line = (validator.span.start_line.saturating_sub(1)) as u32;
                builder.push(v_line, 0, (validator.name.len() + 1) as u32, token_types::DECORATOR, 0);
            }

            if let Some(value) = &val.value {
                visit_expression(value, builder);
            }
        }
    }
}

/// Visit template content
fn visit_template_content(content: &TemplateContent, builder: &mut SemanticTokenBuilder) {
    match content {
        TemplateContent::Component(comp) => {
            // Component name
            let line = (comp.span.start_line.saturating_sub(1)) as u32;
            builder.push(line, 1, comp.name.len() as u32, token_types::CLASS, 0); // +1 for <

            // Attributes
            for attr in &comp.attributes {
                let attr_line = (attr.span.start_line.saturating_sub(1)) as u32;
                let attr_col = (attr.span.start_col.saturating_sub(1)) as u32;
                builder.push(attr_line, attr_col, attr.name.len() as u32, token_types::PROPERTY, 0);

                match &attr.value {
                    orbis_dsl::ast::AttributeValue::String { .. } => {
                        // String literal
                    }
                    orbis_dsl::ast::AttributeValue::Expression { value } => {
                        visit_expression(value, builder);
                    }
                }
            }

            // Events
            for event in &comp.events {
                let event_line = (event.span.start_line.saturating_sub(1)) as u32;
                let event_col = (event.span.start_col.saturating_sub(1)) as u32;
                // Event names start with @ (included in length)
                builder.push(event_line, event_col, event.event.len() as u32 + 1, token_types::EVENT, 0);
            }

            // Children
            for child in &comp.children {
                visit_template_content(child, builder);
            }
        }

        TemplateContent::FragmentUsage(frag) => {
            let line = (frag.span.start_line.saturating_sub(1)) as u32;
            builder.push(line, 1, frag.name.len() as u32, token_types::FUNCTION, 0);

            for prop in &frag.properties {
                let prop_line = (prop.span.start_line.saturating_sub(1)) as u32;
                let prop_col = (prop.span.start_col.saturating_sub(1)) as u32;
                builder.push(prop_line, prop_col, prop.name.len() as u32, token_types::PARAMETER, 0);
            }
        }

        TemplateContent::ControlFlow(cf) => {
            visit_control_flow(cf, builder);
        }

        TemplateContent::SlotDefinition(slot) => {
            let line = (slot.span.start_line.saturating_sub(1)) as u32;
            builder.push(line, 0, 4, token_types::KEYWORD, 0); // "slot"
        }

        TemplateContent::Text { value, span } => {
            let line = (span.start_line.saturating_sub(1)) as u32;
            let col = (span.start_col.saturating_sub(1)) as u32;
            builder.push(line, col, value.len() as u32, token_types::STRING, 0);
        }

        TemplateContent::Expression { expr, .. } => {
            visit_expression(expr, builder);
        }

        TemplateContent::Comment { value, span } => {
            let line = (span.start_line.saturating_sub(1)) as u32;
            let col = (span.start_col.saturating_sub(1)) as u32;
            builder.push(line, col, value.len() as u32 + 4, token_types::COMMENT, 0);
        }
    }
}

/// Visit control flow
fn visit_control_flow(cf: &ControlFlow, builder: &mut SemanticTokenBuilder) {
    match cf {
        ControlFlow::If(if_block) => {
            let line = (if_block.span.start_line.saturating_sub(1)) as u32;
            builder.push(line, 0, 2, token_types::KEYWORD, 0); // "if"

            visit_expression(&if_block.condition, builder);

            for content in &if_block.then_branch {
                visit_template_content(content, builder);
            }

            for else_if in &if_block.else_if_branches {
                let else_if_line = (else_if.span.start_line.saturating_sub(1)) as u32;
                builder.push(else_if_line, 0, 7, token_types::KEYWORD, 0); // "else if"
                visit_expression(&else_if.condition, builder);
                for content in &else_if.body {
                    visit_template_content(content, builder);
                }
            }

            if let Some(else_branch) = &if_block.else_branch {
                builder.push(line, 0, 4, token_types::KEYWORD, 0); // "else"
                for content in else_branch {
                    visit_template_content(content, builder);
                }
            }
        }

        ControlFlow::For(for_block) => {
            let line = (for_block.span.start_line.saturating_sub(1)) as u32;
            builder.push(line, 0, 3, token_types::KEYWORD, 0); // "for"

            // Binding
            let bind_line = (for_block.binding.span.start_line.saturating_sub(1)) as u32;
            builder.push(
                bind_line,
                4,
                for_block.binding.item.name.len() as u32,
                token_types::VARIABLE,
                token_modifiers::DECLARATION,
            );

            if let Some(index) = &for_block.binding.index {
                builder.push(
                    bind_line,
                    4 + for_block.binding.item.name.len() as u32 + 2,
                    index.name.len() as u32,
                    token_types::VARIABLE,
                    token_modifiers::DECLARATION,
                );
            }

            // "in" keyword
            builder.push(line, 0, 2, token_types::KEYWORD, 0);

            visit_expression(&for_block.iterable, builder);

            for content in &for_block.body {
                visit_template_content(content, builder);
            }
        }

        ControlFlow::When(when_block) => {
            let line = (when_block.span.start_line.saturating_sub(1)) as u32;
            builder.push(line, 0, 4, token_types::KEYWORD, 0); // "when"

            visit_expression(&when_block.subject, builder);

            for arm in &when_block.arms {
                let arm_line = (arm.span.start_line.saturating_sub(1)) as u32;

                match &arm.pattern {
                    orbis_dsl::ast::WhenPattern::Literal { value, .. } => {
                        visit_expression(value, builder);
                    }
                    orbis_dsl::ast::WhenPattern::Binding { name, span } => {
                        let bind_line = (span.start_line.saturating_sub(1)) as u32;
                        builder.push(bind_line, 0, name.len() as u32, token_types::VARIABLE, token_modifiers::DECLARATION);
                    }
                    orbis_dsl::ast::WhenPattern::Range { start, end, .. } => {
                        visit_expression(start, builder);
                        visit_expression(end, builder);
                    }
                    orbis_dsl::ast::WhenPattern::Or { patterns, .. } => {
                        // Sub-patterns
                        for pattern in patterns {
                            if let orbis_dsl::ast::WhenPattern::Literal { value, .. } = pattern {
                                visit_expression(value, builder);
                            }
                        }
                    }
                    orbis_dsl::ast::WhenPattern::Wildcard { .. } => {
                        builder.push(arm_line, 0, 1, token_types::OPERATOR, 0); // "_"
                    }
                }

                for content in &arm.body {
                    visit_template_content(content, builder);
                }
            }
        }
    }
}

/// Visit expression
fn visit_expression(expr: &Expression, builder: &mut SemanticTokenBuilder) {
    match expr {
        Expression::Identifier(id) => {
            let line = (id.span.start_line.saturating_sub(1)) as u32;
            let col = (id.span.start_col.saturating_sub(1)) as u32;
            builder.push(line, col, id.name.len() as u32, token_types::VARIABLE, 0);
        }

        Expression::MemberAccess(ma) => {
            let line = (ma.span.start_line.saturating_sub(1)) as u32;
            let col = (ma.span.start_col.saturating_sub(1)) as u32;
            builder.push(line, col, ma.root.len() as u32, token_types::VARIABLE, 0);

            let mut offset = col + ma.root.len() as u32 + 1; // +1 for dot
            for segment in &ma.path {
                builder.push(line, offset, segment.len() as u32, token_types::PROPERTY, 0);
                offset += segment.len() as u32 + 1;
            }
        }

        Expression::Literal(lit) => {
            let line = (lit.span.start_line.saturating_sub(1)) as u32;
            let col = (lit.span.start_col.saturating_sub(1)) as u32;
            let (len, token_type) = match &lit.value {
                orbis_dsl::ast::LiteralValue::String(s) => (s.len() + 2, token_types::STRING),
                orbis_dsl::ast::LiteralValue::Number(n) => (format!("{}", n).len(), token_types::NUMBER),
                orbis_dsl::ast::LiteralValue::Integer(i) => (format!("{}", i).len(), token_types::NUMBER),
                orbis_dsl::ast::LiteralValue::Boolean(b) => (if *b { 4 } else { 5 }, token_types::KEYWORD),
                orbis_dsl::ast::LiteralValue::Null => (4, token_types::KEYWORD),
            };
            builder.push(line, col, len as u32, token_type, 0);
        }

        Expression::Binary(bin) => {
            visit_expression(&bin.left, builder);
            visit_expression(&bin.right, builder);
            // Note: Operator position requires more context, skipping for now
        }

        Expression::Unary(unary) => {
            let line = (unary.span.start_line.saturating_sub(1)) as u32;
            let col = (unary.span.start_col.saturating_sub(1)) as u32;
            builder.push(line, col, unary.op.as_str().len() as u32, token_types::OPERATOR, 0);

            visit_expression(&unary.operand, builder);
        }

        Expression::MethodCall(call) => {
            let line = (call.span.start_line.saturating_sub(1)) as u32;
            let col = (call.span.start_col.saturating_sub(1)) as u32;
            builder.push(line, col, call.namespace.len() as u32, token_types::NAMESPACE, 0);
            builder.push(line, col + call.namespace.len() as u32 + 1, call.method.len() as u32, token_types::FUNCTION, 0);

            for arg in &call.arguments {
                visit_expression(&arg.value, builder);
            }
        }

        Expression::Array(arr) => {
            for elem in &arr.elements {
                visit_expression(elem, builder);
            }
        }

        Expression::Object(obj) => {
            for pair in &obj.pairs {
                let line = (pair.span.start_line.saturating_sub(1)) as u32;
                let col = (pair.span.start_col.saturating_sub(1)) as u32;
                builder.push(line, col, pair.key.len() as u32, token_types::PROPERTY, 0);
                visit_expression(&pair.value, builder);
            }
        }

        Expression::SpecialVariable(sv) => {
            let line = (sv.span.start_line.saturating_sub(1)) as u32;
            let col = (sv.span.start_col.saturating_sub(1)) as u32;
            let prefix = sv.kind.prefix();
            builder.push(line, col, prefix.len() as u32, token_types::VARIABLE, token_modifiers::READONLY);
        }

        Expression::Grouped { inner, .. } => {
            visit_expression(inner, builder);
        }

        Expression::ArrowFunction(arrow) => {
            // Note: Parameter spans not available in current AST structure
            // Would need to be added to support precise highlighting
            match &arrow.body {
                orbis_dsl::ast::ArrowBody::Expression(expr) => {
                    visit_expression(expr, builder);
                }
                orbis_dsl::ast::ArrowBody::Block(stmts) => {
                    for stmt in stmts {
                        match stmt {
                            orbis_dsl::ast::ArrowStatement::Expression(expr) => {
                                visit_expression(expr, builder);
                            }
                            orbis_dsl::ast::ArrowStatement::Return(Some(expr)) => {
                                visit_expression(expr, builder);
                            }
                            orbis_dsl::ast::ArrowStatement::Let { value, .. } => {
                                visit_expression(value, builder);
                            }
                            orbis_dsl::ast::ArrowStatement::Const { value, .. } => {
                                visit_expression(value, builder);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        Expression::InterpolatedString(interp) => {
            // Note: Individual part positions not readily available
            // Would need span information for each StringPart
            for part in &interp.parts {
                match part {
                    orbis_dsl::ast::StringPart::Literal { .. } => {
                        // String literal part
                    }
                    orbis_dsl::ast::StringPart::Expression { value } => {
                        visit_expression(value, builder);
                    }
                }
            }
        }

        Expression::Assignment(assign) => {
            visit_expression(&assign.target, builder);
            let line = (assign.span.start_line.saturating_sub(1)) as u32;
            builder.push(line, 0, 1, token_types::OPERATOR, 0); // =
            visit_expression(&assign.value, builder);
        }
    }
}

/// Visit action item
fn visit_action_item(action_item: &ActionItem, builder: &mut SemanticTokenBuilder) {
    match action_item {
        ActionItem::Simple(action) => {
            visit_action(action, builder);
        }
        ActionItem::WithHandlers(awh) => {
            visit_action(&awh.action, builder);
            for handler in &awh.handlers {
                for action in &handler.actions {
                    visit_action_item(action, builder);
                }
            }
        }
    }
}

/// Visit action
fn visit_action(action: &Action, builder: &mut SemanticTokenBuilder) {
    match action {
        Action::StateAssignment(sa) => {
            let line = (sa.span.start_line.saturating_sub(1)) as u32;
            builder.push(line, 0, sa.target.root.len() as u32, token_types::VARIABLE, token_modifiers::MODIFICATION);
            visit_expression(&sa.value, builder);
        }

        Action::MethodCall(mc) => {
            let line = (mc.span.start_line.saturating_sub(1)) as u32;
            let col = (mc.span.start_col.saturating_sub(1)) as u32;
            
            // Highlight namespace (console, toast, etc.)
            builder.push(line, col, mc.namespace.len() as u32, token_types::NAMESPACE, 0);
            
            // Highlight method name (log, show, etc.)
            let method_col = col + mc.namespace.len() as u32 + 1; // +1 for dot
            builder.push(line, method_col, mc.method.len() as u32, token_types::FUNCTION, 0);
            
            for arg in &mc.arguments {
                visit_expression(&arg.value, builder);
            }
        }

        Action::Fetch(fa) => {
            let line = (fa.span.start_line.saturating_sub(1)) as u32;
            builder.push(line, 0, 5, token_types::FUNCTION, 0); // "fetch"
            if let Some(url) = &fa.url {
                visit_expression(url, builder);
            }
        }

        Action::Submit(sa) => {
            let line = (sa.span.start_line.saturating_sub(1)) as u32;
            builder.push(line, 0, 6, token_types::FUNCTION, 0); // "submit"
        }

        Action::Call(ca) => {
            let line = (ca.span.start_line.saturating_sub(1)) as u32;
            builder.push(line, 0, 4, token_types::FUNCTION, 0); // "call"
            builder.push(line, 5, ca.api.len() as u32, token_types::TYPE, 0);
        }

        Action::Custom(ca) => {
            let line = (ca.span.start_line.saturating_sub(1)) as u32;
            builder.push(line, 0, ca.name.len() as u32, token_types::FUNCTION, 0);
        }
    }
}

/// Helper to emit a token at a span
fn emit_token(builder: &mut SemanticTokenBuilder, span: &Span, offset: u32, length: u32, token_type: u32, modifiers: u32) {
    let line = (span.start_line.saturating_sub(1)) as u32;
    builder.push(line, offset, length, token_type, modifiers);
}
