---
sidebar_position: 7
title: Best Practices
description: Production-ready plugin development guidelines
---

# Best Practices

Guidelines for building production-ready Orbis plugins.

## Code Organization

### Project Structure

```
my-plugin/
├── Cargo.toml
├── manifest.json
├── README.md
├── CHANGELOG.md
├── src/
│   ├── lib.rs          # Entry points
│   ├── handlers.rs     # Route handlers
│   ├── models.rs       # Data structures
│   └── utils.rs        # Helpers
└── tests/
    ├── unit.rs
    └── integration.rs
```

### Module Separation

```rust
// src/lib.rs - Keep minimal
mod handlers;
mod models;
mod utils;

#[no_mangle]
pub extern "C" fn init() -> i32 {
    handlers::initialize()
}

#[no_mangle]
pub extern "C" fn execute(ptr: i32, len: i32) -> i32 {
    handlers::dispatch(ptr, len)
}
```

```rust
// src/handlers.rs - Route handling
use crate::models::*;

pub fn initialize() -> i32 { 0 }

pub fn dispatch(ptr: i32, len: i32) -> i32 {
    match parse_request(ptr, len) {
        Ok(req) => handle_route(&req),
        Err(_) => error_response("Invalid request"),
    }
}

fn handle_route(req: &Request) -> i32 {
    match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/items") => get_items(),
        ("POST", "/items") => create_item(&req.body),
        _ => not_found(),
    }
}
```

## Schema Design

### State Organization

```json
{
  "state": {
    "data": {
      "type": "object",
      "default": {
        "items": [],
        "selectedItem": null
      }
    },
    "ui": {
      "type": "object", 
      "default": {
        "isLoading": false,
        "activeTab": "all",
        "searchQuery": ""
      }
    },
    "form": {
      "type": "object",
      "default": {
        "name": "",
        "description": ""
      }
    },
    "errors": {
      "type": "object",
      "default": {}
    }
  }
}
```

### Naming Conventions

| Element | Convention | Example |
|---------|------------|---------|
| Page IDs | `kebab-case` | `item-list` |
| State paths | `camelCase` | `selectedItem` |
| Route paths | `kebab-case` | `/my-plugin/items` |
| Action types | `camelCase` | `updateState` |
| Component types | `PascalCase` | `StatCard` |

### Reusable Layouts

Define common patterns:

```json
{
  "templates": {
    "pageWrapper": {
      "type": "Container",
      "className": "p-6 max-w-7xl mx-auto",
      "children": []
    },
    "cardGrid": {
      "type": "Grid",
      "columns": { "sm": 1, "md": 2, "lg": 3 },
      "gap": "1rem"
    }
  }
}
```

## Performance

### Minimize State Updates

```json
{
  "events": {
    "onClick": [
      {
        "type": "updateState",
        "path": "ui.selectedId",
        "value": "{{$row.id}}"
      }
    ]
  }
}
```

❌ Avoid updating entire objects:
```json
{
  "type": "updateState",
  "path": "data",
  "value": { "...all data plus changes..." }
}
```

### Debounce Expensive Operations

```json
{
  "fieldType": "text",
  "events": {
    "onChange": [
      {
        "type": "debouncedAction",
        "delay": 300,
        "action": {
          "type": "callApi",
          "api": "search",
          "args": { "query": "{{state.searchQuery}}" }
        }
      }
    ]
  }
}
```

### Lazy Loading

Load data only when needed:

```json
{
  "type": "Tabs",
  "tabs": [
    { "id": "overview", "label": "Overview" },
    { "id": "details", "label": "Details" }
  ],
  "events": {
    "onTabChange": [
      {
        "type": "conditional",
        "condition": "{{$value}} === 'details' && !state.detailsLoaded",
        "then": [
          { "type": "callApi", "api": "getDetails" },
          { "type": "updateState", "path": "detailsLoaded", "value": true }
        ]
      }
    ]
  }
}
```

### Optimize Lists

