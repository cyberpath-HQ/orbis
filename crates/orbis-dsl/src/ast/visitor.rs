//! Visitor pattern for AST traversal
//!
//! This module provides traits for walking the AST using the visitor pattern.
//! Both immutable (`Visitor`) and mutable (`MutVisitor`) traversal are supported.

use super::component::{
    Attribute, Component, EventBinding, FragmentDefinition, FragmentParam, FragmentUsage,
    SlotContent, SlotDefinition,
};
use super::control::{ForBlock, IfBlock, WhenArm, WhenBlock};
use super::expr::{
    ArrayLiteral, ArrowFunction, BinaryExpr, Expression, Identifier, InterpolatedString, Literal,
    MemberAccess, MethodCall, ObjectLiteral, SpecialVariable, StateAssignment, StringPart,
    UnaryExpr,
};
use super::node::{
    Action, ActionItem, ActionWithHandlers, AstFile, ControlFlow, ExportStatement, HookEntry,
    HooksBlock, ImportStatement, InterfaceDefinition, InterfaceMember, LifecycleHook, PageBlock,
    ResponseHandler, StateBlock, StylesBlock, TemplateBlock, TemplateContent, TopLevelElement,
    WatcherHook,
};
use super::state::{ComputedState, RegularState, StateDeclaration, ValidatedState, Validator};
use super::types::{GenericParam, TypeAnnotation};

/// Trait for types that can be visited/walked
pub trait Walkable {
    /// Accept an immutable visitor
    fn walk<V: Visitor>(&self, visitor: &mut V);

    /// Accept a mutable visitor
    fn walk_mut<V: MutVisitor>(&mut self, visitor: &mut V);
}

/// Immutable visitor trait for AST traversal
///
/// Default implementations visit children. Override specific methods
/// to customize behavior for that node type.
#[allow(unused_variables)]
pub trait Visitor: Sized {
    // ========================================================================
    // File level
    // ========================================================================

    fn visit_file(&mut self, file: &AstFile) {
        walk_file(self, file);
    }

    fn visit_import(&mut self, import: &ImportStatement) {}

    fn visit_top_level(&mut self, element: &TopLevelElement) {
        walk_top_level(self, element);
    }

    fn visit_export(&mut self, export: &ExportStatement) {
        walk_export(self, export);
    }

    // ========================================================================
    // Block level
    // ========================================================================

    fn visit_page(&mut self, page: &PageBlock) {}

    fn visit_state_block(&mut self, state: &StateBlock) {
        walk_state_block(self, state);
    }

    fn visit_hooks_block(&mut self, hooks: &HooksBlock) {
        walk_hooks_block(self, hooks);
    }

    fn visit_template(&mut self, template: &TemplateBlock) {
        walk_template(self, template);
    }

    fn visit_interface(&mut self, interface: &InterfaceDefinition) {
        walk_interface(self, interface);
    }

    fn visit_interface_member(&mut self, member: &InterfaceMember) {
        walk_interface_member(self, member);
    }

    fn visit_styles(&mut self, styles: &StylesBlock) {}

    // ========================================================================
    // State declarations
    // ========================================================================

    fn visit_state_declaration(&mut self, decl: &StateDeclaration) {
        walk_state_declaration(self, decl);
    }

    fn visit_regular_state(&mut self, state: &RegularState) {
        walk_regular_state(self, state);
    }

    fn visit_computed_state(&mut self, state: &ComputedState) {
        walk_computed_state(self, state);
    }

    fn visit_validated_state(&mut self, state: &ValidatedState) {
        walk_validated_state(self, state);
    }

    fn visit_validator(&mut self, validator: &Validator) {}

    // ========================================================================
    // Hooks
    // ========================================================================

    fn visit_hook_entry(&mut self, entry: &HookEntry) {
        walk_hook_entry(self, entry);
    }

    fn visit_lifecycle_hook(&mut self, hook: &LifecycleHook) {
        walk_lifecycle_hook(self, hook);
    }

    fn visit_watcher_hook(&mut self, hook: &WatcherHook) {
        walk_watcher_hook(self, hook);
    }

