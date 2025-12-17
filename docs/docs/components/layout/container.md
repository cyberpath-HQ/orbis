---
sidebar_position: 1
title: Layout Components
description: Container, Flex, Grid, Spacer, Divider, Section
---

# Layout Components

Layout components structure your UI with containers, flexbox, grid, and spacing.

## Container

A basic block-level wrapper.

```json
{
  "type": "Container",
  "className": "p-6 max-w-4xl mx-auto",
  "children": [
    { "type": "Heading", "text": "Welcome" },
    { "type": "Text", "content": "Content here" }
  ]
}
```

### Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `children` | array | `[]` | Child components |
| `className` | string | - | CSS classes |
| `as` | string | `div` | HTML element |

### Examples

**Page wrapper:**
```json
{
  "type": "Container",
  "className": "min-h-screen p-6 bg-gray-50",
  "children": [...]
}
```

**Card-like container:**
```json
{
  "type": "Container",
  "className": "bg-white rounded-lg shadow-md p-4",
  "children": [...]
}
```

---

## Flex

Flexbox layout container.

```json
{
  "type": "Flex",
  "direction": "row",
  "justify": "between",
  "align": "center",
  "gap": "1rem",
  "children": [
    { "type": "Text", "content": "Left" },
    { "type": "Button", "label": "Right" }
  ]
}
```

### Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `children` | array | `[]` | Child components |
| `direction` | `row` \| `column` \| `row-reverse` \| `column-reverse` | `row` | Flex direction |
| `justify` | `start` \| `end` \| `center` \| `between` \| `around` \| `evenly` | `start` | Main axis alignment |
| `align` | `start` \| `end` \| `center` \| `stretch` \| `baseline` | `stretch` | Cross axis alignment |
| `wrap` | `nowrap` \| `wrap` \| `wrap-reverse` | `nowrap` | Flex wrap |
| `gap` | string | - | Gap between items |
| `className` | string | - | CSS classes |

### Examples

**Header with actions:**
```json
{
  "type": "Flex",
  "justify": "between",
  "align": "center",
  "children": [
    { "type": "Heading", "text": "Dashboard", "level": 1 },
    {
      "type": "Flex",
      "gap": "0.5rem",
      "children": [
        { "type": "Button", "label": "Export", "variant": "outline" },
        { "type": "Button", "label": "Create New" }
      ]
    }
  ]
}
```

**Vertical stack:**
```json
{
  "type": "Flex",
  "direction": "column",
  "gap": "1rem",
  "children": [
    { "type": "Field", "name": "name", "label": "Name" },
    { "type": "Field", "name": "email", "label": "Email" },
    { "type": "Button", "label": "Submit" }
  ]
}
```

**Centered content:**
```json
{
  "type": "Flex",
  "justify": "center",
  "align": "center",
  "className": "min-h-[400px]",
  "children": [
    { "type": "Text", "content": "Centered" }
  ]
}
```

---

## Grid

CSS Grid layout.

```json
{
  "type": "Grid",
  "columns": 3,
  "gap": "1rem",
  "children": [
    { "type": "Card", "title": "Card 1" },
    { "type": "Card", "title": "Card 2" },
    { "type": "Card", "title": "Card 3" }
  ]
}
```

### Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `children` | array | `[]` | Child components |
| `columns` | number \| object | `1` | Number of columns |
| `rows` | number | - | Number of rows |
| `gap` | string | - | Gap between cells |
| `rowGap` | string | - | Row gap |
| `columnGap` | string | - | Column gap |
| `className` | string | - | CSS classes |

### Responsive Columns

Use an object for responsive breakpoints:

```json
{
  "type": "Grid",
  "columns": {
    "sm": 1,
    "md": 2,
    "lg": 3,
    "xl": 4
  },
  "gap": "1.5rem",
  "children": [...]
}
```

### Examples

**Dashboard stats:**
```json
{
  "type": "Grid",
  "columns": { "sm": 1, "md": 2, "lg": 4 },
  "gap": "1rem",
  "children": [
    { "type": "StatCard", "title": "Users", "value": "{{state.stats.users}}" },
    { "type": "StatCard", "title": "Revenue", "value": "${{state.stats.revenue}}" },
    { "type": "StatCard", "title": "Orders", "value": "{{state.stats.orders}}" },
    { "type": "StatCard", "title": "Growth", "value": "{{state.stats.growth}}%" }
  ]
}
```

