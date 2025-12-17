---
sidebar_position: 3
title: Expressions
description: Expression syntax and evaluation reference
---

## Expressions

Expressions in Orbis use double curly braces `{{expression}}` for dynamic value evaluation.

## Basic Syntax

### Interpolation

```json
"Hello, {{state.name}}!"
"Count: {{state.count}}"
```

### In Properties

```json
{
  "type": "Text",
  "text": "{{state.message}}",
  "visible": "{{state.showMessage}}",
  "className": "{{state.isError ? 'text-red-500' : 'text-green-500'}}"
}
```

## Expression Types

### String Expressions

```json
"{{state.firstName}} {{state.lastName}}"
"Hello, {{state.name}}!"
"Total: ${{state.total.toFixed(2)}}"
```

### Numeric Expressions

```json
"{{state.count + 1}}"
"{{state.price * state.quantity}}"
"{{Math.round(state.average * 100) / 100}}"
```

### Boolean Expressions

```json
"{{state.isActive}}"
"{{state.count > 0}}"
"{{state.user && state.user.isAdmin}}"
```

### Ternary Expressions

```json
"{{state.count > 0 ? 'Has items' : 'Empty'}}"
"{{state.loading ? 'Loading...' : 'Ready'}}"
"{{state.error ? state.error.message : 'Success'}}"
```

## Operators

### Comparison Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `===` | Strict equal | `{{state.status === 'active'}}` |
| `!==` | Not equal | `{{state.count !== 0}}` |
| `>` | Greater than | `{{state.age > 18}}` |
| `<` | Less than | `{{state.price < 100}}` |
| `>=` | Greater or equal | `{{state.score >= 60}}` |
| `<=` | Less or equal | `{{state.quantity <= 10}}` |

### Logical Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `&&` | Logical AND | `{{state.a && state.b}}` |
| `\|\|` | Logical OR | `{{state.a \|\| state.b}}` |
| `!` | Logical NOT | `{{!state.loading}}` |

### Arithmetic Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `+` | Addition | `{{state.a + state.b}}` |
| `-` | Subtraction | `{{state.a - state.b}}` |
| `*` | Multiplication | `{{state.price * state.qty}}` |
| `/` | Division | `{{state.total / state.count}}` |
| `%` | Modulo | `{{state.index % 2}}` |

### String Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `+` | Concatenation | `{{state.first + ' ' + state.last}}` |

## Built-in Functions

### Array Methods

```json
"{{state.items.length}}"
"{{state.items.filter(i => i.active)}}"
"{{state.items.map(i => i.name)}}"
"{{state.items.find(i => i.id === state.selectedId)}}"
"{{state.items.some(i => i.selected)}}"
"{{state.items.every(i => i.valid)}}"
"{{state.items.reduce((sum, i) => sum + i.value, 0)}}"
"{{state.items.includes(state.item)}}"
"{{state.items.indexOf(state.item)}}"
"{{state.items.slice(0, 5)}}"
"{{state.items.join(', ')}}"
```

### String Methods

```json
"{{state.name.toUpperCase()}}"
"{{state.name.toLowerCase()}}"
"{{state.name.trim()}}"
"{{state.name.split(' ')}}"
"{{state.name.substring(0, 10)}}"
"{{state.name.replace('old', 'new')}}"
"{{state.name.startsWith('prefix')}}"
"{{state.name.endsWith('suffix')}}"
"{{state.name.includes('search')}}"
```

### Number Methods

```json
"{{state.price.toFixed(2)}}"
"{{Math.round(state.value)}}"
"{{Math.floor(state.value)}}"
"{{Math.ceil(state.value)}}"
"{{Math.abs(state.value)}}"
"{{Math.min(state.a, state.b)}}"
"{{Math.max(state.a, state.b)}}"
```

### Object Methods

```json
"{{Object.keys(state.data)}}"
"{{Object.values(state.data)}}"
"{{Object.entries(state.data)}}"
```

### JSON Methods

```json
"{{JSON.stringify(state.data)}}"
```

## Context Variables

### State Context

```json
"{{state.fieldName}}"
"{{state.nested.field}}"
"{{state.array[0]}}"
```

### Form Context

```json
"{{form.formId}}"
"{{form.formId.fieldName}}"
"{{form.formId.$valid}}"
"{{form.formId.$dirty}}"
"{{form.formId.$errors}}"
"{{form.formId.$errors.fieldName}}"
```

### Event Context

```json
"{{$value}}"     // Current input value
"{{$event}}"     // Full event object
"{{$index}}"     // Loop iteration index
"{{$item}}"      // Current loop item
```

### Response Context

```json
"{{$response}}"           // Full API response
"{{$response.data}}"      // Response data
"{{$response.items}}"     // Response items
"{{$error}}"              // Error object
"{{$error.message}}"      // Error message
```

### Navigation Context

```json
"{{params.id}}"           // Route parameters
"{{query.search}}"        // Query parameters
"{{route.path}}"          // Current route path
```

### Loading Context

```json
"{{loading.keyName}}"     // Loading state
"{{loading.submit}}"      // Submit loading
"{{loading.fetch}}"       // Fetch loading
```

### Special Values

```json
"{{$now}}"                // Current timestamp
"{{$window}}"             // Window object
"{{$window.innerWidth}}"  // Window width
```

## Advanced Patterns

### Nullish Coalescing

```json
"{{state.user?.name ?? 'Anonymous'}}"
```

### Optional Chaining

```json
"{{state.data?.items?.[0]?.name}}"
```

### Template Literals

```json
"{{`Hello, ${state.name}!`}}"
```

### Spread Operator

```json
"{{ { ...state.user, name: 'Updated' } }}"
"{{ [...state.items, newItem] }}"
```

### Destructuring in Callbacks

```json
"{{state.items.map(({ id, name }) => `${id}: ${name}`).join(', ')}}"
```

## Expression Contexts

### Component Properties

```json
{
  "type": "Text",
  "text": "{{state.message}}",
  "visible": "{{state.showMessage}}",
  "className": "{{state.isActive ? 'active' : 'inactive'}}"
}
```

### Action Parameters

```json
{
  "type": "callApi",
  "api": "getUser",
  "params": {
    "id": "{{params.userId}}",
    "includeDetails": "{{state.showDetails}}"
  }
}
```

### Conditional Rendering

```json
{
  "type": "Conditional",
  "conditions": [
    {
      "when": "{{state.status === 'loading'}}",
      "render": { "type": "Skeleton" }
    },
    {
      "when": "{{state.status === 'error'}}",
      "render": { "type": "Alert", "variant": "destructive" }
    }
  ],
  "fallback": { "type": "Text", "text": "Ready" }
}
```

### Loop Rendering

```json
{
  "type": "Loop",
  "items": "{{state.items}}",
  "itemAs": "item",
  "indexAs": "index",
  "render": {
    "type": "Text",
    "text": "{{$index + 1}}. {{$item.name}}"
  }
}
```

## Best Practices

1. **Keep expressions simple** - Extract complex logic to state
2. **Use safe access** - Handle null/undefined values
3. **Avoid side effects** - Expressions should be pure
4. **Use meaningful variable names** - Self-documenting code
5. **Test edge cases** - Empty arrays, null values, etc.