    // ========================================================================
    // Template content
    // ========================================================================

    fn visit_template_content(&mut self, content: &TemplateContent) {
        walk_template_content(self, content);
    }

    fn visit_component(&mut self, component: &Component) {
        walk_component(self, component);
    }

    fn visit_attribute(&mut self, attr: &Attribute) {}

    fn visit_event_binding(&mut self, event: &EventBinding) {
        walk_event_binding(self, event);
    }

    fn visit_control_flow(&mut self, flow: &ControlFlow) {
        walk_control_flow(self, flow);
    }

    fn visit_if_block(&mut self, if_block: &IfBlock) {
        walk_if_block(self, if_block);
    }

    fn visit_for_block(&mut self, for_block: &ForBlock) {
        walk_for_block(self, for_block);
    }

    fn visit_when_block(&mut self, when_block: &WhenBlock) {
        walk_when_block(self, when_block);
    }

    fn visit_when_arm(&mut self, arm: &WhenArm) {
        walk_when_arm(self, arm);
    }

    // ========================================================================
    // Fragments
    // ========================================================================

    fn visit_fragment_definition(&mut self, fragment: &FragmentDefinition) {
        walk_fragment_definition(self, fragment);
    }

    fn visit_fragment_param(&mut self, param: &FragmentParam) {}

    fn visit_fragment_usage(&mut self, usage: &FragmentUsage) {
        walk_fragment_usage(self, usage);
    }

    fn visit_slot_content(&mut self, slot: &SlotContent) {
        walk_slot_content(self, slot);
    }

    fn visit_slot_definition(&mut self, slot: &SlotDefinition) {
        walk_slot_definition(self, slot);
    }

    // ========================================================================
    // Actions
    // ========================================================================

    fn visit_action_item(&mut self, item: &ActionItem) {
        walk_action_item(self, item);
    }

    fn visit_action(&mut self, action: &Action) {
        walk_action(self, action);
    }

    fn visit_action_with_handlers(&mut self, action: &ActionWithHandlers) {
        walk_action_with_handlers(self, action);
    }

    fn visit_response_handler(&mut self, handler: &ResponseHandler) {
        walk_response_handler(self, handler);
    }

    fn visit_state_assignment(&mut self, assignment: &StateAssignment) {
        walk_state_assignment(self, assignment);
    }

    fn visit_method_call(&mut self, call: &MethodCall) {
        walk_method_call(self, call);
    }

    // ========================================================================
    // Expressions
    // ========================================================================

    fn visit_expression(&mut self, expr: &Expression) {
        walk_expression(self, expr);
    }

    fn visit_binary_expr(&mut self, expr: &BinaryExpr) {
        walk_binary_expr(self, expr);
    }

    fn visit_unary_expr(&mut self, expr: &UnaryExpr) {
        walk_unary_expr(self, expr);
    }

    fn visit_literal(&mut self, literal: &Literal) {}

    fn visit_identifier(&mut self, ident: &Identifier) {}

    fn visit_member_access(&mut self, access: &MemberAccess) {}

    fn visit_special_variable(&mut self, var: &SpecialVariable) {}

    fn visit_object_literal(&mut self, obj: &ObjectLiteral) {
        walk_object_literal(self, obj);
    }

    fn visit_array_literal(&mut self, arr: &ArrayLiteral) {
        walk_array_literal(self, arr);
    }

    fn visit_arrow_function(&mut self, func: &ArrowFunction) {}

    fn visit_interpolated_string(&mut self, s: &InterpolatedString) {
        walk_interpolated_string(self, s);
    }

    fn visit_assignment(&mut self, a: &super::expr::Assignment) {
        walk_assignment(self, a);
    }

    // ========================================================================
    // Types
    // ========================================================================

    fn visit_type_annotation(&mut self, ty: &TypeAnnotation) {
        walk_type_annotation(self, ty);
    }

    fn visit_generic_param(&mut self, param: &GenericParam) {}
}

/// Mutable visitor trait for AST transformation
#[allow(unused_variables)]
pub trait MutVisitor: Sized {
    fn visit_file(&mut self, file: &mut AstFile) {
        walk_file_mut(self, file);
    }

