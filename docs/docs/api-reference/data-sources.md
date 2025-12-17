---
sidebar_position: 7
title: Data Sources
description: Data loading and management reference
---

## Data Sources

Reference for loading and managing data in Orbis plugins.

## API Data Sources

### Basic API Call

```json
{
  "onMount": [
    {
      "type": "callApi",
      "api": "getItems",
      "storeAs": "items"
    }
  ]
}
```

### With Parameters

```json
{
  "type": "callApi",
  "api": "getItems",
  "params": {
    "page": "{{state.currentPage}}",
    "limit": 20,
    "search": "{{state.searchQuery}}"
  },
  "storeAs": "items"
}
```

### HTTP Methods

```json
{ "type": "callApi", "api": "getItems", "method": "GET" }
{ "type": "callApi", "api": "createItem", "method": "POST", "params": { "data": "{{form.newItem}}" } }
{ "type": "callApi", "api": "updateItem", "method": "PUT", "params": { "id": "{{state.id}}", "data": "{{form.editItem}}" } }
{ "type": "callApi", "api": "deleteItem", "method": "DELETE", "params": { "id": "{{state.id}}" } }
```

## Response Handling

### Store Full Response

```json
{
  "type": "callApi",
  "api": "getItems",
  "storeAs": "data"
}
```

Access: `{{state.data}}`

### Extract Response Data

```json
{
  "type": "callApi",
  "api": "getItems",
  "onSuccess": [
    { "type": "updateState", "path": "items", "value": "{{$response.data}}" },
    { "type": "updateState", "path": "total", "value": "{{$response.total}}" }
  ]
}
```

### Transform Response

```json
{
  "type": "callApi",
  "api": "getUsers",
  "onSuccess": [
    {
      "type": "updateState",
      "path": "users",
      "value": "{{$response.map(u => ({ ...u, displayName: `${u.firstName} ${u.lastName}` }))}}"
    }
  ]
}
```

## Error Handling

### Basic Error Handling

```json
{
  "type": "callApi",
  "api": "getData",
  "onError": [
    { "type": "showToast", "message": "Failed to load data", "level": "error" }
  ]
}
```

### Detailed Error Handling

```json
{
  "type": "callApi",
  "api": "getData",
  "onError": [
    { "type": "updateState", "path": "error", "value": "{{$error}}" },
    { "type": "showToast", "message": "{{$error.message}}", "level": "error" }
  ]
}
```

### Retry Pattern

```json
{
  "type": "callApi",
  "api": "getData",
  "onError": [
    {
      "type": "conditional",
      "condition": "{{state.retryCount < 3}}",
      "then": [
        { "type": "updateState", "path": "retryCount", "value": "{{state.retryCount + 1}}" },
        { "type": "callApi", "api": "getData" }
      ],
      "else": [
        { "type": "showToast", "message": "Failed after 3 attempts", "level": "error" }
      ]
    }
  ]
}
```

## Loading States

### With Loading Indicator

```json
{
  "type": "callApi",
  "api": "getData",
  "onStart": [
    { "type": "setLoading", "key": "data", "value": true }
  ],
  "onComplete": [
    { "type": "setLoading", "key": "data", "value": false }
  ]
}
```

### UI with Loading

```json
{
  "type": "Conditional",
  "conditions": [
    {
      "when": "{{loading.data}}",
      "render": { "type": "Skeleton", "count": 5 }
    }
  ],
  "fallback": {
    "type": "Loop",
    "items": "{{state.items}}",
    "render": { "type": "Card", "children": [] }
  }
}
```

## Pagination

### Page-Based Pagination

```json
{
  "type": "callApi",
  "api": "getItems",
  "params": {
    "page": "{{state.page}}",
    "limit": "{{state.pageSize}}"
  },
  "onSuccess": [
    { "type": "updateState", "path": "items", "value": "{{$response.data}}" },
    { "type": "updateState", "path": "totalPages", "value": "{{$response.pages}}" }
  ]
}
```

### Cursor-Based Pagination

```json
{
  "type": "callApi",
  "api": "getItems",
  "params": {
    "cursor": "{{state.cursor}}",
    "limit": 20
  },
  "onSuccess": [
    { "type": "updateState", "path": "items", "value": "{{[...state.items, ...$response.data]}}" },
    { "type": "updateState", "path": "cursor", "value": "{{$response.nextCursor}}" },
    { "type": "updateState", "path": "hasMore", "value": "{{!!$response.nextCursor}}" }
  ]
}
```

