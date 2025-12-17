---
sidebar_position: 2
title: updateState
description: Modify page state values
---

# updateState

Modifies a value in the page state.

## Syntax

```json
{
  "type": "updateState",
  "path": "fieldName",
  "value": "newValue"
}
```

## Properties

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `path` | string | ✅ | Dot-notation path to state field |
| `value` | any | ✅ | New value (supports expressions) |

## Examples

### Simple Value

```json
{
  "type": "updateState",
  "path": "count",
  "value": 5
}
```

### Computed Value

```json
{
  "type": "updateState",
  "path": "count",
  "value": "{{state.count}} + 1"
}
```

### Nested Path

```json
{
  "type": "updateState",
  "path": "user.profile.name",
  "value": "John Doe"
}
```

### Array Operations

**Append to array:**
```json
{
  "type": "updateState",
  "path": "items",
  "value": "[...state.items, { id: Date.now(), name: 'New Item' }]"
}
```

**Filter array:**
```json
{
  "type": "updateState",
  "path": "items",
  "value": "{{state.items.filter(item => item.id !== state.selectedId)}}"
}
```

**Update array item:**
```json
{
  "type": "updateState",
  "path": "items",
  "value": "{{state.items.map(item => item.id === state.editId ? { ...item, name: state.newName } : item)}}"
}
```

### Object Operations

**Update object:**
```json
{
  "type": "updateState",
  "path": "settings",
  "value": "{{ ...state.settings, theme: 'dark' }}"
}
```

**Clear object:**
```json
{
  "type": "updateState",
  "path": "formData",
  "value": {}
}
```

### Boolean Toggle

```json
{
  "type": "updateState",
  "path": "isVisible",
  "value": "{{!state.isVisible}}"
}
```

### From Event Context

**From form field:**
```json
{
  "type": "Field",
  "name": "search",
  "events": {
    "onChange": [
      { "type": "updateState", "path": "searchQuery", "value": "{{$value}}" }
    ]
  }
}
```

**From table row:**
```json
{
  "type": "Table",
  "events": {
    "onRowClick": [
      { "type": "updateState", "path": "selectedItem", "value": "{{$row}}" }
    ]
  }
}
```

**From API response:**
```json
{
  "type": "callApi",
  "api": "getData",
  "onSuccess": [
    { "type": "updateState", "path": "data", "value": "$response.data" },
    { "type": "updateState", "path": "total", "value": "$response.meta.total" }
  ]
}
```

## Common Patterns

### Counter

```json
{
  "type": "Flex",
  "gap": "1rem",
  "align": "center",
  "children": [
    {
      "type": "Button",
      "label": "-",
      "events": {
        "onClick": [
          { "type": "updateState", "path": "count", "value": "{{state.count}} - 1" }
        ]
      }
    },
    { "type": "Text", "content": "{{state.count}}" },
    {
      "type": "Button",
      "label": "+",
      "events": {
        "onClick": [
          { "type": "updateState", "path": "count", "value": "{{state.count}} + 1" }
        ]
      }
    }
  ]
}
```

### Selection

```json
{
  "type": "List",
  "dataSource": "state:items",
  "itemTemplate": {
    "type": "Flex",
    "className": "p-2 cursor-pointer {{$item.id === state.selectedId ? 'bg-blue-100' : ''}}",
    "children": [
      { "type": "Text", "content": "{{$item.name}}" }
    ],
    "events": {
      "onClick": [
        { "type": "updateState", "path": "selectedId", "value": "{{$item.id}}" }
      ]
    }
  }
}
```

### Form Reset

```json
{
  "type": "Button",
  "label": "Reset",
  "events": {
    "onClick": [
      {
        "type": "updateState",
        "path": "formData",
        "value": {
          "name": "",
          "email": "",
          "message": ""
        }
      }
    ]
  }
}
```

### Pagination

```json
{
  "type": "Button",
  "label": "Next Page",
  "disabled": "{{state.currentPage >= state.totalPages}}",
  "events": {
    "onClick": [
      { "type": "updateState", "path": "currentPage", "value": "{{state.currentPage}} + 1" },
      { "type": "callApi", "api": "getItems", "args": { "page": "{{state.currentPage}} + 1" } }
    ]
  }
}
```

## State Path Syntax

### Root Level

```json
{ "path": "count", "value": 0 }
```

State: `{ count: 0 }`

### Nested Object

```json
{ "path": "user.profile.name", "value": "John" }
```

State: `{ user: { profile: { name: "John" } } }`

### Array Index

```json
{ "path": "items.0.name", "value": "First Item" }
```

State: `{ items: [{ name: "First Item" }] }`

## Notes

- Updates are reactive - components re-render automatically
- Nested paths create intermediate objects if they don't exist
- Use expressions for computed values
- Array mutations should create new arrays for reactivity
