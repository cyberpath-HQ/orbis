---
sidebar_position: 2
title: State Management
description: Complete state management API reference
---

## State Management

State management in Orbis plugins is declarative and schema-driven. This reference covers all state-related APIs.

## State Initialization

Define initial state in your page definition:

```json
{
  "id": "myPage",
  "title": "My Page",
  "initialState": {
    "count": 0,
    "user": null,
    "items": [],
    "settings": {
      "theme": "light",
      "notifications": true
    }
  }
}
```

## State Types

### Primitive Types

| Type | Example | Access Pattern |
|------|---------|----------------|
| `string` | `"hello"` | `{{state.message}}` |
| `number` | `42` | `{{state.count}}` |
| `boolean` | `true` | `{{state.isActive}}` |
| `null` | `null` | `{{state.user}}` |

### Complex Types

| Type | Example | Access Pattern |
|------|---------|----------------|
| `object` | `{"name": "John"}` | `{{state.user.name}}` |
| `array` | `[1, 2, 3]` | `{{state.items[0]}}` |
| `nested` | `{"a": {"b": 1}}` | `{{state.a.b}}` |

## State Access

### Basic Access

```json
"{{state.fieldName}}"
```

### Nested Access

```json
"{{state.user.profile.name}}"
"{{state.settings.display.theme}}"
```

### Array Access

```json
"{{state.items[0]}}"
"{{state.items[state.selectedIndex]}}"
"{{state.items.length}}"
```

### Safe Access

Use optional chaining for potentially undefined values:

```json
"{{state.user?.name}}"
"{{state.items?.[0]?.title}}"
```

## State Mutations

### updateState Action

```json
{
  "type": "updateState",
  "path": "fieldName",
  "value": "newValue"
}
```

### Path Syntax

| Path | Target | Example |
|------|--------|---------|
| `field` | Root field | `count` |
| `a.b` | Nested field | `user.name` |
| `arr[0]` | Array index | `items[0]` |
| `arr[]` | Array push | `items[]` |
| `a.b.c` | Deep nested | `settings.ui.theme` |

### Value Types

**Static values:**

```json
{ "type": "updateState", "path": "count", "value": 10 }
{ "type": "updateState", "path": "name", "value": "John" }
{ "type": "updateState", "path": "active", "value": true }
```

**Expression values:**

```json
{ "type": "updateState", "path": "count", "value": "{{state.count + 1}}" }
{ "type": "updateState", "path": "fullName", "value": "{{state.firstName}} {{state.lastName}}" }
```

**Object values:**

```json
{
  "type": "updateState",
  "path": "user",
  "value": {
    "name": "{{$response.name}}",
    "email": "{{$response.email}}"
  }
}
```

**Array values:**

```json
{ "type": "updateState", "path": "items", "value": "{{$response.data}}" }
{ "type": "updateState", "path": "selectedIds", "value": [] }
```

## Array Operations

### Push to Array

```json
{ "type": "updateState", "path": "items[]", "value": "{{newItem}}" }
```

### Update Array Item

```json
{ "type": "updateState", "path": "items[{{state.selectedIndex}}]", "value": "{{updatedItem}}" }
```

### Remove from Array

```json
{
  "type": "updateState",
  "path": "items",
  "value": "{{state.items.filter((_, i) => i !== state.selectedIndex)}}"
}
```

### Clear Array

```json
{ "type": "updateState", "path": "items", "value": [] }
```

### Map Array

```json
{
  "type": "updateState",
  "path": "items",
  "value": "{{state.items.map(item => ({ ...item, selected: false }))}}"
}
```

## Object Operations

### Update Object Property

```json
{ "type": "updateState", "path": "user.name", "value": "Jane" }
```

### Merge Objects

```json
{
  "type": "updateState",
  "path": "settings",
  "value": "{{ { ...state.settings, theme: 'dark' } }}"
}
```

### Clear Object

```json
{ "type": "updateState", "path": "user", "value": null }
```

## Computed State

Use expressions for computed values:

```json
{
  "type": "Text",
  "text": "Total: {{state.items.reduce((sum, item) => sum + item.price, 0)}}"
}
```

```json
{
  "type": "Text",
  "text": "{{state.items.filter(i => i.active).length}} active items"
}
```

## State in Components

### Conditional Visibility

```json
{
  "type": "Alert",
  "visible": "{{state.showWarning}}",
  "message": "Warning message"
}
```

### Dynamic Content

```json
{
  "type": "Text",
  "text": "Welcome, {{state.user.name}}!"
}
```

### Disabled State

```json
{
  "type": "Button",
  "disabled": "{{state.isSubmitting || !state.isValid}}"
}
```

### Loading State

```json
{
  "type": "Button",
  "loading": "{{loading.submit}}",
  "text": "{{loading.submit ? 'Saving...' : 'Save'}}"
}
```

## State Persistence

State can be persisted across sessions:

```json
{
  "initialState": {
    "theme": "light"
  },
  "stateConfig": {
    "persist": ["theme", "sidebarOpen"]
  }
}
```

## State Reset

Reset state to initial values:

```json
{
  "type": "updateState",
  "path": "user",
  "value": null
}
```

Reset multiple fields:

```json
{
  "type": "sequence",
  "actions": [
    { "type": "updateState", "path": "user", "value": null },
    { "type": "updateState", "path": "items", "value": [] },
    { "type": "updateState", "path": "loading", "value": false }
  ]
}
```

## Best Practices

1. **Initialize all state** - Define all fields in `initialState`
2. **Use appropriate types** - Match state types to usage
3. **Keep state flat** - Avoid deeply nested structures when possible
4. **Use meaningful names** - Self-documenting field names
5. **Handle null/undefined** - Use safe access patterns
6. **Minimize state** - Only store what's needed
