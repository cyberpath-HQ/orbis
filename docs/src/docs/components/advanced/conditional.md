---
sidebar_position: 8
title: Advanced Components
description: Conditional, Loop, Accordion, Icon, Fragment, Slot, Custom
---

# Advanced Components

Components for dynamic rendering and specialized use cases.

## Conditional

Conditional rendering based on expressions.

```json
{
  "type": "Conditional",
  "condition": "{{state.isLoggedIn}}",
  "then": { "type": "Text", "content": "Welcome back!" },
  "else": { "type": "Button", "label": "Sign In" }
}
```

### Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `condition` | string | - | Expression to evaluate |
| `then` | object | - | Component when true |
| `else` | object | - | Component when false (optional) |

### Condition Syntax

```json
// Boolean state
"condition": "{{state.isActive}}"

// Comparison
"condition": "{{state.count}} > 0"
"condition": "{{state.status}} === 'completed'"

// Logical operators
"condition": "{{state.isAdmin}} && {{state.hasPermission}}"
"condition": "{{state.isEmpty}} || {{state.isLoading}}"

// Negation
"condition": "!{{state.hasError}}"

// Complex
"condition": "({{state.role}} === 'admin' || {{state.role}} === 'manager') && {{state.isVerified}}"
```

### Examples

**Loading state:**
```json
{
  "type": "Conditional",
  "condition": "{{state.isLoading}}",
  "then": { "type": "Skeleton", "count": 5 },
  "else": { "type": "Table", "dataSource": "state:items" }
}
```

**Empty state:**
```json
{
  "type": "Conditional",
  "condition": "{{state.items.length}} === 0",
  "then": {
    "type": "EmptyState",
    "title": "No Items",
    "description": "Create your first item"
  },
  "else": { "type": "List", "dataSource": "state:items" }
}
```

**Role-based access:**
```json
{
  "type": "Conditional",
  "condition": "{{state.user.role}} === 'admin'",
  "then": {
    "type": "Button",
    "label": "Admin Settings",
    "events": { "onClick": [{ "type": "navigate", "to": "/admin" }] }
  }
}
```

**Nested conditions:**
```json
{
  "type": "Conditional",
  "condition": "{{state.status}} === 'loading'",
  "then": { "type": "Skeleton" },
  "else": {
    "type": "Conditional",
    "condition": "{{state.status}} === 'error'",
    "then": { "type": "Alert", "variant": "destructive", "message": "{{state.error}}" },
    "else": { "type": "Text", "content": "{{state.data}}" }
  }
}
```

---

## Loop

Iterate over arrays to render repeated content.

```json
{
  "type": "Loop",
  "items": "{{state.items}}",
  "itemTemplate": {
    "type": "Card",
    "title": "{{$item.name}}",
    "content": { "type": "Text", "content": "{{$item.description}}" }
  }
}
```

### Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `items` | string \| array | - | Data source or static array |
| `itemTemplate` | object | - | Component template for each item |
| `keyField` | string | - | Unique key field name |
| `emptyMessage` | string | - | Empty state message |

### Loop Variables

Inside `itemTemplate`, you have access to:

| Variable | Description |
|----------|-------------|
| `$item` | Current item |
| `$index` | Zero-based index |
| `$first` | Is first item |
| `$last` | Is last item |
| `$even` | Is even index |
| `$odd` | Is odd index |

### Examples

**Basic list:**
```json
{
  "type": "Loop",
  "items": "{{state.users}}",
  "keyField": "id",
  "itemTemplate": {
    "type": "Flex",
    "align": "center",
    "gap": "1rem",
    "className": "p-4 border-b",
    "children": [
      { "type": "Avatar", "src": "{{$item.avatar}}", "fallback": "{{$item.initials}}" },
      {
        "type": "Flex",
        "direction": "column",
        "children": [
          { "type": "Text", "content": "{{$item.name}}", "className": "font-medium" },
          { "type": "Text", "variant": "muted", "content": "{{$item.email}}" }
        ]
      }
    ]
  }
}
```

**Grid of cards:**
```json
{
  "type": "Grid",
  "columns": { "sm": 1, "md": 2, "lg": 3 },
  "gap": "1rem",
  "children": [
    {
      "type": "Loop",
      "items": "{{state.products}}",
      "itemTemplate": {
        "type": "Card",
        "className": "overflow-hidden",
        "content": {
          "type": "Container",
          "children": [
            { "type": "Image", "src": "{{$item.image}}", "className": "w-full h-48 object-cover" },
            {
              "type": "Container",
              "className": "p-4",
              "children": [
                { "type": "Heading", "level": 3, "text": "{{$item.name}}" },
                { "type": "Text", "content": "${{$item.price}}" }
              ]
            }
          ]
        }
      }
    }
  ]
}
```

