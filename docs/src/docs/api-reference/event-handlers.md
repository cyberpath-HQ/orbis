---
sidebar_position: 4
title: Event Handlers
description: Complete event handling reference
---

## Event Handlers

Events connect user interactions to actions. This reference covers all available events.

## Event Syntax

Events are defined in the `events` property:

```json
{
  "type": "Button",
  "text": "Click me",
  "events": {
    "onClick": [
      { "type": "showToast", "message": "Clicked!" }
    ]
  }
}
```

## Mouse Events

### onClick

Triggered when element is clicked.

```json
{
  "events": {
    "onClick": [
      { "type": "updateState", "path": "clicked", "value": true }
    ]
  }
}
```

**Context variables:**

- `$event` - Mouse event object

### onDoubleClick

Triggered on double-click.

```json
{
  "events": {
    "onDoubleClick": [
      { "type": "navigate", "route": "/edit/{{state.item.id}}" }
    ]
  }
}
```

### onMouseEnter / onMouseLeave

Triggered when mouse enters/leaves element.

```json
{
  "events": {
    "onMouseEnter": [
      { "type": "updateState", "path": "hovering", "value": true }
    ],
    "onMouseLeave": [
      { "type": "updateState", "path": "hovering", "value": false }
    ]
  }
}
```

## Form Events

### onChange

Triggered when input value changes.

```json
{
  "type": "Field",
  "fieldType": "text",
  "name": "search",
  "events": {
    "onChange": [
      { "type": "updateState", "path": "searchQuery", "value": "{{$value}}" }
    ]
  }
}
```

**Context variables:**

- `$value` - New input value
- `$event` - Change event object

### onBlur

Triggered when input loses focus.

```json
{
  "events": {
    "onBlur": [
      { "type": "validateForm", "formId": "myForm" }
    ]
  }
}
```

### onFocus

Triggered when input receives focus.

```json
{
  "events": {
    "onFocus": [
      { "type": "updateState", "path": "inputFocused", "value": true }
    ]
  }
}
```

### onSubmit

Triggered when form is submitted.

```json
{
  "type": "Form",
  "id": "contactForm",
  "events": {
    "onSubmit": [
      { "type": "validateForm", "formId": "contactForm" },
      { "type": "callApi", "api": "submitContact" }
    ]
  }
}
```

**Context variables:**

- `$event` - Submit event object
- Form data available via `form.formId`

## Keyboard Events

### onKeyDown

Triggered when key is pressed.

```json
{
  "events": {
    "onKeyDown": [
      {
        "type": "conditional",
        "condition": "{{$event.key === 'Enter'}}",
        "then": [
          { "type": "callApi", "api": "submit" }
        ]
      }
    ]
  }
}
```

**Context variables:**

- `$event.key` - Key pressed
- `$event.ctrlKey` - Ctrl modifier
- `$event.shiftKey` - Shift modifier
- `$event.altKey` - Alt modifier
- `$event.metaKey` - Meta modifier

### onKeyUp

Triggered when key is released.

```json
{
  "events": {
    "onKeyUp": [
      { "type": "updateState", "path": "lastKey", "value": "{{$event.key}}" }
    ]
  }
}
```

## Lifecycle Events

### onMount

Triggered when page/component mounts.

```json
{
  "onMount": [
    { "type": "callApi", "api": "fetchInitialData", "storeAs": "data" }
  ]
}
```

Common uses:

- Fetch initial data
- Initialize state
- Set up subscriptions

### onUnmount

Triggered when page/component unmounts.

```json
{
  "onUnmount": [
    { "type": "updateState", "path": "cleanup", "value": true }
  ]
}
```

Common uses:

- Clean up resources
- Save state
- Cancel pending operations

## Selection Events

### onSelect

Triggered when selection changes.

```json
{
  "type": "Table",
  "events": {
    "onSelect": [
      { "type": "updateState", "path": "selectedRows", "value": "{{$value}}" }
    ]
  }
}
```

**Context variables:**

- `$value` - Selected items

## Custom Events

### onEvent:[name]

Listen for custom events.

```json
{
  "events": {
    "onEvent:itemSelected": [
      { "type": "updateState", "path": "selected", "value": "{{$event.data}}" }
    ]
  }
}
```

Emit custom events:

```json
{
  "type": "emit",
  "event": "itemSelected",
  "data": "{{state.currentItem}}"
}
```

## Table/List Events

### onRowClick

Triggered when table row is clicked.

```json
{
  "type": "Table",
  "events": {
    "onRowClick": [
      { "type": "updateState", "path": "selectedItem", "value": "{{$item}}" }
    ]
  }
}
```

**Context variables:**

- `$item` - Row data
- `$index` - Row index

### onSort

Triggered when table sort changes.

```json
{
  "type": "Table",
  "events": {
    "onSort": [
      { "type": "updateState", "path": "sortBy", "value": "{{$value.column}}" },
      { "type": "updateState", "path": "sortOrder", "value": "{{$value.order}}" }
    ]
  }
}
```

### onPageChange

Triggered when pagination changes.

```json
{
  "type": "Table",
  "events": {
    "onPageChange": [
      { "type": "updateState", "path": "page", "value": "{{$value}}" },
      { "type": "callApi", "api": "fetchPage" }
    ]
  }
}
```

## Tab Events

### onTabChange

Triggered when active tab changes.

```json
{
  "type": "Tabs",
  "events": {
    "onTabChange": [
      { "type": "updateState", "path": "activeTab", "value": "{{$value}}" }
    ]
  }
}
```

## Modal Events

### onClose

Triggered when modal is closed.

```json
{
  "type": "Modal",
  "events": {
    "onClose": [
      { "type": "resetForm", "formId": "editForm" },
      { "type": "updateState", "path": "modalOpen", "value": false }
    ]
  }
}
```

## Event Context Summary

| Variable | Type | Available In | Description |
|----------|------|--------------|-------------|
| `$value` | any | onChange, onSelect | Current value |
| `$event` | object | All events | Event object |
| `$item` | any | Loop, Table events | Current item |
| `$index` | number | Loop, Table events | Current index |

## Multiple Handlers

Execute multiple actions:

```json
{
  "events": {
    "onClick": [
      { "type": "setLoading", "key": "submit", "value": true },
      { "type": "callApi", "api": "submit" },
      { "type": "setLoading", "key": "submit", "value": false },
      { "type": "showToast", "message": "Done!" }
    ]
  }
}
```

## Conditional Event Handling

```json
{
  "events": {
    "onClick": [
      {
        "type": "conditional",
        "condition": "{{state.canSubmit}}",
        "then": [
          { "type": "callApi", "api": "submit" }
        ],
        "else": [
          { "type": "showToast", "message": "Cannot submit", "level": "warning" }
        ]
      }
    ]
  }
}
```

## Best Practices

1. **Keep handlers focused** - One purpose per handler
2. **Use loading states** - Show progress for async operations
3. **Handle errors** - Provide user feedback
4. **Debounce expensive operations** - Use `debouncedAction`
5. **Clean up on unmount** - Prevent memory leaks
