---
sidebar_position: 4
title: navigate
description: Navigate between routes
---

## navigate

Changes the current route/page.

## Syntax

```json
{
  "type": "navigate",
  "to": "/path/to/page"
}
```

## Properties

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `to` | string | ✅ | Target route path |
| `replace` | boolean | - | Replace history instead of push |
| `state` | object | - | Navigation state |

## Examples

### Basic Navigation

```json
{
  "type": "navigate",
  "to": "/dashboard"
}
```

### With Dynamic Path

```json
{
  "type": "navigate",
  "to": "/items/{{state.selectedId}}"
}
```

### From Event Context

```json
{
  "type": "Table",
  "events": {
    "onRowClick": [
      { "type": "navigate", "to": "/users/{{$row.id}}" }
    ]
  }
}
```

### Replace History

```json
{
  "type": "navigate",
  "to": "/login",
  "replace": true
}
```

### With State

```json
{
  "type": "navigate",
  "to": "/items/new",
  "state": {
    "prefill": "{{state.selectedTemplate}}"
  }
}
```

## Common Patterns

### After Form Submit

```json
{
  "type": "Form",
  "events": {
    "onSubmit": [
      {
        "type": "callApi",
        "api": "createItem",
        "onSuccess": [
          { "type": "showToast", "message": "Created!", "level": "success" },
          { "type": "navigate", "to": "/items" }
        ]
      }
    ]
  }
}
```

### After Delete

```json
{
  "type": "callApi",
  "api": "deleteItem",
  "onSuccess": [
    { "type": "showToast", "message": "Deleted" },
    { "type": "navigate", "to": "/items", "replace": true }
  ]
}
```

### Back Navigation

```json
{
  "type": "Button",
  "label": "Back",
  "variant": "ghost",
  "events": {
    "onClick": [
      { "type": "navigate", "to": "/items" }
    ]
  }
}
```

### Conditional Navigation

```json
{
  "type": "conditional",
  "condition": "{{state.isAuthenticated}}",
  "then": [
    { "type": "navigate", "to": "/dashboard" }
  ],
  "else": [
    { "type": "navigate", "to": "/login" }
  ]
}
```

### Tab-Based Navigation

```json
{
  "type": "Tabs",
  "events": {
    "onTabChange": [
      { "type": "navigate", "to": "/settings/{{$value}}" }
    ]
  }
}
```

### External URLs

For external URLs, use [`openUrl`](./utility-actions#openurl) instead:

```json
{
  "type": "openUrl",
  "url": "https://github.com"
}
```

## Route Parameters

### Accessing Route Params

In page definitions, access route params via `params`:

```json
{
  "route": "/items/:id",
  "onMount": [
    {
      "type": "callApi",
      "api": "getItem",
      "args": { "id": "{{params.id}}" }
    }
  ]
}
```

### Building Dynamic Routes

```json
{
  "type": "navigate",
  "to": "/items/{{state.item.id}}/edit"
}
```

### Query Parameters

Include query params in the path:

```json
{
  "type": "navigate",
  "to": "/items?category={{state.selectedCategory}}&sort={{state.sortField}}"
}
```

## Best Practices

1. **Use replace for redirects** - Prevents back-button loops
2. **Navigate after success** - Wait for API completion
3. **Preserve history** - Don't use replace unnecessarily
4. **Use relative paths** - For plugin routes