**With index styling:**
```json
{
  "type": "Loop",
  "items": "{{state.items}}",
  "itemTemplate": {
    "type": "Flex",
    "className": "{{$even ? 'bg-gray-50' : 'bg-white'}} p-4",
    "children": [
      { "type": "Text", "content": "{{$index + 1}}. {{$item.name}}" }
    ]
  }
}
```

**Static array:**
```json
{
  "type": "Loop",
  "items": [
    { "icon": "Home", "label": "Dashboard", "path": "/" },
    { "icon": "Settings", "label": "Settings", "path": "/settings" }
  ],
  "itemTemplate": {
    "type": "Link",
    "to": "{{$item.path}}",
    "children": {
      "type": "Flex",
      "align": "center",
      "gap": "0.5rem",
      "children": [
        { "type": "Icon", "name": "{{$item.icon}}" },
        { "type": "Text", "content": "{{$item.label}}" }
      ]
    }
  }
}
```

---

## Accordion

Collapsible content sections.

```json
{
  "type": "Accordion",
  "items": [
    {
      "title": "Section 1",
      "content": { "type": "Text", "content": "Content for section 1" }
    },
    {
      "title": "Section 2",
      "content": { "type": "Text", "content": "Content for section 2" }
    }
  ]
}
```

### Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `items` | array | `[]` | Accordion items |
| `type` | `single` \| `multiple` | `single` | Selection mode |
| `defaultValue` | string \| array | - | Initially expanded |
| `collapsible` | boolean | `true` | Allow collapse all |
| `className` | string | - | CSS classes |

### Item Definition

```typescript
interface AccordionItem {
  id?: string;
  title: string;
  content: ComponentSchema;
  disabled?: boolean;
}
```

### Examples

**FAQ section:**
```json
{
  "type": "Accordion",
  "type": "single",
  "collapsible": true,
  "items": [
    {
      "id": "faq-1",
      "title": "What is Orbis?",
      "content": {
        "type": "Text",
        "content": "Orbis is a desktop application platform with plugin support."
      }
    },
    {
      "id": "faq-2",
      "title": "How do I install plugins?",
      "content": {
        "type": "Text",
        "content": "Copy WASM plugin files to the plugins directory."
      }
    },
    {
      "id": "faq-3",
      "title": "Is Orbis open source?",
      "content": {
        "type": "Text",
        "content": "Yes, Orbis is open source under the MIT license."
      }
    }
  ]
}
```

**Multiple open:**
```json
{
  "type": "Accordion",
  "type": "multiple",
  "defaultValue": ["section-1"],
  "items": [
    {
      "id": "section-1",
      "title": "General Settings",
      "content": { "type": "Form", "..." }
    },
    {
      "id": "section-2",
      "title": "Advanced Settings",
      "content": { "type": "Form", "..." }
    }
  ]
}
```

**Dynamic from state:**
```json
{
  "type": "Accordion",
  "items": "{{state.faqItems}}"
}
```

---

## Icon

Display icons from lucide-react.

```json
{
  "type": "Icon",
  "name": "Settings",
  "className": "w-5 h-5"
}
```

### Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `name` | string | - | Icon name (required) |
| `size` | number \| string | - | Icon size |
| `className` | string | - | CSS classes |
| `strokeWidth` | number | `2` | Stroke width |

### Examples

**With size:**
```json
{
  "type": "Flex",
  "gap": "1rem",
  "align": "center",
  "children": [
    { "type": "Icon", "name": "Star", "className": "w-4 h-4" },
    { "type": "Icon", "name": "Star", "className": "w-6 h-6" },
    { "type": "Icon", "name": "Star", "className": "w-8 h-8" }
  ]
}
```

**With color:**
```json
{
  "type": "Icon",
  "name": "CheckCircle",
  "className": "w-5 h-5 text-green-500"
}
```

**Dynamic icon:**
```json
{
  "type": "Icon",
  "name": "{{state.status === 'success' ? 'CheckCircle' : 'AlertCircle'}}",
  "className": "w-5 h-5 {{state.status === 'success' ? 'text-green-500' : 'text-red-500'}}"
}
```

### Common Icons

| Category | Icons |
|----------|-------|
| Navigation | `Home`, `ArrowLeft`, `ArrowRight`, `ChevronDown`, `Menu` |
| Actions | `Plus`, `Edit`, `Trash`, `Download`, `Upload`, `Copy` |
| Status | `Check`, `X`, `AlertTriangle`, `Info`, `HelpCircle` |
| User | `User`, `Users`, `Settings`, `LogOut`, `Bell` |
| Files | `File`, `FileText`, `Folder`, `FolderOpen`, `Image` |
| Data | `Database`, `Table`, `BarChart`, `LineChart`, `PieChart` |