    fn visit_expression(&mut self, expr: &mut Expression) {
        walk_expression_mut(self, expr);
    }

    fn visit_component(&mut self, component: &mut Component) {
        walk_component_mut(self, component);
    }

    fn visit_template_content(&mut self, content: &mut TemplateContent) {
        walk_template_content_mut(self, content);
    }

    fn visit_state_declaration(&mut self, decl: &mut StateDeclaration) {}

    fn visit_validator(&mut self, validator: &mut Validator) {}

    fn visit_attribute(&mut self, attr: &mut Attribute) {}
}

// ============================================================================
// Walk functions for Visitor
// ============================================================================

pub fn walk_file<V: Visitor>(visitor: &mut V, file: &AstFile) {
    for import in &file.imports {
        visitor.visit_import(import);
    }
    for element in &file.elements {
        visitor.visit_top_level(element);
    }
}

pub fn walk_top_level<V: Visitor>(visitor: &mut V, element: &TopLevelElement) {
    match element {
        TopLevelElement::Comment { .. } => {
            // Comments don't need to be visited - they're just metadata
        }
        TopLevelElement::Export(e) => visitor.visit_export(e),
        TopLevelElement::Page(p) => visitor.visit_page(p),
        TopLevelElement::State(s) => visitor.visit_state_block(s),
        TopLevelElement::Hooks(h) => visitor.visit_hooks_block(h),
        TopLevelElement::Template(t) => visitor.visit_template(t),
        TopLevelElement::Fragment(f) => visitor.visit_fragment_definition(f),
        TopLevelElement::Interface(i) => visitor.visit_interface(i),
        TopLevelElement::Styles(s) => visitor.visit_styles(s),
    }
}

pub fn walk_export<V: Visitor>(visitor: &mut V, export: &ExportStatement) {
    match &export.item {
        super::node::ExportableItem::Fragment(f) => visitor.visit_fragment_definition(f),
        super::node::ExportableItem::Interface(i) => visitor.visit_interface(i),
        super::node::ExportableItem::Const { value, type_annotation, .. } => {
            visitor.visit_expression(value);
            if let Some(ty) = type_annotation {
                visitor.visit_type_annotation(ty);
            }
        }
    }
}

pub fn walk_state_block<V: Visitor>(visitor: &mut V, state: &StateBlock) {
    for decl in &state.declarations {
        visitor.visit_state_declaration(decl);
    }
}

pub fn walk_state_declaration<V: Visitor>(visitor: &mut V, decl: &StateDeclaration) {
    match decl {
        StateDeclaration::Regular(s) => visitor.visit_regular_state(s),
        StateDeclaration::Computed(s) => visitor.visit_computed_state(s),
        StateDeclaration::Validated(s) => visitor.visit_validated_state(s),
    }
}

pub fn walk_regular_state<V: Visitor>(visitor: &mut V, state: &RegularState) {
    if let Some(ty) = &state.type_annotation {
        visitor.visit_type_annotation(ty);
    }
    if let Some(value) = &state.value {
        visitor.visit_expression(value);
    }
}

pub fn walk_computed_state<V: Visitor>(visitor: &mut V, state: &ComputedState) {
    if let Some(ty) = &state.type_annotation {
        visitor.visit_type_annotation(ty);
    }
    visitor.visit_expression(&state.expression);
}

pub fn walk_validated_state<V: Visitor>(visitor: &mut V, state: &ValidatedState) {
    if let Some(ty) = &state.type_annotation {
        visitor.visit_type_annotation(ty);
    }
    if let Some(value) = &state.value {
        visitor.visit_expression(value);
    }
    for validator in &state.validators {
        visitor.visit_validator(validator);
    }
}

pub fn walk_hooks_block<V: Visitor>(visitor: &mut V, hooks: &HooksBlock) {
    for entry in &hooks.entries {
        visitor.visit_hook_entry(entry);
    }
}

