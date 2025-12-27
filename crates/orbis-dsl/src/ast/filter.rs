//! Filtering and query utilities for the AST
//!
//! This module provides utilities for querying and filtering AST nodes,
//! making it easy to find specific elements in the tree.

use super::component::{Component, FragmentDefinition, FragmentUsage};
use super::control::{ForBlock, IfBlock, WhenBlock};
use super::expr::{Expression, MemberAccess, MethodCall};
use super::node::{
    AstFile, ControlFlow, HookEntry, InterfaceDefinition, LifecycleHookKind, PageBlock, StateBlock,
    TemplateContent, TopLevelElement,
};
use super::state::StateDeclaration;
use super::visitor::Visitor;

/// Node kind for filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeKind {
    // Top-level
    Page,
    State,
    Hooks,
    Template,
    Fragment,
    Interface,
    Styles,
    Import,
    Export,

    // Template
    Component,
    IfBlock,
    ForBlock,
    WhenBlock,
    FragmentUsage,
    SlotDefinition,
    Text,
    Comment,

    // State
    RegularState,
    ComputedState,
    ValidatedState,

    // Hooks
    LifecycleHook,
    WatcherHook,

    // Expressions
    BinaryExpr,
    UnaryExpr,
    Literal,
    Identifier,
    MemberAccess,
    MethodCall,
    SpecialVariable,
    ObjectLiteral,
    ArrayLiteral,
    ArrowFunction,
}

/// AST filter for finding specific nodes
pub struct AstFilter;

impl AstFilter {
    /// Find all components in the AST
    pub fn find_components(file: &AstFile) -> Vec<&Component> {
        let mut collector = ComponentCollector { components: vec![] };
        collector.visit_file(file);
        collector.components
    }

