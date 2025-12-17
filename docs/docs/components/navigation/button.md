---
sidebar_position: 6
title: Navigation Components
description: Button, Link, Tabs, Breadcrumb, Dropdown, PageHeader
---

# Navigation Components

Components for user interaction and navigation.

## Button

Interactive button with multiple variants and states.

```json
{
  "type": "Button",
  "label": "Click Me",
  "events": {
    "onClick": [
      { "type": "updateState", "path": "clicked", "value": true }
    ]
  }
}
```

### Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `label` | string | - | Button text |
| `variant` | `default` \| `secondary` \| `outline` \| `ghost` \| `link` \| `destructive` | `default` | Button style |
| `size` | `sm` \| `md` \| `lg` \| `icon` | `md` | Button size |
| `icon` | string | - | Icon name |
| `iconPosition` | `left` \| `right` | `left` | Icon placement |
| `disabled` | boolean \| string | `false` | Disabled state |
| `loading` | boolean \| string | `false` | Loading state |
| `type` | `button` \| `submit` \| `reset` | `button` | HTML type |
| `className` | string | - | CSS classes |
| `events` | object | - | Event handlers |

### Events

| Event | Trigger |
|-------|---------|
| `onClick` | Button clicked |

### Variants

```json
{
  "type": "Flex",
  "gap": "0.5rem",
  "wrap": "wrap",
  "children": [
    { "type": "Button", "label": "Default", "variant": "default" },
    { "type": "Button", "label": "Secondary", "variant": "secondary" },
    { "type": "Button", "label": "Outline", "variant": "outline" },
    { "type": "Button", "label": "Ghost", "variant": "ghost" },
    { "type": "Button", "label": "Link", "variant": "link" },
    { "type": "Button", "label": "Destructive", "variant": "destructive" }
  ]
}
```

### Sizes

```json
{
  "type": "Flex",
  "gap": "0.5rem",
  "align": "center",
  "children": [
    { "type": "Button", "label": "Small", "size": "sm" },
    { "type": "Button", "label": "Medium", "size": "md" },
    { "type": "Button", "label": "Large", "size": "lg" }
  ]
}
```

### Examples

**With icon:**
```json
{
  "type": "Button",
  "label": "Add Item",
  "icon": "Plus",
  "events": { "onClick": [{ "type": "navigate", "to": "/items/new" }] }
}
```

**Icon only:**
```json
{
  "type": "Button",
  "icon": "Settings",
  "size": "icon",
  "variant": "ghost",
  "ariaLabel": "Open settings",
  "events": { "onClick": [{ "type": "navigate", "to": "/settings" }] }
}
```

**Loading state:**
```json
{
  "type": "Button",
  "label": "Save",
  "loading": "{{state.isSaving}}",
  "events": {
    "onClick": [
      { "type": "updateState", "path": "isSaving", "value": true },
      { "type": "callApi", "api": "save" },
      { "type": "updateState", "path": "isSaving", "value": false }
    ]
  }
}
```

**Conditional disabled:**
```json
{
  "type": "Button",
  "label": "Submit",
  "disabled": "{{!state.formValid}}",
  "events": { "onClick": [{ "type": "callApi", "api": "submit" }] }
}
```

---

## Link

Navigation link component.

```json
{
  "type": "Link",
  "to": "/dashboard",
  "label": "Go to Dashboard"
}
```

### Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `to` | string | - | Navigation path |
| `label` | string | - | Link text |
| `external` | boolean | `false` | Open in new tab |
| `children` | object | - | Custom content |
| `className` | string | - | CSS classes |

### Examples

**Simple link:**
```json
{
  "type": "Link",
  "to": "/help",
  "label": "Help Center"
}
```

**External link:**
```json
{
  "type": "Link",
  "to": "https://github.com",
  "label": "GitHub",
  "external": true
}
```

**With custom content:**
```json
{
  "type": "Link",
  "to": "/profile",
  "children": {
    "type": "Flex",
    "align": "center",
    "gap": "0.5rem",
    "children": [
      { "type": "Avatar", "src": "{{user.avatar}}", "size": "sm" },
      { "type": "Text", "content": "{{user.name}}" }
    ]
  }
}
```

---

## Tabs

Tabbed navigation for switching between views.

```json
{
  "type": "Tabs",
  "defaultValue": "overview",
  "tabs": [
    { "id": "overview", "label": "Overview" },
    { "id": "analytics", "label": "Analytics" },
    { "id": "settings", "label": "Settings" }
  ],
  "content": {
    "overview": { "type": "Text", "content": "Overview content" },
    "analytics": { "type": "Text", "content": "Analytics content" },
    "settings": { "type": "Text", "content": "Settings content" }
  }
}
```

### Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `tabs` | array | `[]` | Tab definitions |
| `content` | object | - | Content for each tab |
| `defaultValue` | string | - | Initially active tab |
| `bindTo` | string | - | State path for active tab |
| `variant` | `default` \| `pills` \| `underline` | `default` | Tab style |
| `orientation` | `horizontal` \| `vertical` | `horizontal` | Layout direction |
| `className` | string | - | CSS classes |

### Events

| Event | Payload |
|-------|---------|
| `onTabChange` | `$value` - new tab id |

### Tab Definition

```typescript
interface Tab {
  id: string;
  label: string;
  icon?: string;
  disabled?: boolean;
  badge?: string;
}
```

### Examples

**With icons:**
```json
{
  "type": "Tabs",
  "tabs": [
    { "id": "home", "label": "Home", "icon": "Home" },
    { "id": "profile", "label": "Profile", "icon": "User" },
    { "id": "settings", "label": "Settings", "icon": "Settings" }
  ],
  "content": { "..." }
}
```

**With badges:**
```json
{
  "type": "Tabs",
  "tabs": [
    { "id": "all", "label": "All" },
    { "id": "unread", "label": "Unread", "badge": "{{state.unreadCount}}" }
  ]
}
```

**Controlled:**
```json
{
  "type": "Tabs",
  "bindTo": "activeTab",
  "tabs": [
    { "id": "pending", "label": "Pending" },
    { "id": "completed", "label": "Completed" }
  ],
  "events": {
    "onTabChange": [
      {
        "type": "callApi",
        "api": "getItems",
        "args": { "status": "{{$value}}" }
      }
    ]
  }
}
```

---

## Breadcrumb

Navigation path indicator.

```json
{
  "type": "Breadcrumb",
  "items": [
    { "label": "Home", "to": "/" },
    { "label": "Products", "to": "/products" },
    { "label": "Electronics", "to": "/products/electronics" },
    { "label": "Laptops" }
  ]
}
```

### Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `items` | array | `[]` | Breadcrumb items |
| `separator` | string | `/` | Separator character |
| `className` | string | - | CSS classes |

### Item Definition

```typescript
interface BreadcrumbItem {
  label: string;
  to?: string;      // Optional - last item typically has no link
  icon?: string;
}
```

### Examples

**With home icon:**
```json
{
  "type": "Breadcrumb",
  "items": [
    { "label": "Home", "to": "/", "icon": "Home" },
    { "label": "Users", "to": "/users" },
    { "label": "John Doe" }
  ]
}
```

**Dynamic from state:**
```json
{
  "type": "Breadcrumb",
  "items": "{{state.breadcrumbs}}"
}
```

---

## Dropdown

Dropdown menu for actions.

```json
{
  "type": "Dropdown",
  "trigger": {
    "type": "Button",
    "icon": "MoreVertical",
    "variant": "ghost",
    "size": "icon"
  },
  "items": [
    { "label": "Edit", "icon": "Edit", "action": [{ "type": "navigate", "to": "/edit" }] },
    { "label": "Duplicate", "icon": "Copy" },
    { "separator": true },
    { "label": "Delete", "icon": "Trash", "variant": "destructive" }
  ]
}
```

### Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `trigger` | object | - | Trigger component |
| `items` | array | `[]` | Menu items |
| `align` | `start` \| `center` \| `end` | `end` | Menu alignment |
| `side` | `top` \| `bottom` \| `left` \| `right` | `bottom` | Menu position |
| `className` | string | - | CSS classes |

### Item Definition

```typescript
interface DropdownItem {
  label?: string;
  icon?: string;
  shortcut?: string;
  disabled?: boolean;
  variant?: 'default' | 'destructive';
  action?: Action[];
  separator?: boolean;  // If true, renders a separator
}
```

### Examples

**User menu:**
```json
{
  "type": "Dropdown",
  "trigger": {
    "type": "Flex",
    "align": "center",
    "gap": "0.5rem",
    "className": "cursor-pointer",
    "children": [
      { "type": "Avatar", "src": "{{user.avatar}}", "size": "sm" },
      { "type": "Icon", "name": "ChevronDown", "className": "w-4 h-4" }
    ]
  },
  "items": [
    { "label": "Profile", "icon": "User", "action": [{ "type": "navigate", "to": "/profile" }] },
    { "label": "Settings", "icon": "Settings", "action": [{ "type": "navigate", "to": "/settings" }] },
    { "separator": true },
    { "label": "Sign Out", "icon": "LogOut", "action": [{ "type": "callApi", "api": "auth.logout" }] }
  ]
}
```

