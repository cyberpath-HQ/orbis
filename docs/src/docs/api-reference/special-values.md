---
sidebar_position: 5
title: Special Values
description: Built-in variables and special values reference
---

## Special Values

Orbis provides several built-in variables and special values available in expressions.

## Event Variables

### $value

The current value in change events.

```json
{
  "type": "Field",
  "events": {
    "onChange": [
      { "type": "updateState", "path": "inputValue", "value": "{{$value}}" }
    ]
  }
}
```

**Available in:** `onChange`, `onSelect`, `onSort`, `onPageChange`, `onTabChange`

### $event

The full event object.

```json
{
  "events": {
    "onClick": [
      { "type": "updateState", "path": "clickX", "value": "{{$event.clientX}}" }
    ],
    "onKeyDown": [
      {
        "type": "conditional",
        "condition": "{{$event.key === 'Enter' && $event.ctrlKey}}",
        "then": [{ "type": "callApi", "api": "submit" }]
      }
    ]
  }
}
```

**Properties:**

| Property | Type | Description |
|----------|------|-------------|
| `$event.key` | string | Key pressed (keyboard events) |
| `$event.ctrlKey` | boolean | Ctrl modifier |
| `$event.shiftKey` | boolean | Shift modifier |
| `$event.altKey` | boolean | Alt modifier |
| `$event.metaKey` | boolean | Meta/Cmd modifier |
| `$event.clientX` | number | Mouse X (mouse events) |
| `$event.clientY` | number | Mouse Y (mouse events) |
| `$event.target` | object | Event target element |

## Loop Variables

### $item

Current item in a Loop component.

```json
{
  "type": "Loop",
  "items": "{{state.users}}",
  "render": {
    "type": "Text",
    "text": "Name: {{$item.name}}, Email: {{$item.email}}"
  }
}
```

### $index

Current index in a Loop component.

```json
{
  "type": "Loop",
  "items": "{{state.items}}",
  "render": {
    "type": "Text",
    "text": "{{$index + 1}}. {{$item.title}}"
  }
}
```

**Note:** Index is 0-based.

### Custom Loop Variables

Use `itemAs` and `indexAs` for custom names:

```json
{
  "type": "Loop",
  "items": "{{state.products}}",
  "itemAs": "product",
  "indexAs": "i",
  "render": {
    "type": "Text",
    "text": "{{i + 1}}. {{product.name}}"
  }
}
```

## API Response Variables

### $response

Full response from API calls.

```json
{
  "type": "callApi",
  "api": "getUsers",
  "onSuccess": [
    { "type": "updateState", "path": "users", "value": "{{$response.data}}" },
    { "type": "updateState", "path": "total", "value": "{{$response.total}}" }
  ]
}
```

**Common properties:**

| Property | Type | Description |
|----------|------|-------------|
| `$response` | any | Full response |
| `$response.data` | any | Response data |
| `$response.items` | array | List items |
| `$response.total` | number | Total count |
| `$response.page` | number | Current page |
| `$response.pages` | number | Total pages |

### $error

Error object from failed operations.

```json
{
  "type": "callApi",
  "api": "saveData",
  "onError": [
    { "type": "showToast", "message": "Error: {{$error.message}}", "level": "error" },
    { "type": "updateState", "path": "error", "value": "{{$error}}" }
  ]
}
```

**Properties:**

| Property | Type | Description |
|----------|------|-------------|
| `$error.message` | string | Error message |
| `$error.code` | string | Error code |
| `$error.status` | number | HTTP status |
| `$error.details` | object | Additional details |

## Navigation Variables

### params

Route parameters from dynamic routes.

```json
// Route: /users/:id
{
  "onMount": [
    { "type": "callApi", "api": "getUser", "params": { "id": "{{params.id}}" } }
  ]
}
```

### query

Query string parameters.

```json
// URL: /search?q=test&page=2
{
  "onMount": [
    { "type": "updateState", "path": "searchQuery", "value": "{{query.q}}" },
    { "type": "updateState", "path": "page", "value": "{{query.page || 1}}" }
  ]
}
```

### route