    /// Find components by name
    pub fn find_components_by_name<'a>(file: &'a AstFile, name: &str) -> Vec<&'a Component> {
        Self::find_components(file)
            .into_iter()
            .filter(|c| c.name.eq_ignore_ascii_case(name))
            .collect()
    }

    /// Find all state declarations
    pub fn find_state_declarations(file: &AstFile) -> Vec<&StateDeclaration> {
        let mut result = vec![];
        for element in &file.elements {
            if let TopLevelElement::State(state) = element {
                result.extend(state.declarations.iter());
            }
        }
        result
    }

    /// Find state declaration by name
    pub fn find_state_by_name<'a>(file: &'a AstFile, name: &str) -> Option<&'a StateDeclaration> {
        Self::find_state_declarations(file)
            .into_iter()
            .find(|s| s.name() == name)
    }

    /// Find all fragment definitions
    pub fn find_fragments(file: &AstFile) -> Vec<&FragmentDefinition> {
        let mut result = vec![];
        for element in &file.elements {
            match element {
                TopLevelElement::Fragment(f) => result.push(f),
                TopLevelElement::Export(e) => {
                    if let super::node::ExportableItem::Fragment(f) = &e.item {
                        result.push(f);
                    }
                }
                _ => {}
            }
        }
        result
    }

    /// Find fragment by name
    pub fn find_fragment_by_name<'a>(file: &'a AstFile, name: &str) -> Option<&'a FragmentDefinition> {
        Self::find_fragments(file)
            .into_iter()
            .find(|f| f.name == name)
    }

    /// Find all interface definitions
    pub fn find_interfaces(file: &AstFile) -> Vec<&InterfaceDefinition> {
        let mut result = vec![];
        for element in &file.elements {
            match element {
                TopLevelElement::Interface(i) => result.push(i),
                TopLevelElement::Export(e) => {
                    if let super::node::ExportableItem::Interface(i) = &e.item {
                        result.push(i);
                    }
                }
                _ => {}
            }
        }
        result
    }

    /// Find interface by name
    pub fn find_interface_by_name<'a>(file: &'a AstFile, name: &str) -> Option<&'a InterfaceDefinition> {
        Self::find_interfaces(file)
            .into_iter()
            .find(|i| i.name == name)
    }

    /// Find all fragment usages
    pub fn find_fragment_usages(file: &AstFile) -> Vec<&FragmentUsage> {
        let mut collector = FragmentUsageCollector { usages: vec![] };
        collector.visit_file(file);
        collector.usages
    }

    /// Find usages of a specific fragment
    pub fn find_usages_of_fragment<'a>(file: &'a AstFile, name: &str) -> Vec<&'a FragmentUsage> {
        Self::find_fragment_usages(file)
            .into_iter()
            .filter(|u| u.name == name)
            .collect()
    }

    /// Find all method calls
    pub fn find_method_calls(file: &AstFile) -> Vec<&MethodCall> {
        let mut collector = MethodCallCollector { calls: vec![] };
        collector.visit_file(file);
        collector.calls
    }

    /// Find method calls by namespace
    pub fn find_method_calls_by_namespace<'a>(file: &'a AstFile, namespace: &str) -> Vec<&'a MethodCall> {
        Self::find_method_calls(file)
            .into_iter()
            .filter(|c| c.namespace.eq_ignore_ascii_case(namespace))
            .collect()
    }

    /// Find all state paths (state.xxx references)
    pub fn find_state_paths(file: &AstFile) -> Vec<&MemberAccess> {
        let mut collector = StatePathCollector { paths: vec![] };
        collector.visit_file(file);
        collector.paths
    }

    /// Find all if blocks
    pub fn find_if_blocks(file: &AstFile) -> Vec<&IfBlock> {
        let mut collector = ControlFlowCollector {
            ifs: vec![],
            fors: vec![],
            whens: vec![],
        };
        collector.visit_file(file);
        collector.ifs
    }

    /// Find all for blocks
    pub fn find_for_blocks(file: &AstFile) -> Vec<&ForBlock> {
        let mut collector = ControlFlowCollector {
            ifs: vec![],
            fors: vec![],
            whens: vec![],
        };
        collector.visit_file(file);
        collector.fors
    }

    /// Find all when blocks
    pub fn find_when_blocks(file: &AstFile) -> Vec<&WhenBlock> {
        let mut collector = ControlFlowCollector {
            ifs: vec![],
            fors: vec![],
            whens: vec![],
        };
        collector.visit_file(file);
        collector.whens
    }

    /// Get page block if present
    pub fn get_page(file: &AstFile) -> Option<&PageBlock> {
        file.elements.iter().find_map(|e| {
            if let TopLevelElement::Page(p) = e {
                Some(p)
            } else {
                None
            }
        })
    }

    /// Get state block if present
    pub fn get_state_block(file: &AstFile) -> Option<&StateBlock> {
        file.elements.iter().find_map(|e| {
            if let TopLevelElement::State(s) = e {
                Some(s)
            } else {
                None
            }
        })
    }

    /// Find all lifecycle hooks of a specific kind
    pub fn find_lifecycle_hooks(file: &AstFile, kind: LifecycleHookKind) -> Vec<&super::node::LifecycleHook> {
        let mut result = vec![];
        for element in &file.elements {
            if let TopLevelElement::Hooks(hooks) = element {
                for entry in &hooks.entries {
                    if let HookEntry::Lifecycle(hook) = entry {
                        if hook.kind == kind {
                            result.push(hook);
                        }
                    }
                }
            }
        }
        result
    }

    /// Find all watcher hooks
    pub fn find_watcher_hooks(file: &AstFile) -> Vec<&super::node::WatcherHook> {
        let mut result = vec![];
        for element in &file.elements {
            if let TopLevelElement::Hooks(hooks) = element {
                for entry in &hooks.entries {
                    if let HookEntry::Watcher(hook) = entry {
                        result.push(hook);
                    }
                }
            }
        }
        result
    }

    /// Check if file has any validated state
    pub fn has_validated_state(file: &AstFile) -> bool {
        Self::find_state_declarations(file)
            .iter()
            .any(|s| matches!(s, StateDeclaration::Validated(_)))
    }

    /// Check if file has any computed state
    pub fn has_computed_state(file: &AstFile) -> bool {
        Self::find_state_declarations(file)
            .iter()
            .any(|s| matches!(s, StateDeclaration::Computed(_)))
    }

    /// Get all unique component names used in the file
    pub fn get_unique_component_names(file: &AstFile) -> Vec<String> {
        let components = Self::find_components(file);
        let mut names: Vec<String> = components.iter().map(|c| c.name.clone()).collect();
        names.sort();
        names.dedup();
        names
    }

    /// Get all unique fragment names used in the file
    pub fn get_unique_fragment_names(file: &AstFile) -> Vec<String> {
        let usages = Self::find_fragment_usages(file);
        let mut names: Vec<String> = usages.iter().map(|u| u.name.clone()).collect();
        names.sort();
        names.dedup();
        names
    }
}