**Context menu:**
```json
{
  "type": "Dropdown",
  "trigger": {
    "type": "Button",
    "label": "Actions",
    "icon": "ChevronDown",
    "iconPosition": "right",
    "variant": "outline"
  },
  "items": [
    { "label": "Export CSV", "icon": "Download", "action": [{ "type": "download", "url": "/export/csv" }] },
    { "label": "Export PDF", "icon": "FileText", "action": [{ "type": "download", "url": "/export/pdf" }] },
    { "separator": true },
    { "label": "Print", "icon": "Printer", "shortcut": "⌘P" }
  ]
}
```

---

## PageHeader

Page title with optional subtitle and actions.

```json
{
  "type": "PageHeader",
  "title": "Dashboard",
  "subtitle": "Overview of your account"
}
```

### Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `title` | string | - | Page title |
| `subtitle` | string | - | Subtitle text |
| `backLink` | string | - | Back navigation path |
| `actions` | array | - | Action buttons |
| `breadcrumb` | array | - | Breadcrumb items |
| `className` | string | - | CSS classes |

### Examples

**With actions:**
```json
{
  "type": "PageHeader",
  "title": "Users",
  "subtitle": "Manage your team members",
  "actions": [
    { "type": "Button", "label": "Export", "variant": "outline", "icon": "Download" },
    { "type": "Button", "label": "Add User", "icon": "Plus" }
  ]
}
```

**With back link:**
```json
{
  "type": "PageHeader",
  "title": "Edit User",
  "backLink": "/users",
  "actions": [
    { "type": "Button", "label": "Cancel", "variant": "ghost" },
    { "type": "Button", "label": "Save" }
  ]
}
```

**With breadcrumb:**
```json
{
  "type": "PageHeader",
  "title": "{{state.product.name}}",
  "breadcrumb": [
    { "label": "Products", "to": "/products" },
    { "label": "{{state.product.category}}", "to": "/products?category={{state.product.categoryId}}" },
    { "label": "{{state.product.name}}" }
  ],
  "actions": [
    { "type": "Button", "label": "Edit", "variant": "outline" }
  ]
}
```

**Dashboard header:**
```json
{
  "type": "PageHeader",
  "title": "Welcome back, {{state.user.name}}!",
  "subtitle": "Here's what's happening with your projects.",
  "actions": [
    {
      "type": "Dropdown",
      "trigger": { "type": "Button", "label": "Quick Actions", "icon": "ChevronDown", "iconPosition": "right" },
      "items": [
        { "label": "New Project", "icon": "Plus" },
        { "label": "Import Data", "icon": "Upload" }
      ]
    }
  ]
}
```

---

## Navigation Patterns

### Sidebar Navigation

```json
{
  "type": "Container",
  "className": "w-64 h-screen bg-gray-50 p-4",
  "children": [
    {
      "type": "Flex",
      "direction": "column",
      "gap": "0.25rem",
      "children": [
        {
          "type": "Link",
          "to": "/dashboard",
          "children": {
            "type": "Flex",
            "align": "center",
            "gap": "0.75rem",
            "className": "px-3 py-2 rounded hover:bg-gray-100 {{$path === '/dashboard' ? 'bg-gray-100' : ''}}",
            "children": [
              { "type": "Icon", "name": "Home" },
              { "type": "Text", "content": "Dashboard" }
            ]
          }
        },
        {
          "type": "Link",
          "to": "/projects",
          "children": {
            "type": "Flex",
            "align": "center",
            "gap": "0.75rem",
            "className": "px-3 py-2 rounded hover:bg-gray-100",
            "children": [
              { "type": "Icon", "name": "Folder" },
              { "type": "Text", "content": "Projects" }
            ]
          }
        }
      ]
    }
  ]
}
```

### Action Bar

```json
{
  "type": "Flex",
  "justify": "between",
  "align": "center",
  "className": "mb-6",
  "children": [
    {
      "type": "Flex",
      "gap": "0.5rem",
      "children": [
        { "type": "Button", "label": "All", "variant": "{{state.filter === 'all' ? 'default' : 'ghost'}}" },
        { "type": "Button", "label": "Active", "variant": "{{state.filter === 'active' ? 'default' : 'ghost'}}" },
        { "type": "Button", "label": "Archived", "variant": "{{state.filter === 'archived' ? 'default' : 'ghost'}}" }
      ]
    },
    {
      "type": "Flex",
      "gap": "0.5rem",
      "children": [
        { "type": "Button", "icon": "Search", "variant": "ghost", "size": "icon" },
        { "type": "Button", "icon": "Filter", "variant": "ghost", "size": "icon" },
        { "type": "Button", "label": "Add New", "icon": "Plus" }
      ]
    }
  ]
}
```