pub fn walk_hook_entry<V: Visitor>(visitor: &mut V, entry: &HookEntry) {
    match entry {
        HookEntry::Lifecycle(h) => visitor.visit_lifecycle_hook(h),
        HookEntry::Watcher(w) => visitor.visit_watcher_hook(w),
    }
}

pub fn walk_lifecycle_hook<V: Visitor>(visitor: &mut V, hook: &LifecycleHook) {
    for action in &hook.actions {
        visitor.visit_action_item(action);
    }
}

pub fn walk_watcher_hook<V: Visitor>(visitor: &mut V, hook: &WatcherHook) {
    for target in &hook.targets {
        visitor.visit_expression(target);
    }
    for action in &hook.actions {
        visitor.visit_action_item(action);
    }
}

pub fn walk_template<V: Visitor>(visitor: &mut V, template: &TemplateBlock) {
    for content in &template.content {
        visitor.visit_template_content(content);
    }
}

pub fn walk_template_content<V: Visitor>(visitor: &mut V, content: &TemplateContent) {
    match content {
        TemplateContent::Component(c) => visitor.visit_component(c),
        TemplateContent::ControlFlow(f) => visitor.visit_control_flow(f),
        TemplateContent::FragmentUsage(u) => visitor.visit_fragment_usage(u),
        TemplateContent::SlotDefinition(s) => visitor.visit_slot_definition(s),
        TemplateContent::Expression { expr, .. } => visitor.visit_expression(expr),
        TemplateContent::Text { .. } | TemplateContent::Comment { .. } => {}
    }
}

pub fn walk_component<V: Visitor>(visitor: &mut V, component: &Component) {
    for attr in &component.attributes {
        visitor.visit_attribute(attr);
    }
    for event in &component.events {
        visitor.visit_event_binding(event);
    }
    for child in &component.children {
        visitor.visit_template_content(child);
    }
}

pub fn walk_event_binding<V: Visitor>(visitor: &mut V, event: &EventBinding) {
    match &event.handler.handler_type {
        super::component::HandlerType::Expression(expr) => {
            visitor.visit_expression(expr);
        }
        super::component::HandlerType::Arrow(arrow) => {
            visitor.visit_arrow_function(arrow);
        }
        super::component::HandlerType::Identifier(_) => {}
    }
}

pub fn walk_control_flow<V: Visitor>(visitor: &mut V, flow: &ControlFlow) {
    match flow {
        ControlFlow::If(i) => visitor.visit_if_block(i),
        ControlFlow::For(f) => visitor.visit_for_block(f),
        ControlFlow::When(w) => visitor.visit_when_block(w),
    }
}

pub fn walk_if_block<V: Visitor>(visitor: &mut V, if_block: &IfBlock) {
    visitor.visit_expression(&if_block.condition);
    for content in &if_block.then_branch {
        visitor.visit_template_content(content);
    }
    for branch in &if_block.else_if_branches {
        visitor.visit_expression(&branch.condition);
        for content in &branch.body {
            visitor.visit_template_content(content);
        }
    }
    if let Some(else_branch) = &if_block.else_branch {
        for content in else_branch {
            visitor.visit_template_content(content);
        }
    }
}

pub fn walk_for_block<V: Visitor>(visitor: &mut V, for_block: &ForBlock) {
    visitor.visit_expression(&for_block.iterable);
    for content in &for_block.body {
        visitor.visit_template_content(content);
    }
}

pub fn walk_when_block<V: Visitor>(visitor: &mut V, when_block: &WhenBlock) {
    visitor.visit_expression(&when_block.subject);
    for arm in &when_block.arms {
        visitor.visit_when_arm(arm);
    }
}

pub fn walk_when_arm<V: Visitor>(visitor: &mut V, arm: &WhenArm) {
    for content in &arm.body {
        visitor.visit_template_content(content);
    }
}

pub fn walk_fragment_definition<V: Visitor>(visitor: &mut V, fragment: &FragmentDefinition) {
    for param in &fragment.params {
        visitor.visit_fragment_param(param);
    }
    for content in &fragment.body {
        visitor.visit_template_content(content);
    }
}

