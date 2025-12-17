---
sidebar_position: 1
title: Actions Overview
description: All available action types
---

# Actions Overview

Actions define what happens in response to events. They are the building blocks of interactivity in Orbis plugins.

## Action Structure

Every action has a `type` and type-specific properties:

```json
{
  "type": "updateState",
  "path": "count",
  "value": "{{state.count}} + 1"
}
```

## Action Types

| Type | Purpose |
|------|---------|
| [`updateState`](./update-state) | Modify page state |
| [`callApi`](./call-api) | Make API requests |
| [`navigate`](./navigate) | Change routes |
| [`showToast`](./show-toast) | Display notifications |
| [`showDialog`](./dialogs) | Show confirmation dialogs |
| [`closeDialog`](./dialogs) | Close dialogs |
| [`debouncedAction`](./flow-control) | Debounce actions |
| [`validateForm`](./form-actions) | Validate forms |
| [`resetForm`](./form-actions) | Reset form values |
| [`setLoading`](./utility-actions) | Set loading state |
| [`download`](./utility-actions) | Download files |
| [`copy`](./utility-actions) | Copy to clipboard |
| [`openUrl`](./utility-actions) | Open external URLs |
| [`emit`](./utility-actions) | Emit custom events |
| [`conditional`](./flow-control) | Conditional execution |
| [`sequence`](./flow-control) | Sequential execution |

## Using Actions

### In Event Handlers

```json
{
  "type": "Button",
  "label": "Click Me",
  "events": {
    "onClick": [
      { "type": "updateState", "path": "clicked", "value": true },
      { "type": "showToast", "message": "Button clicked!" }
    ]
  }
}
```

### In Lifecycle Hooks

```json
{
  "onMount": [
    { "type": "callApi", "api": "getData" }
  ],
  "onUnmount": [
    { "type": "updateState", "path": "selectedItem", "value": null }
  ]
}
```

### Chaining Actions

Actions execute in sequence:

```json
{
  "events": {
    "onClick": [
      { "type": "setLoading", "loading": true },
      { "type": "callApi", "api": "save" },
      { "type": "showToast", "message": "Saved!" },
      { "type": "setLoading", "loading": false }
    ]
  }
}
```

## Event Context

Actions have access to event context via special variables:

| Variable | Context | Description |
|----------|---------|-------------|
| `$event` | All events | Raw event object |
| `$value` | onChange | New field value |
| `$row` | Table onRowClick | Row data |
| `$index` | List item events | Item index |
| `$item` | List item events | Item data |
| `$response` | callApi onSuccess | API response |
| `$error` | callApi onError | Error object |
| `$page` | onPageChange | Page number |

### Example

```json
{
  "type": "Table",
  "dataSource": "state:items",
  "events": {
    "onRowClick": [
      { "type": "updateState", "path": "selectedId", "value": "{{$row.id}}" },
      { "type": "navigate", "to": "/items/{{$row.id}}" }
    ]
  }
}
```

## Error Handling

### With callApi

```json
{
  "type": "callApi",
  "api": "saveData",
  "onSuccess": [
    { "type": "showToast", "message": "Saved!", "level": "success" }
  ],
  "onError": [
    { "type": "showToast", "message": "Error: {{$error.message}}", "level": "error" }
  ]
}
```

### Fallback Pattern

```json
{
  "type": "callApi",
  "api": "primaryApi",
  "onError": [
    {
      "type": "callApi",
      "api": "fallbackApi",
      "onError": [
        { "type": "showToast", "message": "All APIs failed", "level": "error" }
      ]
    }
  ]
}
```

## Conditional Actions

Execute actions based on conditions:

```json
{
  "type": "conditional",
  "condition": "{{state.isValid}}",
  "then": [
    { "type": "callApi", "api": "submit" }
  ],
  "else": [
    { "type": "showToast", "message": "Please fix errors", "level": "warning" }
  ]
}
```

## Common Patterns

### Loading State

```json
[
  { "type": "setLoading", "loading": true },
  {
    "type": "callApi",
    "api": "fetchData",
    "onSuccess": [
      { "type": "updateState", "path": "data", "value": "$response.data" }
    ],
    "finally": [
      { "type": "setLoading", "loading": false }
    ]
  }
]
```

### Optimistic Update

```json
[
  { "type": "updateState", "path": "items", "value": "{{state.items.filter(i => i.id !== $itemId)}}" },
  {
    "type": "callApi",
    "api": "deleteItem",
    "onError": [
      { "type": "callApi", "api": "getItems" },
      { "type": "showToast", "message": "Failed to delete", "level": "error" }
    ]
  }
]
```

### Form Submission

```json
[
  { "type": "validateForm", "form": "myForm" },
  {
    "type": "conditional",
    "condition": "{{state.$formValid}}",
    "then": [
      { "type": "setLoading", "loading": true },
      {
        "type": "callApi",
        "api": "submitForm",
        "args": "{{state.formData}}",
        "onSuccess": [
          { "type": "showToast", "message": "Submitted!", "level": "success" },
          { "type": "resetForm", "form": "myForm" },
          { "type": "navigate", "to": "/success" }
        ]
      },
      { "type": "setLoading", "loading": false }
    ]
  }
]
```

## Next Steps

Explore each action type in detail:

- [Update State](./update-state)
- [Call API](./call-api)
- [Navigate](./navigate)
- [Show Toast](./show-toast)
- [Dialogs](./dialogs)
- [Form Actions](./form-actions)
- [Utility Actions](./utility-actions)
- [Flow Control](./flow-control)
