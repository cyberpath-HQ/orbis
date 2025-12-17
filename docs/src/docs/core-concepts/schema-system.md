---
sidebar_position: 3
title: Schema System
description: Defining UI with JSON schemas
---

# Schema System

The Orbis schema system allows you to define user interfaces declaratively using JSON. Instead of writing React components, you describe *what* you want, and Orbis renders it.

## Overview

Every UI in Orbis is defined by a **component schema** - a JSON object that describes the component type, its properties, and its children.

```json
{
  "type": "Button",
  "label": "Click Me",
  "variant": "default",
  "events": {
    "onClick": [
      { "type": "showToast", "message": "Hello!" }
    ]
  }
}
```

This schema renders a button that shows a toast when clicked.

## Schema Structure

### Basic Component Schema

Every component schema has:

```json
{
  "type": "ComponentType",   // Required: component type
  "id": "unique-id",         // Optional: unique identifier
  "className": "...",        // Optional: CSS classes
  "style": {},               // Optional: inline styles
  "visible": true,           // Optional: visibility condition
  "children": [],            // Optional: child components
  "events": {},              // Optional: event handlers
  // ...component-specific props
}
```

### Component Types

Orbis provides 35+ built-in component types:

| Category | Components |
|----------|------------|
| **Layout** | Container, Flex, Grid, Spacer, Divider, Section |
| **Typography** | Text, Heading |
| **Forms** | Form, Field |
| **Data Display** | Table, List, Card, StatCard, Badge, Avatar, Image |
| **Feedback** | Alert, Progress, Skeleton, LoadingOverlay, EmptyState |
| **Navigation** | Button, Link, Tabs, Breadcrumb, Dropdown, PageHeader |
| **Overlays** | Modal, Tooltip |
| **Advanced** | Accordion, Conditional, Loop, Icon, Fragment, Slot, Custom |

## Core Properties

### ID and ClassName

Identify and style components:

```json
{
  "type": "Container",
  "id": "main-content",
  "className": "p-4 bg-background rounded-lg shadow"
}
```

### Visibility

Control when components are shown:

```json
{
  "type": "Alert",
  "visible": "{{state.showAlert}}",
  "message": "This appears conditionally"
}
```

Visibility can be:
- `true` / `false` - static
- Expression string - dynamic based on state

### Inline Styles

Apply custom CSS:

```json
{
  "type": "Container",
  "style": {
    "padding": "20px",
    "backgroundColor": "#f0f0f0",
    "borderRadius": "8px"
  }
}
```

## Nesting Components

Components can contain other components:

```json
{
  "type": "Card",
  "title": "User Profile",
  "content": {
    "type": "Flex",
    "direction": "column",
    "gap": "1rem",
    "children": [
      {
        "type": "Avatar",
        "src": "{{state.user.avatar}}",
        "size": "lg"
      },
      {
        "type": "Heading",
        "level": 2,
        "text": "{{state.user.name}}"
      },
      {
        "type": "Text",
        "content": "{{state.user.bio}}"
      }
    ]
  }
}
```

## Expressions

Expressions make schemas dynamic by interpolating values from state.

### Syntax

Use double curly braces:

```json
{
  "text": "Hello, {{state.username}}!",
  "disabled": "{{state.isLoading}}",
  "value": "{{state.count + 1}}"
}
```

### Expression Context

Expressions can access:

| Source | Syntax | Description |
|--------|--------|-------------|
| State | `state.path.to.value` | Page state values |
| Context | `context.key` | Contextual data (loops, etc.) |
| Item | `$item` | Current item in loops |
| Index | `$index` | Current index in loops |
| Row | `$row` | Current row in tables |
| Event | `$event.value` | Event data |

### Expression Operations

Expressions support JavaScript-like operations:

```json
{
  "visible": "{{state.count > 0}}",
  "text": "{{state.firstName + ' ' + state.lastName}}",
  "className": "{{state.active ? 'active' : 'inactive'}}"
}
```

Supported operators:
- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Comparison: `==`, `===`, `!=`, `!==`, `>`, `>=`, `<`, `<=`
- Logical: `&&`, `||`, `!`
- Ternary: `condition ? trueValue : falseValue`

## Event Handlers

Events connect user interactions to actions:

```json
{
  "type": "Button",
  "label": "Submit",
  "events": {
    "onClick": [
      {
        "type": "validateForm",
        "formId": "my-form"
      },
      {
        "type": "callApi",
        "api": "my-plugin.submit",
        "args": {
          "data": "{{state.formData}}"
        }
      }
    ]
  }
}
```

### Available Events

| Event | Triggered When |
|-------|----------------|
| `onClick` | Component is clicked |
| `onChange` | Input value changes |
| `onSubmit` | Form is submitted |
| `onFocus` | Component gains focus |
| `onBlur` | Component loses focus |
| `onRowClick` | Table row is clicked |
| `onSelect` | Item is selected |
| `onPageChange` | Table page changes |
| `onSortChange` | Table sort changes |
| `onClose` | Modal/dialog closes |
| `onOpen` | Modal/dialog opens |