**Feature grid:**
```json
{
  "type": "Grid",
  "columns": { "sm": 1, "lg": 2 },
  "gap": "2rem",
  "children": [
    {
      "type": "Card",
      "title": "Recent Activity",
      "content": { "type": "List", "dataSource": "state:recentActivity" }
    },
    {
      "type": "Card",
      "title": "Performance",
      "content": { "type": "Chart", "chartType": "line" }
    }
  ]
}
```

---

## Spacer

Adds vertical space between elements.

```json
{
  "type": "Spacer",
  "size": "lg"
}
```

### Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `size` | `xs` \| `sm` \| `md` \| `lg` \| `xl` | `md` | Space amount |
| `className` | string | - | CSS classes |

### Size Reference

| Size | Value |
|------|-------|
| `xs` | 0.5rem (8px) |
| `sm` | 1rem (16px) |
| `md` | 1.5rem (24px) |
| `lg` | 2rem (32px) |
| `xl` | 3rem (48px) |

### Example

```json
{
  "type": "Container",
  "children": [
    { "type": "Heading", "text": "Section 1" },
    { "type": "Text", "content": "Content..." },
    { "type": "Spacer", "size": "xl" },
    { "type": "Heading", "text": "Section 2" },
    { "type": "Text", "content": "More content..." }
  ]
}
```

---

## Divider

A horizontal or vertical separator line.

```json
{
  "type": "Divider"
}
```

### Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `orientation` | `horizontal` \| `vertical` | `horizontal` | Direction |
| `className` | string | - | CSS classes |

### Examples

**Horizontal divider:**
```json
{
  "type": "Container",
  "children": [
    { "type": "Text", "content": "Above" },
    { "type": "Divider" },
    { "type": "Text", "content": "Below" }
  ]
}
```

**Vertical divider:**
```json
{
  "type": "Flex",
  "align": "center",
  "gap": "1rem",
  "children": [
    { "type": "Text", "content": "Left" },
    { "type": "Divider", "orientation": "vertical", "className": "h-6" },
    { "type": "Text", "content": "Right" }
  ]
}
```

**Styled divider:**
```json
{
  "type": "Divider",
  "className": "my-8 border-blue-500"
}
```

---

## Section

Semantic section wrapper with optional title.

```json
{
  "type": "Section",
  "title": "User Settings",
  "children": [
    { "type": "Form", "..." }
  ]
}
```

### Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `children` | array | `[]` | Child components |
| `title` | string | - | Section heading |
| `description` | string | - | Section description |
| `ariaLabel` | string | - | Accessibility label |
| `className` | string | - | CSS classes |

### Examples

**Settings section:**
```json
{
  "type": "Section",
  "title": "Notifications",
  "description": "Configure how you receive notifications",
  "className": "mb-8",
  "children": [
    {
      "type": "Form",
      "fields": [
        { "name": "email", "fieldType": "checkbox", "label": "Email notifications" },
        { "name": "push", "fieldType": "checkbox", "label": "Push notifications" }
      ]
    }
  ]
}
```

**Page with sections:**
```json
{
  "type": "Container",
  "className": "space-y-8",
  "children": [
    {
      "type": "Section",
      "title": "Profile",
      "children": [{ "...profile content..." }]
    },
    {
      "type": "Section",
      "title": "Security",
      "children": [{ "...security content..." }]
    },
    {
      "type": "Section",
      "title": "Preferences",
      "children": [{ "...preferences content..." }]
    }
  ]
}
```

---

## Layout Patterns

### Page Layout

```json
{
  "type": "Container",
  "className": "min-h-screen",
  "children": [
    {
      "type": "Container",
      "className": "max-w-7xl mx-auto p-6",
      "children": [
        { "type": "PageHeader", "title": "Dashboard" },
        { "type": "Spacer", "size": "lg" },
        {
          "type": "Grid",
          "columns": { "lg": 3 },
          "gap": "1rem",
          "children": [{ "...content..." }]
        }
      ]
    }
  ]
}
```

### Two-Column Layout

```json
{
  "type": "Grid",
  "columns": { "md": 2 },
  "gap": "2rem",
  "children": [
    {
      "type": "Container",
      "children": [{ "...main content..." }]
    },
    {
      "type": "Container",
      "className": "lg:sticky lg:top-6",
      "children": [{ "...sidebar..." }]
    }
  ]
}
```

### Centered Card

```json
{
  "type": "Flex",
  "justify": "center",
  "align": "center",
  "className": "min-h-screen p-6",
  "children": [
    {
      "type": "Card",
      "className": "w-full max-w-md",
      "title": "Sign In",
      "content": { "type": "Form", "..." }
    }
  ]
}
```