For large lists, use pagination:

```json
{
  "type": "Table",
  "dataSource": "state:items",
  "pageSize": 25,
  "serverPagination": true,
  "events": {
    "onPageChange": [
      {
        "type": "callApi",
        "api": "getItems",
        "args": { 
          "page": "{{$page}}",
          "pageSize": 25
        }
      }
    ]
  }
}
```

## Error Handling

### Graceful Degradation

```json
{
  "onMount": [
    {
      "type": "callApi",
      "api": "getData",
      "onSuccess": [
        { "type": "updateState", "path": "data", "value": "$response.data" },
        { "type": "updateState", "path": "ui.loadError", "value": null }
      ],
      "onError": [
        { "type": "updateState", "path": "ui.loadError", "value": "$error.message" }
      ]
    }
  ]
}
```

```json
{
  "type": "Conditional",
  "condition": "{{state.ui.loadError}}",
  "then": {
    "type": "Alert",
    "variant": "destructive",
    "title": "Failed to Load",
    "message": "{{state.ui.loadError}}",
    "action": {
      "type": "Button",
      "label": "Retry",
      "events": { "onClick": [{ "type": "callApi", "api": "getData" }] }
    }
  },
  "else": { "...normal content..." }
}
```

### Validation Feedback

```json
{
  "type": "Form",
  "fields": [
    {
      "name": "email",
      "fieldType": "text",
      "label": "Email",
      "validation": {
        "required": { "message": "Email is required" },
        "email": { "message": "Please enter a valid email" }
      }
    }
  ]
}
```

### Error Boundaries

Your plugin is automatically wrapped in error boundaries. If a component crashes, only that section fails, not the entire app.

## Security

### Input Validation

Always validate in backend routes:

```rust
fn create_item(body: &str) -> i32 {
    let input: CreateItemInput = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(_) => return error_response("Invalid input"),
    };
    
    // Validate fields
    if input.name.trim().is_empty() {
        return error_response("Name is required");
    }
    
    if input.name.len() > 100 {
        return error_response("Name too long");
    }
    
    // Sanitize
    let name = sanitize_string(&input.name);
    
    // Process...
}
```

### Permission Scoping

Request minimal permissions:

```json
{
  "permissions": ["database:read"]
}
```

❌ Avoid over-permissioning:
```json
{
  "permissions": ["database:read", "database:write", "filesystem", "network"]
}
```

### Secure API Calls

```json
{
  "type": "callApi",
  "api": "my-plugin.sensitiveAction",
  "args": {
    "id": "{{state.selectedId}}"
  }
}
```

Never expose secrets in state or expressions.

## User Experience

### Loading States

Always show loading feedback:

```json
{
  "onMount": [
    { "type": "setLoading", "loading": true },
    {
      "type": "callApi",
      "api": "getData",
      "onSuccess": [
        { "type": "updateState", "path": "data", "value": "$response.data" }
      ],
      "finally": [
        { "type": "setLoading", "loading": false }
      ]
    }
  ]
}
```

```json
{
  "type": "Conditional",
  "condition": "{{state.$loading}}",
  "then": { "type": "LoadingOverlay" }
}
```

### Empty States

Handle empty data gracefully:

```json
{
  "type": "Conditional",
  "condition": "{{state.items.length}} === 0",
  "then": {
    "type": "EmptyState",
    "icon": "FileText",
    "title": "No Items Yet",
    "description": "Create your first item to get started.",
    "action": {
      "type": "Button",
      "label": "Create Item",
      "events": { "onClick": [{ "type": "navigate", "to": "/items/new" }] }
    }
  },
  "else": { "type": "Table", "dataSource": "state:items" }
}
```

### Feedback Actions

Confirm user actions:

```json
{
  "events": {
    "onClick": [
      {
        "type": "callApi",
        "api": "saveItem",
        "onSuccess": [
          { "type": "showToast", "message": "Saved successfully!", "level": "success" }
        ],
        "onError": [
          { "type": "showToast", "message": "Failed to save: {{$error.message}}", "level": "error" }
        ]
      }
    ]
  }
}
```

