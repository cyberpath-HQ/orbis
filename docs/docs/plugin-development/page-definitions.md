---
sidebar_position: 4
title: Page Definitions
description: Configuring UI pages in plugins
---

# Page Definitions

Pages are the primary way plugins provide user interfaces. Each page has its own route, state, and layout.

## Page Structure

```json
{
  "pages": [
    {
      "id": "my-page",
      "title": "My Page",
      "route": "/my-plugin/page",
      "icon": "FileText",
      "state": { ... },
      "layout": { ... },
      "onMount": [ ... ],
      "onUnmount": [ ... ]
    }
  ]
}
```

## Required Fields

### id

Unique identifier within the plugin.

```json
"id": "dashboard"
```

Used for internal references and debugging.

### title

Display title shown in navigation.

```json
"title": "Dashboard"
```

Supports expressions:
```json
"title": "{{state.userName}}'s Dashboard"
```

### route

URL path for the page.

```json
"route": "/my-plugin/dashboard"
```

#### Route Patterns

| Pattern | Example | Description |
|---------|---------|-------------|
| Static | `/my-plugin` | Fixed path |
| Nested | `/my-plugin/settings` | Nested path |
| Parameter | `/my-plugin/items/:id` | Dynamic segment |
| Catch-all | `/my-plugin/*` | Match remaining path |

#### Route Parameters

Access in expressions:
```json
"route": "/items/:itemId",
"layout": {
  "type": "Text",
  "content": "Viewing item: {{params.itemId}}"
}
```

### layout

Root component schema defining the UI.

```json
"layout": {
  "type": "Container",
  "className": "p-6",
  "children": [
    { "type": "Heading", "text": "Welcome" }
  ]
}
```

See [Components](../components/overview) for all available components.

## Optional Fields

### icon

Icon displayed in navigation (from lucide-react).

```json
"icon": "LayoutDashboard"
```

Common icons:
- `Home`, `Settings`, `User`, `Users`
- `FileText`, `FolderOpen`, `Database`
- `ChartBar`, `Activity`, `Bell`
- `Plus`, `Edit`, `Trash`, `Search`