pub fn walk_fragment_usage<V: Visitor>(visitor: &mut V, usage: &FragmentUsage) {
    for attr in &usage.properties {
        visitor.visit_attribute(attr);
    }
    for event in &usage.events {
        visitor.visit_event_binding(event);
    }
    for slot in &usage.slot_content {
        visitor.visit_slot_content(slot);
    }
}

pub fn walk_slot_content<V: Visitor>(visitor: &mut V, slot: &SlotContent) {
    for content in &slot.content {
        visitor.visit_template_content(content);
    }
}

pub fn walk_slot_definition<V: Visitor>(visitor: &mut V, slot: &SlotDefinition) {
    for content in &slot.fallback {
        visitor.visit_template_content(content);
    }
}

pub fn walk_action_item<V: Visitor>(visitor: &mut V, item: &ActionItem) {
    match item {
        ActionItem::Simple(a) => visitor.visit_action(a),
        ActionItem::WithHandlers(a) => visitor.visit_action_with_handlers(a),
    }
}

pub fn walk_action<V: Visitor>(visitor: &mut V, action: &Action) {
    match action {
        Action::StateAssignment(a) => visitor.visit_state_assignment(a),
        Action::MethodCall(m) => visitor.visit_method_call(m),
        Action::Fetch(f) => {
            if let Some(url) = &f.url {
                visitor.visit_expression(url);
            }
            if let Some(body) = &f.body {
                visitor.visit_expression(body);
            }
            if let Some(headers) = &f.headers {
                visitor.visit_expression(headers);
            }
        }
        Action::Submit(s) => {
            if let Some(target) = &s.target {
                visitor.visit_expression(target);
            }
            if let Some(data) = &s.data {
                visitor.visit_expression(data);
            }
        }
        Action::Call(c) => {
            for arg in &c.args {
                visitor.visit_expression(arg);
            }
        }
        Action::Custom(c) => {
            for param in &c.params {
                visitor.visit_expression(param);
            }
        }
    }
}

pub fn walk_action_with_handlers<V: Visitor>(visitor: &mut V, action: &ActionWithHandlers) {
    visitor.visit_action(&action.action);
    for handler in &action.handlers {
        visitor.visit_response_handler(handler);
    }
}

pub fn walk_response_handler<V: Visitor>(visitor: &mut V, handler: &ResponseHandler) {
    for action in &handler.actions {
        visitor.visit_action_item(action);
    }
}

pub fn walk_state_assignment<V: Visitor>(visitor: &mut V, assignment: &StateAssignment) {
    visitor.visit_member_access(&assignment.target);
    visitor.visit_expression(&assignment.value);
}

pub fn walk_method_call<V: Visitor>(visitor: &mut V, call: &MethodCall) {
    for arg in &call.arguments {
        visitor.visit_expression(arg.value());
    }
}

pub fn walk_expression<V: Visitor>(visitor: &mut V, expr: &Expression) {
    match expr {
        Expression::Binary(e) => visitor.visit_binary_expr(e),
        Expression::Unary(e) => visitor.visit_unary_expr(e),
        Expression::Literal(l) => visitor.visit_literal(l),
        Expression::Identifier(i) => visitor.visit_identifier(i),
        Expression::MemberAccess(m) => visitor.visit_member_access(m),
        Expression::MethodCall(m) => visitor.visit_method_call(m),
        Expression::SpecialVariable(s) => visitor.visit_special_variable(s),
        Expression::Object(o) => visitor.visit_object_literal(o),
        Expression::Array(a) => visitor.visit_array_literal(a),
        Expression::Grouped { inner, .. } => visitor.visit_expression(inner),
        Expression::ArrowFunction(f) => visitor.visit_arrow_function(f),
        Expression::InterpolatedString(s) => visitor.visit_interpolated_string(s),
        Expression::Assignment(a) => visitor.visit_assignment(a),
    }
}

pub fn walk_binary_expr<V: Visitor>(visitor: &mut V, expr: &BinaryExpr) {
    visitor.visit_expression(&expr.left);
    visitor.visit_expression(&expr.right);
}

pub fn walk_unary_expr<V: Visitor>(visitor: &mut V, expr: &UnaryExpr) {
    visitor.visit_expression(&expr.operand);
}

