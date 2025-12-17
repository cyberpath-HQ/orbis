---
sidebar_position: 5
title: Expressions
description: Dynamic value interpolation in Orbis
---

# Expressions

Expressions are the core mechanism for making Orbis UIs dynamic. They allow you to compute values, evaluate conditions, and reference data at runtime.

## Syntax

Expressions use double curly braces:

```
{{expression}}
```

They can appear in any string property:

```json
{
  "type": "Text",
  "content": "Hello, {{state.username}}!"
}
```

## Expression Context

Every expression has access to various data sources:

### State

Page state values:

```json
"{{state.count}}"
"{{state.user.name}}"
"{{state.items[0]}}"
```

### Context

Contextual data (from loops, modals, etc.):

```json
"{{context.modalData}}"
"{{context.parentId}}"
```

### Special Variables

Event and iteration context:

| Variable | Description | Available In |
|----------|-------------|--------------|
| `$event` | Event object | Event handlers |
| `$event.value` | Input value | onChange handlers |
| `$event.target` | Event target | All event handlers |
| `$item` | Current item | Loop, List, Table |
| `$index` | Current index | Loop, List, Table |
| `$row` | Current row | Table components |
| `$response` | API response | callApi onSuccess |
| `$response.data` | Response body | callApi onSuccess |
| `$error` | Error object | callApi onError |

## Data Access

### Simple Properties

```json
"{{state.username}}"
```

### Nested Properties

Use dot notation:

```json
"{{state.user.profile.avatar}}"
```

### Array Elements

Use bracket notation:

```json
"{{state.items[0]}}"
"{{state.items[state.selectedIndex]}}"
```

### Object Keys

```json
"{{state.users[userId]}}"
```

## Operators

### Arithmetic

```json
"{{state.count + 1}}"
"{{state.price * state.quantity}}"
"{{state.total - state.discount}}"
"{{state.amount / 100}}"
"{{state.value % 2}}"
```

### Comparison

```json
"{{state.count > 0}}"
"{{state.count >= 10}}"
"{{state.age < 18}}"
"{{state.age <= 65}}"
"{{state.status == 'active'}}"
"{{state.status === 'active'}}"
"{{state.type != 'deleted'}}"
"{{state.type !== 'deleted'}}"
```

### Logical

```json
"{{state.isActive && state.isVerified}}"
"{{state.isAdmin || state.isModerator}}"
"{{!state.isLoading}}"
```

### Ternary

```json
"{{state.count > 0 ? 'Has items' : 'Empty'}}"
"{{state.isActive ? 'active' : 'inactive'}}"
```

## String Operations

### Concatenation

```json
"{{state.firstName + ' ' + state.lastName}}"
"Hello, {{state.name}}!"
```

### Template Strings

Mix static and dynamic content:

```json
"User {{state.username}} has {{state.points}} points"
```

## Common Patterns

### Conditional Visibility

```json
{
  "type": "Alert",
  "visible": "{{state.errors.length > 0}}",
  "message": "There are errors"
}
```

### Dynamic Classes

```json
{
  "type": "Container",
  "className": "{{state.isActive ? 'bg-primary' : 'bg-muted'}}"
}
```

### Computed Labels

```json
{
  "type": "Button",
  "label": "{{state.isEditing ? 'Save' : 'Edit'}}"
}
```

### Disabled States

```json
{
  "type": "Button",
  "disabled": "{{state.isLoading || !state.isValid}}"
}
```

### Loop Item Access

```json
{
  "type": "Loop",
  "dataSource": "state:items",
  "template": {
    "type": "Text",
    "content": "{{$index + 1}}. {{$item.name}}"
  }
}
```

### Event Values

```json
{
  "type": "Field",
  "events": {
    "onChange": [
      {
        "type": "updateState",
        "path": "searchQuery",
        "value": "$event.value"
      }
    ]
  }
}
```

### API Response Handling