### Infinite Scroll

```json
{
  "type": "Button",
  "text": "Load More",
  "visible": "{{state.hasMore}}",
  "loading": "{{loading.loadMore}}",
  "events": {
    "onClick": [
      { "type": "setLoading", "key": "loadMore", "value": true },
      {
        "type": "callApi",
        "api": "getItems",
        "params": { "cursor": "{{state.cursor}}" },
        "onSuccess": [
          { "type": "updateState", "path": "items", "value": "{{[...state.items, ...$response.data]}}" },
          { "type": "updateState", "path": "cursor", "value": "{{$response.nextCursor}}" }
        ],
        "onComplete": [
          { "type": "setLoading", "key": "loadMore", "value": false }
        ]
      }
    ]
  }
}
```

## Caching Strategies

### Store Response

```json
{
  "type": "callApi",
  "api": "getConfig",
  "storeAs": "config"
}
```

Data persists in state until page unmount.

### Conditional Fetch

```json
{
  "onMount": [
    {
      "type": "conditional",
      "condition": "{{!state.dataLoaded}}",
      "then": [
        { "type": "callApi", "api": "getData", "storeAs": "data" },
        { "type": "updateState", "path": "dataLoaded", "value": true }
      ]
    }
  ]
}
```

### Refresh Pattern

```json
{
  "type": "Button",
  "text": "Refresh",
  "icon": "RefreshCw",
  "events": {
    "onClick": [
      { "type": "setLoading", "key": "refresh", "value": true },
      {
        "type": "callApi",
        "api": "getData",
        "storeAs": "data",
        "onComplete": [
          { "type": "setLoading", "key": "refresh", "value": false }
        ]
      }
    ]
  }
}
```

## Static Data

### Initialize in State

```json
{
  "initialState": {
    "countries": [
      { "code": "US", "name": "United States" },
      { "code": "CA", "name": "Canada" },
      { "code": "UK", "name": "United Kingdom" }
    ],
    "statusOptions": ["active", "pending", "inactive"]
  }
}
```

### Use in Components

```json
{
  "type": "Field",
  "fieldType": "select",
  "name": "country",
  "options": "{{state.countries.map(c => ({ value: c.code, label: c.name }))}}"
}
```

## Computed Data

### Derived Values

```json
{
  "type": "Text",
  "text": "Total: ${{state.items.reduce((sum, i) => sum + i.price, 0).toFixed(2)}}"
}
```

### Filtered Lists

```json
{
  "type": "Loop",
  "items": "{{state.items.filter(i => i.status === state.filterStatus)}}"
}
```

### Sorted Data

```json
{
  "type": "Loop",
  "items": "{{state.items.sort((a, b) => state.sortOrder === 'asc' ? a.name.localeCompare(b.name) : b.name.localeCompare(a.name))}}"
}
```

### Grouped Data

```json
{
  "type": "Text",
  "text": "{{Object.keys(state.items.reduce((groups, item) => ({ ...groups, [item.category]: [...(groups[item.category] || []), item] }), {})).join(', ')}}"
}
```

## Data Relationships

### Master-Detail

```json
{
  "type": "Flex",
  "children": [
    {
      "type": "List",
      "items": "{{state.categories}}",
      "events": {
        "onSelect": [
          { "type": "updateState", "path": "selectedCategory", "value": "{{$value}}" },
          { "type": "callApi", "api": "getItems", "params": { "category": "{{$value.id}}" } }
        ]
      }
    },
    {
      "type": "Loop",
      "items": "{{state.items}}",
      "render": { "type": "Card", "children": [] }
    }
  ]
}
```

### Dependent Selects

```json
[
  {
    "type": "Field",
    "fieldType": "select",
    "name": "country",
    "options": "{{state.countries}}",
    "events": {
      "onChange": [
        { "type": "callApi", "api": "getStates", "params": { "country": "{{$value}}" }, "storeAs": "states" },
        { "type": "updateState", "path": "selectedState", "value": null }
      ]
    }
  },
  {
    "type": "Field",
    "fieldType": "select",
    "name": "state",
    "options": "{{state.states}}",
    "disabled": "{{!state.states?.length}}"
  }
]
```

## Best Practices

1. **Show loading states** - Always indicate data fetching
2. **Handle errors gracefully** - Display user-friendly messages
3. **Minimize requests** - Cache when appropriate
4. **Transform at source** - Process data in onSuccess
5. **Use pagination** - Avoid loading large datasets
6. **Clean state on unmount** - Prevent stale data
