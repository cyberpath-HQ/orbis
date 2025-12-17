---
sidebar_position: 6
title: Event Handling
description: Responding to user interactions in Orbis
---

# Event Handling

Events connect user interactions to actions. When a user clicks a button, submits a form, or changes an input, events trigger action sequences that update state and interact with the system.

## Overview

Event handling in Orbis follows this pattern:

```mermaid
graph LR
    User[User Interaction] --> Event[Event Triggered]
    Event --> Actions[Action Sequence]
    Actions --> State[State Updates]
    State --> UI[UI Re-render]
```

## Event Handler Structure

Events are defined in the `events` property of components:

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

Each event handler is an array of actions that execute in sequence.

## Available Events

### Mouse Events

| Event | Triggered When | Components |
|-------|----------------|------------|
| `onClick` | Element is clicked | Most components |
| `onDoubleClick` | Element is double-clicked | Container, Card, etc. |
| `onMouseEnter` | Mouse enters element | Container, Card |
| `onMouseLeave` | Mouse leaves element | Container, Card |

### Form Events

| Event | Triggered When | Components |
|-------|----------------|------------|
| `onChange` | Input value changes | Field, Form inputs |
| `onSubmit` | Form is submitted | Form |
| `onFocus` | Input gains focus | Field |
| `onBlur` | Input loses focus | Field |

### Keyboard Events

| Event | Triggered When | Components |
|-------|----------------|------------|
| `onKeyDown` | Key is pressed | Input fields |
| `onKeyUp` | Key is released | Input fields |

### Data Events

| Event | Triggered When | Components |
|-------|----------------|------------|
| `onRowClick` | Table row is clicked | Table, List |
| `onRowDoubleClick` | Table row is double-clicked | Table |
| `onSelect` | Item is selected | Table, Select |
| `onClear` | Selection is cleared | Select |
| `onSearch` | Search query changes | Searchable components |
| `onPageChange` | Table page changes | Table |
| `onSortChange` | Table sort changes | Table |
| `onFilterChange` | Filter is applied | Table |

### Lifecycle Events

| Event | Triggered When | Components |
|-------|----------------|------------|
| `onLoad` | Component/data loads | Image, async components |
| `onError` | Error occurs | Image, async components |
| `onOpen` | Overlay opens | Modal, Dropdown |
| `onClose` | Overlay closes | Modal, Dropdown, Alert |

## Event Object

Event handlers receive context about the event through special variables.

### $event

The raw event object:

```json
{
  "events": {
    "onClick": [
      {
        "type": "updateState",
        "path": "lastClick",
        "value": "$event"
      }
    ]
  }
}
```

### $event.value

For input events, the current value:

