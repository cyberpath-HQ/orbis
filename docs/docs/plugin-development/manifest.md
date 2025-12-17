---
sidebar_position: 2
title: Plugin Manifest
description: Complete reference for plugin manifest configuration
---

# Plugin Manifest

The manifest file (`manifest.json`) is the heart of every Orbis plugin. It defines metadata, pages, routes, permissions, and configuration.

## Basic Structure

```json
{
  "name": "my-plugin",
  "version": "1.0.0",
  "description": "My awesome plugin",
  "author": "Your Name",
  "homepage": "https://example.com",
  "license": "MIT",
  
  "min_orbis_version": "1.0.0",
  "dependencies": [],
  "permissions": [],
  
  "pages": [],
  "routes": [],
  
  "wasm_entry": "plugin.wasm",
  "config": {}
}
```

## Metadata Fields

### name (Required)

Unique plugin identifier.

```json
"name": "my-plugin"
```

**Rules:**
- Alphanumeric characters, hyphens, and underscores only
- Must be unique across all installed plugins
- Case-sensitive

### version (Required)

Semantic version string.

```json
"version": "1.0.0"
```

**Format:** `MAJOR.MINOR.PATCH` (e.g., `1.2.3`)

Optional pre-release: `1.0.0-beta.1`

### description

Human-readable description.

```json
"description": "A plugin that does awesome things"
```

### author

Plugin author or organization.

```json
"author": "Your Name <you@example.com>"
```

### homepage

URL to plugin documentation or homepage.

```json
"homepage": "https://github.com/user/my-plugin"
```

### license

SPDX license identifier.

```json
"license": "MIT"
```

Common values: `MIT`, `Apache-2.0`, `GPL-3.0`, `BSD-3-Clause`

## Compatibility

### min_orbis_version

Minimum required Orbis version.

```json
"min_orbis_version": "1.0.0"
```

Orbis will refuse to load plugins requiring a newer version.

## Dependencies

Other plugins this plugin requires.

```json
"dependencies": [
  {
    "name": "auth-plugin",
    "version": ">=1.0.0"
  },
  {
    "name": "data-plugin",
    "version": "^2.0.0"
  }
]
```

### Version Specifiers

| Specifier | Meaning |
|-----------|---------|
| `1.0.0` | Exact version |
| `>=1.0.0` | 1.0.0 or higher |
| `>1.0.0` | Greater than 1.0.0 |
| `<=1.0.0` | 1.0.0 or lower |
| `<1.0.0` | Less than 1.0.0 |
| `^1.0.0` | Compatible with 1.x.x |
| `~1.0.0` | Approximately 1.0.x |
| `*` | Any version |

## Permissions

Capabilities requested by the plugin.

```json
"permissions": [
  {
    "type": "network",
    "allowed_hosts": ["api.example.com"]
  },
  {
    "type": "storage",
    "scope": "plugin-data"
  }
]
```

### Permission Types

#### network

HTTP/HTTPS request permissions.

```json
{
  "type": "network",
  "allowed_hosts": [
    "api.example.com",
    "*.myservice.io",
    "https://secure.example.com"
  ]
}
```

Wildcards (`*`) match any subdomain.

#### storage

Database/storage access.

```json
{
  "type": "storage",
  "scope": "plugin-data",
  "read": true,
  "write": true
}
```

Scopes:
- `plugin-data` - Plugin's isolated storage
- `shared-read` - Read access to shared data
- `full` - Full database access (requires admin approval)

#### filesystem

Local file access.

```json
{
  "type": "filesystem",
  "paths": [
    { "path": "$HOME/Documents/MyApp", "access": "read-write" },
    { "path": "/tmp/my-plugin", "access": "read-write" }
  ]
}
```

Variables:
- `$HOME` - User's home directory
- `$TEMP` - Temporary directory
- `$PLUGIN` - Plugin's directory

#### ipc

Inter-plugin communication.

```json
{
  "type": "ipc",
  "allowed_plugins": ["auth-plugin", "data-plugin"]
}
```

#### notification

System notifications.

```json
{
  "type": "notification"
}
```

## Pages

UI pages exposed by the plugin.

```json
"pages": [
  {
    "id": "dashboard",
    "title": "Dashboard",
    "route": "/my-plugin",
    "icon": "LayoutDashboard",
    "state": {},
    "layout": {},
    "onMount": [],
    "onUnmount": []
  }
]
```

### Page Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | ✅ | Unique page ID |
| `title` | string | ✅ | Display title |
| `route` | string | ✅ | URL path |
| `icon` | string | ❌ | lucide-react icon name |
| `state` | object | ❌ | State definition |
| `layout` | object | ✅ | Root component schema |
| `onMount` | array | ❌ | Actions on page load |
| `onUnmount` | array | ❌ | Actions on page leave |