Browse all at [lucide.dev](https://lucide.dev/icons).

### state

Page state definition.

```json
"state": {
  "items": {
    "type": "array",
    "default": []
  },
  "selectedItem": {
    "type": "object",
    "default": null,
    "nullable": true
  },
  "isLoading": {
    "type": "boolean",
    "default": false
  },
  "searchQuery": {
    "type": "string",
    "default": ""
  },
  "count": {
    "type": "number",
    "default": 0
  }
}
```

#### State Field Types

| Type | Description | Default |
|------|-------------|---------|
| `string` | Text value | `""` |
| `number` | Numeric value | `0` |
| `boolean` | True/false | `false` |
| `object` | Key-value map | `{}` |
| `array` | List of items | `[]` |

### onMount

Actions to execute when page loads.

```json
"onMount": [
  {
    "type": "setLoading",
    "loading": true
  },
  {
    "type": "callApi",
    "api": "my-plugin.getData",
    "onSuccess": [
      { "type": "updateState", "path": "items", "value": "$response.data" },
      { "type": "setLoading", "loading": false }
    ],
    "onError": [
      { "type": "showToast", "message": "Failed to load data", "level": "error" },
      { "type": "setLoading", "loading": false }
    ]
  }
]
```

Common patterns:
- Fetch initial data
- Initialize from URL parameters
- Set up subscriptions

### onUnmount

Actions to execute when leaving the page.

```json
"onUnmount": [
  {
    "type": "updateState",
    "path": "selectedItem",
    "value": null
  }
]
```

Common patterns:
- Clear selections
- Cancel pending requests
- Save draft data

## Layout Patterns

### Simple Page

```json
{
  "layout": {
    "type": "Container",
    "className": "p-6 max-w-4xl mx-auto",
    "children": [
      {
        "type": "PageHeader",
        "title": "My Page",
        "subtitle": "Page description here"
      },
      {
        "type": "Card",
        "content": {
          "type": "Text",
          "content": "Page content goes here"
        }
      }
    ]
  }
}
```

### Dashboard Layout

```json
{
  "layout": {
    "type": "Container",
    "className": "p-6",
    "children": [
      {
        "type": "PageHeader",
        "title": "Dashboard",
        "actions": [
          { "type": "Button", "label": "New Item", "icon": "Plus" }
        ]
      },
      {
        "type": "Grid",
        "columns": { "sm": 1, "md": 2, "lg": 4 },
        "gap": "1rem",
        "className": "mb-6",
        "children": [
          { "type": "StatCard", "title": "Total Users", "value": "{{state.stats.users}}" },
          { "type": "StatCard", "title": "Active Sessions", "value": "{{state.stats.sessions}}" },
          { "type": "StatCard", "title": "Revenue", "value": "${{state.stats.revenue}}" },
          { "type": "StatCard", "title": "Growth", "value": "{{state.stats.growth}}%", "changeType": "increase" }
        ]
      },
      {
        "type": "Grid",
        "columns": { "sm": 1, "lg": 2 },
        "gap": "1rem",
        "children": [
          {
            "type": "Card",
            "title": "Recent Activity",
            "content": { "type": "List", "dataSource": "state:recentActivity" }
          },
          {
            "type": "Card",
            "title": "Performance",
            "content": { "type": "Chart", "chartType": "line", "dataSource": "state:performanceData" }
          }
        ]
      }
    ]
  }
}
```

### List/Detail Layout

**List Page:**
```json
{
  "id": "items-list",
  "route": "/items",
  "layout": {
    "type": "Container",
    "children": [
      {
        "type": "PageHeader",
        "title": "Items",
        "actions": [
          {
            "type": "Button",
            "label": "New Item",
            "events": {
              "onClick": [{ "type": "navigate", "to": "/items/new" }]
            }
          }
        ]
      },
      {
        "type": "Table",
        "dataSource": "state:items",
        "columns": [
          { "key": "name", "label": "Name" },
          { "key": "status", "label": "Status" },
          { "key": "createdAt", "label": "Created" }
        ],
        "events": {
          "onRowClick": [
            { "type": "navigate", "to": "/items/{{$row.id}}" }
          ]
        }
      }
    ]
  }
}
```

**Detail Page:**
```json
{
  "id": "item-detail",
  "route": "/items/:id",
  "state": {
    "item": { "type": "object", "default": null }
  },
  "onMount": [
    {
      "type": "callApi",
      "api": "my-plugin.getItem",
      "args": { "id": "{{params.id}}" },
      "onSuccess": [
        { "type": "updateState", "path": "item", "value": "$response.data" }
      ]
    }
  ],
  "layout": {
    "type": "Container",
    "children": [
      {
        "type": "PageHeader",
        "title": "{{state.item.name}}",
        "backLink": "/items"
      },
      {
        "type": "Conditional",
        "condition": "{{state.item}}",
        "then": {
          "type": "Card",
          "content": {
            "type": "Flex",
            "direction": "column",
            "gap": "1rem",
            "children": [
              { "type": "DataDisplay", "label": "Name", "value": "{{state.item.name}}" },
              { "type": "DataDisplay", "label": "Status", "value": "{{state.item.status}}" },
              { "type": "DataDisplay", "label": "Created", "value": "{{state.item.createdAt}}" }
            ]
          }
        },
        "else": {
          "type": "Skeleton"
        }
      }
    ]
  }
}
```

### Form Page

```json
{
  "id": "create-item",
  "route": "/items/new",
  "state": {
    "formData": {
      "type": "object",
      "default": {
        "name": "",
        "description": "",
        "category": ""
      }
    },
    "isSubmitting": { "type": "boolean", "default": false }
  },
  "layout": {
    "type": "Container",
    "className": "p-6 max-w-2xl mx-auto",
    "children": [
      {
        "type": "PageHeader",
        "title": "Create Item",
        "backLink": "/items"
      },
      {
        "type": "Card",
        "content": {
          "type": "Form",
          "id": "create-form",
          "fields": [
            {
              "name": "name",
              "fieldType": "text",
              "label": "Name",
              "placeholder": "Enter item name",
              "bindTo": "formData.name",
              "validation": {
                "required": { "message": "Name is required" }
              }
            },
            {
              "name": "description",
              "fieldType": "textarea",
              "label": "Description",
              "placeholder": "Enter description",
              "bindTo": "formData.description"
            },
            {
              "name": "category",
              "fieldType": "select",
              "label": "Category",
              "bindTo": "formData.category",
              "options": [
                { "value": "a", "label": "Category A" },
                { "value": "b", "label": "Category B" }
              ]
            }
          ],
          "events": {
            "onSubmit": [
              { "type": "setLoading", "target": "submit", "loading": true },
              {
                "type": "callApi",
                "api": "my-plugin.createItem",
                "args": { "data": "{{state.formData}}" },
                "onSuccess": [
                  { "type": "showToast", "message": "Item created!", "level": "success" },
                  { "type": "navigate", "to": "/items" }
                ],
                "onError": [
                  { "type": "showToast", "message": "Failed to create item", "level": "error" }
                ]
              },
              { "type": "setLoading", "target": "submit", "loading": false }
            ]
          }
        }
      }
    ]
  }
}
```

## Multiple Pages

Plugins can define multiple pages:

```json
{
  "pages": [
    {
      "id": "dashboard",
      "title": "Dashboard",
      "route": "/my-plugin",
      "icon": "Home",
      "layout": { ... }
    },
    {
      "id": "items",
      "title": "Items",
      "route": "/my-plugin/items",
      "icon": "List",
      "layout": { ... }
    },
    {
      "id": "settings",
      "title": "Settings",
      "route": "/my-plugin/settings",
      "icon": "Settings",
      "layout": { ... }
    }
  ]
}
```

## Best Practices

### Route Naming

```json
// ✅ Good - namespaced routes
"/my-plugin/dashboard"
"/my-plugin/items"
"/my-plugin/items/:id"

// ❌ Avoid - may conflict
"/dashboard"
"/items"
```

### State Organization

```json
// ✅ Good - organized state
"state": {
  "data": { "type": "object" },
  "ui": {
    "type": "object",
    "default": {
      "isLoading": false,
      "selectedTab": "all"
    }
  },
  "form": {
    "type": "object",
    "default": { ... }
  }
}
```

### Loading States

Always show loading feedback:

```json
{
  "type": "Conditional",
  "condition": "{{state.isLoading}}",
  "then": { "type": "Skeleton" },
  "else": { "type": "Table", ... }
}
```

### Error Handling

Handle load failures:

```json
"onMount": [
  {
    "type": "callApi",
    "api": "getData",
    "onError": [
      { "type": "updateState", "path": "error", "value": "$error.message" }
    ]
  }
]
```

```json
{
  "type": "Conditional",
  "condition": "{{state.error}}",
  "then": {
    "type": "Alert",
    "variant": "destructive",
    "title": "Error",
    "message": "{{state.error}}"
  }
}
```

## Next Steps

- **[Components](../components/overview)** - All available components
- **[Actions](../actions/overview)** - All action types
- **[Building Plugins](./building-plugins)** - Build and distribution