```json
{
  "type": "Field",
  "name": "search",
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

### $event.target

The element that triggered the event:

```json
{
  "events": {
    "onFocus": [
      {
        "type": "updateState",
        "path": "focusedField",
        "value": "$event.target.name"
      }
    ]
  }
}
```

## Row/Item Context

When events occur within lists or tables, additional context is available:

### $row

Current table row data:

```json
{
  "type": "Table",
  "events": {
    "onRowClick": [
      {
        "type": "updateState",
        "path": "selectedUser",
        "value": "$row"
      },
      {
        "type": "navigate",
        "to": "/users/{{$row.id}}"
      }
    ]
  }
}
```

### $item

Current loop/list item:

```json
{
  "type": "Loop",
  "dataSource": "state:items",
  "template": {
    "type": "Button",
    "label": "Delete {{$item.name}}",
    "events": {
      "onClick": [
        {
          "type": "updateState",
          "path": "items",
          "mode": "remove",
          "value": "{{$item.id}}"
        }
      ]
    }
  }
}
```

### $index

Current iteration index:

```json
{
  "type": "Loop",
  "template": {
    "type": "Button",
    "label": "Item {{$index + 1}}",
    "events": {
      "onClick": [
        {
          "type": "updateState",
          "path": "selectedIndex",
          "value": "$index"
        }
      ]
    }
  }
}
```

## Action Sequences

Events execute actions in sequence. Actions can be:

### Independent Actions

Run in order, each completing before the next:

```json
{
  "onClick": [
    { "type": "updateState", "path": "step", "value": 1 },
    { "type": "updateState", "path": "step", "value": 2 },
    { "type": "showToast", "message": "Step is now 2" }
  ]
}
```

### Conditional Flow

Branch based on conditions:

```json
{
  "onClick": [
    {
      "type": "conditional",
      "condition": "{{state.isValid}}",
      "then": [
        { "type": "callApi", "api": "submit" }
      ],
      "else": [
        { "type": "showToast", "message": "Please fix errors", "level": "error" }
      ]
    }
  ]
}
```

### Async Actions

API calls with callbacks:

```json
{
  "onClick": [
    { "type": "setLoading", "loading": true },
    {
      "type": "callApi",
      "api": "my-plugin.saveData",
      "onSuccess": [
        { "type": "showToast", "message": "Saved!", "level": "success" }
      ],
      "onError": [
        { "type": "showToast", "message": "Failed: {{$error.message}}", "level": "error" }
      ]
    },
    { "type": "setLoading", "loading": false }
  ]
}
```

:::warning
The `setLoading` after `callApi` runs immediately, not after the API completes. Use the `onSuccess`/`onError` callbacks for post-API actions.
:::

## Component-Specific Events

### Button

```json
{
  "type": "Button",
  "label": "Submit",
  "events": {
    "onClick": [{ "type": "callApi", "api": "submit" }]
  }
}
```

### Field (Input)

```json
{
  "type": "Field",
  "name": "email",
  "fieldType": "email",
  "events": {
    "onChange": [{ "type": "updateState", "path": "email", "value": "$event.value" }],
    "onFocus": [{ "type": "updateState", "path": "focused", "value": "email" }],
    "onBlur": [{ "type": "validateForm", "formId": "my-form" }]
  }
}
```

### Form

```json
{
  "type": "Form",
  "id": "contact-form",
  "events": {
    "onSubmit": [
      { "type": "validateForm", "formId": "contact-form" },
      {
        "type": "conditional",
        "condition": "{{state.formValid}}",
        "then": [
          { "type": "callApi", "api": "submitContact" }
        ]
      }
    ]
  }
}
```

### Table

```json
{
  "type": "Table",
  "columns": [...],
  "dataSource": "state:users",
  "events": {
    "onRowClick": [
      { "type": "updateState", "path": "selectedUser", "value": "$row" }
    ],
    "onSelect": [
      { "type": "updateState", "path": "selectedUsers", "value": "$event.value" }
    ],
    "onPageChange": [
      { "type": "updateState", "path": "currentPage", "value": "$event.value" }
    ],
    "onSortChange": [
      {
        "type": "updateState",
        "path": "sort",
        "value": {
          "column": "$event.column",
          "direction": "$event.direction"
        }
      }
    ]
  }
}
```

### Modal

```json
{
  "type": "Modal",
  "id": "confirm-dialog",
  "events": {
    "onOpen": [
      { "type": "updateState", "path": "modalOpen", "value": true }
    ],
    "onClose": [
      { "type": "updateState", "path": "modalOpen", "value": false },
      { "type": "updateState", "path": "selectedItem", "value": null }
    ]
  }
}
```

## Debouncing

For events that fire rapidly (like typing), use debounced actions:

```json
{
  "type": "Field",
  "name": "search",
  "events": {
    "onChange": [
      {
        "type": "debouncedAction",
        "delay": 300,
        "key": "search",
        "action": {
          "type": "callApi",
          "api": "search",
          "args": { "query": "$event.value" }
        }
      }
    ]
  }
}
```

This waits 300ms after the last keystroke before executing.

## Event Bubbling

Events don't bubble by default. Each component handles its own events.

To handle parent clicks while ignoring child clicks:

```json
{
  "type": "Container",
  "events": {
    "onClick": [{ "type": "updateState", "path": "containerClicked", "value": true }]
  },
  "children": [
    {
      "type": "Button",
      "label": "Inner Button",
      "events": {
        "onClick": [
          // This runs, container's onClick does NOT run
          { "type": "updateState", "path": "buttonClicked", "value": true }
        ]
      }
    }
  ]
}
```

## Best Practices

### Keep Handlers Focused

```json
// ✅ Good - clear purpose
{
  "onClick": [
    { "type": "updateState", "path": "isOpen", "value": true }
  ]
}

// ❌ Avoid - too many concerns
{
  "onClick": [
    { "type": "updateState", "path": "isOpen", "value": true },
    { "type": "updateState", "path": "lastOpened", "value": "now" },
    { "type": "callApi", "api": "logEvent" },
    { "type": "showToast", "message": "Opened" }
  ]
}
```

### Use Descriptive State Paths

```json
// ✅ Good
{ "type": "updateState", "path": "form.isSubmitting", "value": true }

// ❌ Avoid
{ "type": "updateState", "path": "s", "value": true }
```

### Handle Loading and Errors

Always handle async operation states:

```json
{
  "onClick": [
    { "type": "setLoading", "target": "submit", "loading": true },
    {
      "type": "callApi",
      "api": "submit",
      "onSuccess": [
        { "type": "setLoading", "target": "submit", "loading": false },
        { "type": "showToast", "message": "Success!" }
      ],
      "onError": [
        { "type": "setLoading", "target": "submit", "loading": false },
        { "type": "showToast", "message": "Failed!", "level": "error" }
      ]
    }
  ]
}
```

## Debugging Events

### Log State Changes

Use temporary state to debug:

```json
{
  "onClick": [
    { "type": "updateState", "path": "debug.lastEvent", "value": "button clicked" },
    { "type": "updateState", "path": "debug.timestamp", "value": "{{Date.now()}}" }
  ]
}
```

### Development Logging

Enable debug mode:

```bash
RUST_LOG=debug,orbis=trace bun run tauri dev
```

## Next Steps

- **[Actions Overview](../actions/overview)** - All action types
- **[Update State](../actions/update-state)** - State modification actions
- **[Call API](../actions/call-api)** - Making API requests