pub fn walk_object_literal<V: Visitor>(visitor: &mut V, obj: &ObjectLiteral) {
    for pair in &obj.pairs {
        visitor.visit_expression(&pair.value);
    }
}

pub fn walk_array_literal<V: Visitor>(visitor: &mut V, arr: &ArrayLiteral) {
    for element in &arr.elements {
        visitor.visit_expression(element);
    }
}

pub fn walk_interpolated_string<V: Visitor>(visitor: &mut V, s: &InterpolatedString) {
    for part in &s.parts {
        if let StringPart::Expression { value } = part {
            visitor.visit_expression(value);
        }
    }
}

pub fn walk_assignment<V: Visitor>(visitor: &mut V, a: &super::expr::Assignment) {
    visitor.visit_expression(&a.target);
    visitor.visit_expression(&a.value);
}

pub fn walk_interface<V: Visitor>(visitor: &mut V, interface: &InterfaceDefinition) {
    for param in &interface.generics {
        visitor.visit_generic_param(param);
    }
    for member in &interface.members {
        visitor.visit_interface_member(member);
    }
}

pub fn walk_interface_member<V: Visitor>(visitor: &mut V, member: &InterfaceMember) {
    visitor.visit_type_annotation(&member.type_annotation);
}

pub fn walk_type_annotation<V: Visitor>(visitor: &mut V, ty: &TypeAnnotation) {
    match ty {
        TypeAnnotation::Optional { inner, .. } => visitor.visit_type_annotation(inner),
        TypeAnnotation::Array { element, .. } => visitor.visit_type_annotation(element),
        TypeAnnotation::Generic { args, .. } => {
            for arg in args {
                visitor.visit_type_annotation(arg);
            }
        }
        TypeAnnotation::Union { types, .. } => {
            for t in types {
                visitor.visit_type_annotation(t);
            }
        }
        _ => {}
    }
}

// ============================================================================
// Walk functions for MutVisitor
// ============================================================================

pub fn walk_file_mut<V: MutVisitor>(visitor: &mut V, file: &mut AstFile) {
    for element in &mut file.elements {
        match element {
            TopLevelElement::Template(t) => {
                for content in &mut t.content {
                    visitor.visit_template_content(content);
                }
            }
            TopLevelElement::State(s) => {
                for decl in &mut s.declarations {
                    visitor.visit_state_declaration(decl);
                }
            }
            _ => {}
        }
    }
}

pub fn walk_template_content_mut<V: MutVisitor>(visitor: &mut V, content: &mut TemplateContent) {
    if let TemplateContent::Component(c) = content {
        visitor.visit_component(c);
    }
}

pub fn walk_component_mut<V: MutVisitor>(visitor: &mut V, component: &mut Component) {
    for attr in &mut component.attributes {
        visitor.visit_attribute(attr);
    }
    for child in &mut component.children {
        visitor.visit_template_content(child);
    }
}

pub fn walk_expression_mut<V: MutVisitor>(visitor: &mut V, expr: &mut Expression) {
    match expr {
        Expression::Binary(e) => {
            visitor.visit_expression(&mut e.left);
            visitor.visit_expression(&mut e.right);
        }
        Expression::Unary(e) => {
            visitor.visit_expression(&mut e.operand);
        }
        Expression::Grouped { inner, .. } => {
            visitor.visit_expression(inner);
        }
        _ => {}
    }
}

// ============================================================================
// Convenience implementations
// ============================================================================

impl Walkable for AstFile {
    fn walk<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_file(self);
    }

    fn walk_mut<V: MutVisitor>(&mut self, visitor: &mut V) {
        visitor.visit_file(self);
    }
}

impl Walkable for Expression {
    fn walk<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_expression(self);
    }

    fn walk_mut<V: MutVisitor>(&mut self, visitor: &mut V) {
        visitor.visit_expression(self);
    }
}

impl Walkable for Component {
    fn walk<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_component(self);
    }

    fn walk_mut<V: MutVisitor>(&mut self, visitor: &mut V) {
        visitor.visit_component(self);
    }
}
