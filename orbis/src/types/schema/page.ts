/**
 * Page definition schema for the JSON UI system
 */

import type {
    StateDefinition, Expression, IconName, BooleanExpression
} from './base';
import type { Action } from './actions';
import type { ComponentSchema } from './components';

// Permission definition
export interface PermissionRequirement {
    permission: string
    mode?:      `any` | `all`
}

// Page state configuration
export interface PageStateConfig {
    initial:   StateDefinition
    computed?: Record<string, Expression>
    watchers?: Array<{
        watch:   string
        actions: Array<Action>
    }>
}

// Page route configuration
export interface PageRouteConfig {
    path:    string
    params?: Record<string, {
        type:      `string` | `number`
        required?: boolean
    }>
    query?: Record<string, {
        type:     `string` | `number` | `boolean` | `array`
        default?: unknown
    }>
}

// Page meta configuration
export interface PageMetaConfig {
    title:        Expression
    description?: Expression
    icon?:        IconName
    showInMenu?:  boolean
    menuOrder?:   number
    parentRoute?: string
    tags?:        Array<string>
}

// Page security configuration
export interface PageSecurityConfig {
    requiresAuth?:           boolean
    permissions?:            Array<string | PermissionRequirement>
    roles?:                  Array<string>
    redirectOnUnauthorized?: string
}

// Page layout configuration
export interface PageLayoutConfig {
    type?:    `default` | `full` | `sidebar` | `centered`
    sidebar?: ComponentSchema
    header?:  ComponentSchema
    footer?:  ComponentSchema
}

// Page lifecycle hooks
export interface PageLifecycleHooks {
    onMount?:        Array<Action>
    onUnmount?:      Array<Action>
    onParamsChange?: Array<Action>
    onQueryChange?:  Array<Action>
}

// Complete page definition
export interface PageDefinition {
    // Route configuration
    route: string

    // Meta information
    title:        Expression
    description?: Expression
    icon?:        IconName

    // Navigation configuration
    showInMenu?:  boolean
    menuOrder?:   number
    parentRoute?: string

    // Security configuration
    requiresAuth?: boolean
    permissions?:  Array<string>
    roles?:        Array<string>

    // State management
    state?: Record<string, {
        type:     `string` | `number` | `boolean` | `object` | `array`
        default?: unknown
    }>

    // Computed values
    computed?: Record<string, Expression>

    // Page sections/content
    sections: Array<ComponentSchema>

    // Layout configuration
    layout?: PageLayoutConfig

    // Lifecycle hooks
    hooks?: PageLifecycleHooks

    // Page-level actions (can be referenced by name)
    actions?: Record<string, Action>

    // Dialogs defined at page level
    dialogs?: Array<{
        id:           string
        title?:       Expression
        description?: Expression
        content:      ComponentSchema
        footer?:      ComponentSchema
        size?:        `sm` | `md` | `lg` | `xl` | `full`
    }>
}

// Navigation menu item
export interface NavigationItem {
    id:            string
    label:         Expression
    icon?:         IconName
    href?:         string
    external?:     boolean
    children?:     Array<NavigationItem>
    badge?:        Expression
    badgeVariant?: `default` | `secondary` | `destructive` | `outline`
    visible?:      BooleanExpression
    disabled?:     BooleanExpression
}

// Navigation configuration
export interface NavigationConfig {
    primary:    Array<NavigationItem>
    secondary?: Array<NavigationItem>
    user?:      Array<NavigationItem>
    footer?:    Array<NavigationItem>
}

// Application configuration (for core system pages)
export interface AppConfig {
    name:    string
    version: string
    logo?: {
        light: string
        dark:  string
    }
    navigation: NavigationConfig
    theme?: {
        primaryColor?: string
        mode?:         `light` | `dark` | `system`
    }
}

// Plugin page registration
export interface PluginPageRegistration {
    plugin: string
    page:   PageDefinition
}