Current route information.

```json
"{{route.path}}"    // Current path
"{{route.hash}}"    // URL hash
```

## Time Variables

### $now

Current timestamp.

```json
{
  "type": "Text",
  "text": "Current time: {{$now}}"
}
```

```json
{
  "type": "download",
  "url": "/api/export",
  "filename": "export-{{$now}}.csv"
}
```

## Window Variables

### $window

Browser window object.

```json
{
  "type": "Conditional",
  "conditions": [
    {
      "when": "{{$window.innerWidth < 768}}",
      "render": { "type": "Text", "text": "Mobile view" }
    }
  ]
}
```

**Properties:**

| Property | Type | Description |
|----------|------|-------------|
| `$window.innerWidth` | number | Viewport width |
| `$window.innerHeight` | number | Viewport height |
| `$window.location.origin` | string | Origin URL |
| `$window.location.pathname` | string | Current path |
| `$window.location.href` | string | Full URL |

## Form Variables

### form

Form data namespace.

```json
"{{form.myFormId}}"              // Complete form data
"{{form.myFormId.fieldName}}"    // Specific field
"{{form.myFormId.$valid}}"       // Validation state
"{{form.myFormId.$dirty}}"       // Modified state
"{{form.myFormId.$errors}}"      // All errors
"{{form.myFormId.$errors.email}}" // Field error
```

## Loading Variables

### loading

Loading states namespace.

```json
"{{loading.submit}}"     // Submit loading
"{{loading.fetch}}"      // Fetch loading
"{{loading.keyName}}"    // Any loading key
```

## State Variables

### state

Plugin state namespace.

```json
"{{state.fieldName}}"        // Root field
"{{state.user.name}}"        // Nested field
"{{state.items[0]}}"         // Array index
"{{state.items.length}}"     // Array length
```

## Plugin Variables

### plugin

Plugin-specific variables.

```json
"{{plugin.id}}"           // Plugin ID
"{{plugin.version}}"      // Plugin version
"{{plugin.name}}"         // Plugin name
```

## Environment Variables

### env

Environment configuration.

```json
"{{env.API_URL}}"         // API URL
"{{env.DEBUG}}"           // Debug mode
```

## Usage Examples

### Complete Form Submit

```json
{
  "type": "Button",
  "text": "{{loading.submit ? 'Saving...' : 'Save'}}",
  "loading": "{{loading.submit}}",
  "disabled": "{{!form.userForm.$valid || loading.submit}}",
  "events": {
    "onClick": [
      { "type": "setLoading", "key": "submit", "value": true },
      {
        "type": "callApi",
        "api": "saveUser",
        "params": { "data": "{{form.userForm}}" },
        "onSuccess": [
          { "type": "showToast", "message": "Saved {{$response.name}}!", "level": "success" }
        ],
        "onError": [
          { "type": "showToast", "message": "{{$error.message}}", "level": "error" }
        ],
        "onComplete": [
          { "type": "setLoading", "key": "submit", "value": false }
        ]
      }
    ]
  }
}
```

### Dynamic List

```json
{
  "type": "Loop",
  "items": "{{state.users.filter(u => u.active)}}",
  "itemAs": "user",
  "indexAs": "idx",
  "render": {
    "type": "Card",
    "children": [
      { "type": "Text", "text": "{{idx + 1}}. {{user.name}}" },
      { "type": "Text", "text": "Email: {{user.email}}" },
      {
        "type": "Button",
        "text": "View",
        "events": {
          "onClick": [
            { "type": "navigate", "route": "/users/{{user.id}}" }
          ]
        }
      }
    ]
  }
}
```

### Responsive Layout

```json
{
  "type": "Conditional",
  "conditions": [
    {
      "when": "{{$window.innerWidth >= 1024}}",
      "render": { "type": "Flex", "direction": "row", "children": [] }
    },
    {
      "when": "{{$window.innerWidth >= 768}}",
      "render": { "type": "Flex", "direction": "row", "wrap": true, "children": [] }
    }
  ],
  "fallback": { "type": "Flex", "direction": "column", "children": [] }
}
```
