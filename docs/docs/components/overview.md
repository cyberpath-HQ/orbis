---
sidebar_position: 1
title: Components Overview
description: All available UI components
---

# Components Overview

Orbis provides 35+ UI components organized into categories. Each component is defined by a JSON schema and rendered by the SchemaRenderer.

## Component Categories

| Category | Components | Purpose |
|----------|------------|---------|
| [Layout](./layout/container) | 6 | Structure and spacing |
| [Typography](./typography/text) | 2 | Text display |
| [Forms](./forms/form) | 2 | User input |
| [Data Display](./data-display/table) | 9 | Showing data |
| [Feedback](./feedback/alert) | 5 | User notifications |
| [Navigation](./navigation/button) | 6 | User interaction |
| [Overlays](./overlays/modal) | 2 | Dialogs and tooltips |
| [Advanced](./advanced/conditional) | 7 | Dynamic rendering |

## Common Properties

All components share these base properties:

```typescript
interface BaseSchema {
  type: string;           // Component type (required)
  visible?: string;       // Visibility expression
  className?: string;     // CSS classes (Tailwind)
  style?: object;         // Inline styles
  testId?: string;        // Test identifier
  ariaLabel?: string;     // Accessibility label
}
```

### type

The component type identifier (required).

```json
{ "type": "Button" }
```

### visible

Expression controlling visibility:

```json
{
  "type": "Card",
  "visible": "{{state.showCard}}"
}
```

Supports complex expressions:
```json
{ "visible": "{{state.count}} > 0 && {{state.isActive}}" }
```

### className

Tailwind CSS classes:

```json
{
  "type": "Container",
  "className": "p-6 bg-white rounded-lg shadow"
}
```

### testId

For testing:

```json
{
  "type": "Button",
  "testId": "submit-btn"
}
```

## Event Handling

Components with interactivity have events:

```json
{
  "type": "Button",
  "label": "Click Me",
  "events": {
    "onClick": [
      { "type": "updateState", "path": "count", "value": "{{state.count}} + 1" }
    ]
  }
}
```

Available events vary by component. See individual component docs.

## Expressions in Properties

Most string properties support expressions:

```json
{
  "type": "Text",
  "content": "Hello, {{state.userName}}!"
}
```

See [Expressions](../core-concepts/expressions) for full syntax.

## Quick Reference

### Layout Components

| Component | Purpose |
|-----------|---------|
| `Container` | Block wrapper |
| `Flex` | Flexbox layout |
| `Grid` | Grid layout |
| `Spacer` | Vertical space |
| `Divider` | Visual separator |
| `Section` | Semantic grouping |

### Typography Components

| Component | Purpose |
|-----------|---------|
| `Text` | Paragraph text |
| `Heading` | Headings (h1-h6) |

### Form Components

| Component | Purpose |
|-----------|---------|
| `Form` | Form container |
| `Field` | Form field |

### Data Display Components

| Component | Purpose |
|-----------|---------|
| `Table` | Data tables |
| `List` | Lists and menus |
| `Card` | Content cards |
| `StatCard` | Statistics display |
| `DataDisplay` | Key-value pairs |
| `Badge` | Labels and tags |
| `Avatar` | User images |
| `Image` | Images |
| `Chart` | Data visualization |

### Feedback Components

| Component | Purpose |
|-----------|---------|
| `Alert` | Notifications |
| `Progress` | Progress bars |
| `Skeleton` | Loading placeholders |
| `LoadingOverlay` | Loading states |
| `EmptyState` | Empty content |

### Navigation Components

| Component | Purpose |
|-----------|---------|
| `Button` | Clickable buttons |
| `Link` | Navigation links |
| `Tabs` | Tab navigation |
| `Breadcrumb` | Path navigation |
| `Dropdown` | Dropdown menus |
| `PageHeader` | Page headers |

### Overlay Components

| Component | Purpose |
|-----------|---------|
| `Modal` | Dialog windows |
| `Tooltip` | Hover tooltips |

### Advanced Components

| Component | Purpose |
|-----------|---------|
| `Conditional` | Conditional rendering |
| `Loop` | List iteration |
| `Accordion` | Collapsible sections |
| `Icon` | Icon display |
| `Fragment` | Invisible wrapper |
| `Slot` | Plugin slots |
| `Custom` | Custom components |

## Usage Patterns

### Container with Children

```json
{
  "type": "Container",
  "className": "p-6",
  "children": [
    { "type": "Heading", "text": "Title", "level": 1 },
    { "type": "Text", "content": "Description text here." }
  ]
}
```

### Data-Bound Component

```json
{
  "type": "Table",
  "dataSource": "state:items",
  "columns": [
    { "key": "name", "label": "Name" },
    { "key": "status", "label": "Status" }
  ]
}
```

### Interactive Component

```json
{
  "type": "Button",
  "label": "Save",
  "variant": "default",
  "events": {
    "onClick": [
      { "type": "callApi", "api": "saveData" },
      { "type": "showToast", "message": "Saved!" }
    ]
  }
}
```

### Conditional Rendering

```json
{
  "type": "Conditional",
  "condition": "{{state.isLoading}}",
  "then": { "type": "Skeleton" },
  "else": { "type": "Card", "content": { "..." } }
}
```

## Styling

### Tailwind Classes

All components support `className` for Tailwind:

```json
{
  "type": "Card",
  "className": "bg-gradient-to-r from-blue-500 to-purple-500 text-white"
}
```

### Responsive Classes

```json
{
  "type": "Grid",
  "columns": { "sm": 1, "md": 2, "lg": 3 },
  "className": "gap-4 md:gap-6"
}
```

### Dark Mode

Use dark: prefix:

```json
{
  "className": "bg-white dark:bg-gray-900 text-gray-900 dark:text-white"
}
```

## Next Steps

Explore each component category:

- [Layout Components](./layout/container)
- [Typography Components](./typography/text)
- [Form Components](./forms/form)
- [Data Display Components](./data-display/table)
- [Feedback Components](./feedback/alert)
- [Navigation Components](./navigation/button)
- [Overlay Components](./overlays/modal)
- [Advanced Components](./advanced/conditional)