// ============================================================================
// Collector visitors
// ============================================================================

struct ComponentCollector<'a> {
    components: Vec<&'a Component>,
}

impl<'a> Visitor for ComponentCollector<'a> {
    fn visit_component(&mut self, component: &Component) {
        // Safety: We're collecting references from the AST which lives for 'a
        self.components.push(unsafe { std::mem::transmute(component) });
        super::visitor::walk_component(self, component);
    }
}

struct FragmentUsageCollector<'a> {
    usages: Vec<&'a FragmentUsage>,
}

impl<'a> Visitor for FragmentUsageCollector<'a> {
    fn visit_fragment_usage(&mut self, usage: &FragmentUsage) {
        self.usages.push(unsafe { std::mem::transmute(usage) });
        super::visitor::walk_fragment_usage(self, usage);
    }
}

struct MethodCallCollector<'a> {
    calls: Vec<&'a MethodCall>,
}

impl<'a> Visitor for MethodCallCollector<'a> {
    fn visit_method_call(&mut self, call: &MethodCall) {
        self.calls.push(unsafe { std::mem::transmute(call) });
        super::visitor::walk_method_call(self, call);
    }
}

struct StatePathCollector<'a> {
    paths: Vec<&'a MemberAccess>,
}

impl<'a> Visitor for StatePathCollector<'a> {
    fn visit_member_access(&mut self, access: &MemberAccess) {
        if access.is_state() {
            self.paths.push(unsafe { std::mem::transmute(access) });
        }
    }
}

struct ControlFlowCollector<'a> {
    ifs: Vec<&'a IfBlock>,
    fors: Vec<&'a ForBlock>,
    whens: Vec<&'a WhenBlock>,
}

impl<'a> Visitor for ControlFlowCollector<'a> {
    fn visit_if_block(&mut self, if_block: &IfBlock) {
        self.ifs.push(unsafe { std::mem::transmute(if_block) });
        super::visitor::walk_if_block(self, if_block);
    }

    fn visit_for_block(&mut self, for_block: &ForBlock) {
        self.fors.push(unsafe { std::mem::transmute(for_block) });
        super::visitor::walk_for_block(self, for_block);
    }

    fn visit_when_block(&mut self, when_block: &WhenBlock) {
        self.whens.push(unsafe { std::mem::transmute(when_block) });
        super::visitor::walk_when_block(self, when_block);
    }
}

// ============================================================================
// Expression filters
// ============================================================================

impl AstFilter {
    /// Collect all expressions of a specific kind
    pub fn find_expressions<F>(file: &AstFile, predicate: F) -> Vec<&Expression>
    where
        F: Fn(&Expression) -> bool,
    {
        let mut collector = ExpressionCollector {
            expressions: vec![],
            predicate: Box::new(predicate),
        };
        collector.visit_file(file);
        collector.expressions
    }

    /// Find all literal expressions
    pub fn find_literals(file: &AstFile) -> Vec<&Expression> {
        Self::find_expressions(file, |e| e.is_literal())
    }

    /// Find all binary expressions
    pub fn find_binary_expressions(file: &AstFile) -> Vec<&Expression> {
        Self::find_expressions(file, |e| matches!(e, Expression::Binary(_)))
    }
}

struct ExpressionCollector<'a, F>
where
    F: Fn(&Expression) -> bool,
{
    expressions: Vec<&'a Expression>,
    predicate: Box<F>,
}

impl<'a, F> Visitor for ExpressionCollector<'a, F>
where
    F: Fn(&Expression) -> bool,
{
    fn visit_expression(&mut self, expr: &Expression) {
        if (self.predicate)(expr) {
            self.expressions.push(unsafe { std::mem::transmute(expr) });
        }
        super::visitor::walk_expression(self, expr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_kind_equality() {
        assert_eq!(NodeKind::Component, NodeKind::Component);
        assert_ne!(NodeKind::Component, NodeKind::Fragment);
    }
}