### Confirm Destructive Actions

```json
{
  "type": "Button",
  "label": "Delete",
  "variant": "destructive",
  "events": {
    "onClick": [
      {
        "type": "showDialog",
        "title": "Delete Item?",
        "content": "This action cannot be undone.",
        "variant": "destructive",
        "confirmText": "Delete",
        "onConfirm": [
          { "type": "callApi", "api": "deleteItem" },
          { "type": "showToast", "message": "Item deleted", "level": "success" },
          { "type": "navigate", "to": "/items" }
        ]
      }
    ]
  }
}
```

## Accessibility

### Semantic Structure

```json
{
  "type": "Section",
  "ariaLabel": "User Statistics",
  "children": [
    { "type": "Heading", "level": 2, "text": "Statistics" },
    { "...content..." }
  ]
}
```

### Form Labels

Always provide labels:

```json
{
  "name": "email",
  "fieldType": "text",
  "label": "Email Address",
  "placeholder": "you@example.com",
  "required": true
}
```

### Focus Management

```json
{
  "type": "Modal",
  "autoFocus": true,
  "returnFocus": true,
  "events": {
    "onClose": [{ "type": "updateState", "path": "modalOpen", "value": false }]
  }
}
```

### Keyboard Navigation

Ensure interactive elements are keyboard accessible:

```json
{
  "type": "Button",
  "label": "Action",
  "ariaLabel": "Perform main action",
  "events": {
    "onClick": [{ "...actions..." }]
  }
}
```

## Maintenance

### Versioning

Follow semantic versioning:

```json
{
  "version": "1.2.3"
}
```

- **Major (1.x.x)**: Breaking changes
- **Minor (x.1.x)**: New features
- **Patch (x.x.1)**: Bug fixes

### Changelog

Keep a changelog:

```markdown
# Changelog

## [1.2.0] - 2024-01-15
### Added
- New dashboard statistics cards
- Export functionality

### Fixed
- Form validation edge case

## [1.1.0] - 2024-01-01
### Added
- Initial release
```

### Documentation

Document your plugin:

```markdown
# My Plugin

## Features
- Feature 1
- Feature 2

## Installation
Copy `my_plugin.wasm` to plugins directory.

## Configuration
| Setting | Default | Description |
|---------|---------|-------------|
| `api_url` | `""` | Backend API URL |

## Usage
Navigate to `/my-plugin` to access the dashboard.
```

## Checklist

### Before Release

- [ ] All tests passing
- [ ] Manifest valid JSON
- [ ] Version number updated
- [ ] Changelog updated
- [ ] README updated
- [ ] Error states handled
- [ ] Loading states visible
- [ ] Empty states provided
- [ ] Validation complete
- [ ] WASM optimized (`wasm-opt`)
- [ ] Debug code removed

### Quality Gates

| Check | Threshold |
|-------|-----------|
| Test coverage | > 80% |
| WASM size | < 200KB |
| Load time | < 100ms |
| No console errors | 0 |

## Anti-Patterns

### Avoid

❌ **Huge state objects** - Split into logical groups

❌ **Inline styles** - Use className with Tailwind

❌ **Polling for updates** - Use subscriptions when available

❌ **Nested callbacks** - Use action sequences

❌ **Hardcoded strings** - Use state or config

❌ **Missing error handling** - Always handle onError

### Prefer

✅ **Organized state** - `data`, `ui`, `form` groups

✅ **Utility classes** - Consistent styling

✅ **Debounced updates** - For search/filtering

✅ **Action sequences** - Clear flow control

✅ **Configurable values** - Via plugin config

✅ **Graceful degradation** - Show errors, allow retry

## Next Steps

- **[Components](../components/overview)** - UI component reference
- **[Actions](../actions/overview)** - Action reference
- **[API Reference](../api-reference/state-management)** - Technical details