```json
{
  "type": "callApi",
  "api": "my-plugin.getData",
  "onSuccess": [
    {
      "type": "updateState",
      "path": "data",
      "value": "$response.data"
    }
  ],
  "onError": [
    {
      "type": "showToast",
      "message": "Error: {{$error.message}}",
      "level": "error"
    }
  ]
}
```

## Boolean Expressions

Boolean expressions are evaluated for properties like `visible`, `disabled`, `loading`:

```json
{
  "visible": true,              // Static true
  "visible": false,             // Static false
  "visible": "{{state.show}}",  // Dynamic boolean
  "visible": "{{state.count > 0 && state.isActive}}"  // Complex condition
}
```

### Truthy/Falsy Evaluation

Expressions follow JavaScript truthy/falsy rules:

| Value | Boolean Result |
|-------|----------------|
| `true` | `true` |
| `false` | `false` |
| `0` | `false` |
| `""` | `false` |
| `null` | `false` |
| `undefined` | `false` |
| `[]` | `true` (empty array) |
| `{}` | `true` (empty object) |
| Any non-zero number | `true` |
| Any non-empty string | `true` |

## Expression Caching

Orbis caches expression results for performance:

- Parsed expressions are memoized
- Unchanged state produces cached results
- Cache is invalidated on state changes

This means repeated re-renders don't re-parse expressions.

## Error Handling

Invalid expressions degrade gracefully:

```json
// Missing property - returns undefined/empty
"{{state.nonexistent}}"  // → ""

// Type error - returns empty
"{{state.number.toUpperCase()}}"  // → "" (logged in development)
```

In development mode, expression errors are logged to help debugging.

## Limitations

### No Function Calls

Expressions cannot call arbitrary functions:

```json
// ❌ Not supported
"{{state.items.map(i => i.name)}}"
"{{Date.now()}}"
"{{Math.random()}}"
```

Use computed state or actions instead.

### No Assignments

Expressions are read-only:

```json
// ❌ Not supported
"{{state.count = 5}}"
"{{state.count++}}"
```

Use the `updateState` action to modify state.

### No Multi-Statement

Expressions are single expressions:

```json
// ❌ Not supported
"{{let x = 1; x + 2}}"
```

## Best Practices

### Keep Expressions Simple

```json
// ✅ Good
"{{state.isLoggedIn && state.hasPermission}}"

// ❌ Complex - hard to debug
"{{state.users.filter(u => u.active).length > 0 && state.settings.feature}}"
```

### Use Descriptive State Names

```json
// ✅ Good
"{{state.isSubmitButtonDisabled}}"

// ❌ Unclear
"{{state.d}}"
```

### Prefer Pre-Computed State

For complex logic, compute in actions:

```json
// Action that computes value
{
  "type": "updateState",
  "path": "canSubmit",
  "value": "{{state.isValid && !state.isLoading && state.hasChanges}}"
}

// Then use simply
{
  "disabled": "{{!state.canSubmit}}"
}
```

### Handle Missing Data

```json
// ✅ Handle potential undefined
"{{state.user?.name || 'Anonymous'}}"

// Or use conditional rendering
{
  "visible": "{{state.user}}",
  "content": "{{state.user.name}}"
}
```

## Debugging Expressions

### Development Mode

Enable debug logging:

```bash
RUST_LOG=debug bun run tauri dev
```

### Test Expressions

Use the console to test:

```json
{
  "type": "Text",
  "content": "DEBUG: {{state.myValue}}"
}
```

### Check Expression Results

Add debug output:

```json
{
  "type": "Conditional",
  "condition": "{{state.shouldShow}}",
  "then": {
    "type": "Text",
    "content": "Condition is true"
  },
  "else": {
    "type": "Text", 
    "content": "Condition is false: {{state.shouldShow}}"
  }
}
```

## Next Steps

- **[Event Handling](./event-handling)** - Responding to user input
- **[Actions](../actions/overview)** - All action types
- **[State Management](./state-management)** - Managing state