See [Page Definitions](./page-definitions) for full details.

## Routes

API routes for backend functionality.

```json
"routes": [
  {
    "path": "/api/items",
    "method": "GET",
    "handler": "get_items",
    "middleware": ["auth"]
  },
  {
    "path": "/api/items",
    "method": "POST",
    "handler": "create_item"
  },
  {
    "path": "/api/items/:id",
    "method": "GET",
    "handler": "get_item"
  },
  {
    "path": "/api/items/:id",
    "method": "PUT",
    "handler": "update_item"
  },
  {
    "path": "/api/items/:id",
    "method": "DELETE",
    "handler": "delete_item"
  }
]
```

### Route Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `path` | string | ✅ | Route path (supports `:param`) |
| `method` | string | ✅ | HTTP method |
| `handler` | string | ✅ | WASM function name |
| `middleware` | array | ❌ | Applied middleware |

### Path Parameters

```json
{
  "path": "/api/users/:userId/posts/:postId",
  "method": "GET",
  "handler": "get_user_post"
}
```

Parameters are passed to the handler function.

### Middleware

Built-in middleware:

| Name | Description |
|------|-------------|
| `auth` | Requires authenticated user |
| `admin` | Requires admin role |
| `rate-limit` | Applies rate limiting |

## WASM Entry

Path to the compiled WASM binary.

```json
"wasm_entry": "plugin.wasm"
```

Relative to the plugin directory or embedded in the WASM file.

## Configuration

Custom plugin configuration.

```json
"config": {
  "api_base_url": "https://api.example.com",
  "max_items": 100,
  "features": {
    "advanced_mode": true
  }
}
```

Configuration is accessible in WASM handlers and can be used in expressions:

```json
{
  "type": "Text",
  "content": "Max items: {{config.max_items}}"
}
```

## Complete Example

```json
{
  "name": "task-manager",
  "version": "2.1.0",
  "description": "A comprehensive task management plugin",
  "author": "Orbis Team",
  "homepage": "https://github.com/orbis/task-manager",
  "license": "MIT",
  
  "min_orbis_version": "1.0.0",
  
  "dependencies": [
    {
      "name": "auth-plugin",
      "version": ">=1.0.0"
    }
  ],
  
  "permissions": [
    {
      "type": "storage",
      "scope": "plugin-data"
    },
    {
      "type": "notification"
    }
  ],
  
  "pages": [
    {
      "id": "tasks",
      "title": "Tasks",
      "route": "/tasks",
      "icon": "CheckSquare",
      "state": {
        "tasks": { "type": "array", "default": [] },
        "filter": { "type": "string", "default": "all" }
      },
      "layout": {
        "type": "Container",
        "className": "p-6",
        "children": [
          {
            "type": "PageHeader",
            "title": "My Tasks",
            "subtitle": "Manage your daily tasks"
          }
        ]
      },
      "onMount": [
        {
          "type": "callApi",
          "api": "task-manager.get_tasks",
          "onSuccess": [
            { "type": "updateState", "path": "tasks", "value": "$response.data" }
          ]
        }
      ]
    }
  ],
  
  "routes": [
    {
      "path": "/api/tasks",
      "method": "GET",
      "handler": "get_tasks"
    },
    {
      "path": "/api/tasks",
      "method": "POST",
      "handler": "create_task"
    },
    {
      "path": "/api/tasks/:id",
      "method": "PUT",
      "handler": "update_task"
    },
    {
      "path": "/api/tasks/:id",
      "method": "DELETE",
      "handler": "delete_task"
    }
  ],
  
  "wasm_entry": "task_manager.wasm",
  
  "config": {
    "max_tasks": 1000,
    "enable_notifications": true
  }
}
```

## Validation

Orbis validates manifests at load time:

- Required fields are checked
- Version format is verified
- Routes are validated
- Permissions are checked against capability system

Invalid manifests produce detailed error messages.

## Best Practices

### Naming

- Use lowercase with hyphens: `my-awesome-plugin`
- Be descriptive but concise
- Avoid generic names like `plugin` or `app`

### Versioning

- Follow semantic versioning strictly
- Bump major version for breaking changes
- Document changes in CHANGELOG.md

### Permissions

- Request minimum necessary permissions
- Document why each permission is needed
- Users can see permission requests before installing

### Dependencies

- Keep dependencies minimal
- Use version ranges to allow updates
- Test with dependency updates

## Next Steps

- **[Page Definitions](./page-definitions)** - Detailed page configuration
- **[WASM Plugins](./wasm-plugins)** - Backend plugin development
- **[Building Plugins](./building-plugins)** - Build and packaging