## Data Sources

Components that display lists of data use data sources:

```json
{
  "type": "Table",
  "dataSource": "state:users",
  "columns": [
    { "key": "name", "label": "Name" },
    { "key": "email", "label": "Email" }
  ]
}
```

### Data Source Formats

| Format | Example | Description |
|--------|---------|-------------|
| State reference | `state:users` | Array from state |
| Prop reference | `prop:items` | Passed from parent |
| Context reference | `context:data` | From context provider |

## Validation Rules

Form fields support validation:

```json
{
  "type": "Field",
  "name": "email",
  "fieldType": "email",
  "validation": {
    "required": { "message": "Email is required" },
    "email": { "message": "Invalid email format" },
    "maxLength": { "value": 100, "message": "Too long" }
  }
}
```

### Available Validations

| Rule | Description |
|------|-------------|
| `required` | Field must have a value |
| `min` / `max` | Number range |
| `minLength` / `maxLength` | String length |
| `pattern` | Regex pattern |
| `email` | Email format |
| `url` | URL format |
| `custom` | Custom expression |

## Layout Components

### Container

Basic wrapper component:

```json
{
  "type": "Container",
  "className": "p-4",
  "children": [...]
}
```

### Flex

Flexbox layout:

```json
{
  "type": "Flex",
  "direction": "row",
  "justify": "between",
  "align": "center",
  "gap": "1rem",
  "children": [...]
}
```

### Grid

CSS Grid layout:

```json
{
  "type": "Grid",
  "columns": { "sm": 1, "md": 2, "lg": 3 },
  "gap": "1rem",
  "children": [...]
}
```

## Conditional Rendering

### Using `visible`

Simple show/hide:

```json
{
  "type": "Alert",
  "visible": "{{state.hasError}}",
  "message": "{{state.errorMessage}}"
}
```

### Using Conditional Component

Full conditional with else:

```json
{
  "type": "Conditional",
  "condition": "{{state.isLoggedIn}}",
  "then": {
    "type": "Text",
    "content": "Welcome back!"
  },
  "else": {
    "type": "Button",
    "label": "Sign In"
  }
}
```

## Iteration

### Loop Component

Render a template for each item:

```json
{
  "type": "Loop",
  "dataSource": "state:items",
  "itemVar": "item",
  "indexVar": "i",
  "template": {
    "type": "Card",
    "title": "{{item.name}}",
    "content": {
      "type": "Text",
      "content": "Item #{{i + 1}}: {{item.description}}"
    }
  },
  "emptyTemplate": {
    "type": "EmptyState",
    "title": "No items",
    "description": "Add your first item to get started"
  }
}
```

### List Component

Simplified list rendering:

```json
{
  "type": "List",
  "dataSource": "state:notifications",
  "itemTemplate": {
    "type": "Flex",
    "children": [
      { "type": "Icon", "name": "{{$item.icon}}" },
      { "type": "Text", "content": "{{$item.message}}" }
    ]
  }
}
```

## ARIA & Accessibility

All components support ARIA properties:

```json
{
  "type": "Button",
  "label": "Menu",
  "ariaLabel": "Open navigation menu",
  "ariaExpanded": "{{state.menuOpen}}",
  "ariaControls": "nav-menu"
}
```

Available ARIA properties:
- `role`, `ariaLabel`, `ariaLabelledBy`, `ariaDescribedBy`
- `ariaDisabled`, `ariaExpanded`, `ariaPressed`, `ariaSelected`
- `ariaRequired`, `ariaInvalid`, `ariaErrorMessage`
- `ariaLive`, `ariaAtomic`, `ariaBusy`
- `tabIndex`

## Schema Validation

Orbis validates schemas at load time:

- Required properties are checked
- Types are verified
- Unknown component types raise errors
- Circular references are detected

Invalid schemas produce helpful error messages in development mode.

## Best Practices

### Keep Schemas Focused

Split complex UIs into multiple pages:

```json
{
  "pages": [
    { "id": "list", "route": "/items", "layout": {...} },
    { "id": "detail", "route": "/items/:id", "layout": {...} },
    { "id": "create", "route": "/items/new", "layout": {...} }
  ]
}
```

### Use Meaningful IDs

IDs help with debugging and testing:

```json
{
  "type": "Button",
  "id": "submit-order-btn",
  "testId": "submit-order"
}
```

### Leverage Components

Use semantic components instead of generic containers:

```json
// ✅ Good
{ "type": "Card", "title": "Settings", ... }

// ❌ Avoid
{ "type": "Container", "className": "card", ... }
```

## Next Steps

- **[State Management](./state-management)** - Managing page state
- **[Expressions](./expressions)** - Deep dive into expressions
- **[Components](../components/overview)** - All available components