Browse all icons at [lucide.dev](https://lucide.dev/icons).

---

## Fragment

Invisible wrapper for grouping components.

```json
{
  "type": "Fragment",
  "children": [
    { "type": "Text", "content": "First" },
    { "type": "Text", "content": "Second" }
  ]
}
```

### Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `children` | array | `[]` | Child components |

### Use Cases

**Return multiple elements:**
```json
{
  "type": "Conditional",
  "condition": "{{state.showDetails}}",
  "then": {
    "type": "Fragment",
    "children": [
      { "type": "Divider" },
      { "type": "Text", "content": "Additional details..." },
      { "type": "Button", "label": "Learn More" }
    ]
  }
}
```

**In loop:**
```json
{
  "type": "Loop",
  "items": "{{state.items}}",
  "itemTemplate": {
    "type": "Fragment",
    "children": [
      { "type": "Heading", "level": 3, "text": "{{$item.title}}" },
      { "type": "Text", "content": "{{$item.description}}" },
      { "type": "Conditional", "condition": "!{{$last}}", "then": { "type": "Divider" } }
    ]
  }
}
```

---

## Slot

Placeholder for plugin-defined content.

```json
{
  "type": "Slot",
  "name": "sidebar-widgets",
  "fallback": { "type": "Text", "content": "No widgets available" }
}
```

### Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `name` | string | - | Slot identifier |
| `fallback` | object | - | Fallback content |
| `props` | object | - | Props passed to slot content |

### Use Cases

Slots allow plugins to inject content into predefined areas:

```json
{
  "type": "Container",
  "children": [
    { "type": "PageHeader", "title": "Dashboard" },
    {
      "type": "Grid",
      "columns": { "lg": 3 },
      "children": [
        { "type": "Slot", "name": "dashboard-widget-1" },
        { "type": "Slot", "name": "dashboard-widget-2" },
        { "type": "Slot", "name": "dashboard-widget-3" }
      ]
    }
  ]
}
```

---

## Custom

Render a custom component registered by the host application.

```json
{
  "type": "Custom",
  "component": "MySpecialChart",
  "props": {
    "data": "{{state.chartData}}",
    "options": {
      "showLegend": true
    }
  }
}
```

### Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `component` | string | - | Registered component name |
| `props` | object | - | Props to pass |
| `className` | string | - | CSS classes |

### Example

If the host app registers a `MapView` component:

```json
{
  "type": "Custom",
  "component": "MapView",
  "props": {
    "center": "{{state.location}}",
    "zoom": 14,
    "markers": "{{state.markers}}"
  }
}
```

---

## Advanced Patterns

### State Machine Pattern

```json
{
  "state": {
    "status": { "type": "string", "default": "idle" }
  },
  "layout": {
    "type": "Container",
    "children": [
      {
        "type": "Conditional",
        "condition": "{{state.status}} === 'idle'",
        "then": {
          "type": "Button",
          "label": "Start",
          "events": {
            "onClick": [{ "type": "updateState", "path": "status", "value": "loading" }]
          }
        }
      },
      {
        "type": "Conditional",
        "condition": "{{state.status}} === 'loading'",
        "then": { "type": "LoadingOverlay", "text": "Processing..." }
      },
      {
        "type": "Conditional",
        "condition": "{{state.status}} === 'success'",
        "then": { "type": "Alert", "variant": "success", "message": "Done!" }
      },
      {
        "type": "Conditional",
        "condition": "{{state.status}} === 'error'",
        "then": { "type": "Alert", "variant": "destructive", "message": "{{state.error}}" }
      }
    ]
  }
}
```

### Dynamic Form Fields

```json
{
  "type": "Loop",
  "items": "{{state.formSchema.fields}}",
  "itemTemplate": {
    "type": "Conditional",
    "condition": "{{$item.visible}} !== false",
    "then": {
      "type": "Field",
      "name": "{{$item.name}}",
      "fieldType": "{{$item.type}}",
      "label": "{{$item.label}}",
      "bindTo": "formData.{{$item.name}}",
      "options": "{{$item.options}}"
    }
  }
}
```

### Recursive Components

For tree structures, use nested loops:

```json
{
  "type": "Loop",
  "items": "{{state.treeNodes}}",
  "itemTemplate": {
    "type": "Accordion",
    "items": [
      {
        "title": "{{$item.name}}",
        "content": {
          "type": "Conditional",
          "condition": "{{$item.children.length}} > 0",
          "then": {
            "type": "Loop",
            "items": "{{$item.children}}",
            "itemTemplate": { "type": "Text", "content": "{{$item.name}}" }
          },
          "else": { "type": "Text", "content": "No children" }
        }
      }
    ]
  }
}
```
