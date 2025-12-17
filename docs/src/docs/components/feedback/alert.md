---
sidebar_position: 5
title: Feedback Components
description: Alert, Progress, Skeleton, LoadingOverlay, EmptyState
---

# Feedback Components

Components for user notifications, loading states, and empty states.

## Alert

Notification banner for important messages.

```json
{
  "type": "Alert",
  "variant": "info",
  "title": "Information",
  "message": "Your session will expire in 5 minutes."
}
```

### Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `title` | string | - | Alert title |
| `message` | string | - | Alert message |
| `variant` | `default` \| `info` \| `success` \| `warning` \| `destructive` | `default` | Alert style |
| `icon` | string | - | Custom icon |
| `dismissible` | boolean | `false` | Show close button |
| `action` | object | - | Action button |
| `className` | string | - | CSS classes |

### Events

| Event | Trigger |
|-------|---------|
| `onClose` | Dismiss clicked |

### Variants

```json
{
  "type": "Flex",
  "direction": "column",
  "gap": "1rem",
  "children": [
    {
      "type": "Alert",
      "variant": "info",
      "title": "Info",
      "message": "This is an informational alert."
    },
    {
      "type": "Alert",
      "variant": "success",
      "title": "Success",
      "message": "Your changes have been saved."
    },
    {
      "type": "Alert",
      "variant": "warning",
      "title": "Warning",
      "message": "Your subscription expires soon."
    },
    {
      "type": "Alert",
      "variant": "destructive",
      "title": "Error",
      "message": "Failed to save changes."
    }
  ]
}
```

### Examples

**Dismissible alert:**
```json
{
  "type": "Conditional",
  "condition": "{{state.showAlert}}",
  "then": {
    "type": "Alert",
    "variant": "info",
    "title": "New Feature",
    "message": "Check out our new dashboard.",
    "dismissible": true,
    "events": {
      "onClose": [
        { "type": "updateState", "path": "showAlert", "value": false }
      ]
    }
  }
}
```

**Alert with action:**
```json
{
  "type": "Alert",
  "variant": "warning",
  "title": "Subscription Expiring",
  "message": "Your subscription expires in 3 days.",
  "action": {
    "type": "Button",
    "label": "Renew Now",
    "size": "sm",
    "events": {
      "onClick": [{ "type": "navigate", "to": "/billing" }]
    }
  }
}
```

**Error from state:**
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

---

## Progress

Progress bar for showing completion status.

```json
{
  "type": "Progress",
  "value": 75,
  "max": 100
}
```

### Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `value` | number \| string | `0` | Current value |
| `max` | number | `100` | Maximum value |
| `showLabel` | boolean | `false` | Show percentage label |
| `size` | `sm` \| `md` \| `lg` | `md` | Bar height |
| `variant` | `default` \| `success` \| `warning` \| `destructive` | `default` | Color style |
| `className` | string | - | CSS classes |

### Examples

**With label:**
```json
{
  "type": "Flex",
  "direction": "column",
  "gap": "0.5rem",
  "children": [
    {
      "type": "Flex",
      "justify": "between",
      "children": [
        { "type": "Text", "content": "Uploading..." },
        { "type": "Text", "content": "{{state.uploadProgress}}%" }
      ]
    },
    {
      "type": "Progress",
      "value": "{{state.uploadProgress}}",
      "max": 100
    }
  ]
}
```

**Storage usage:**
```json
{
  "type": "Card",
  "title": "Storage",
  "content": {
    "type": "Flex",
    "direction": "column",
    "gap": "0.5rem",
    "children": [
      {
        "type": "Progress",
        "value": "{{state.storage.used}}",
        "max": "{{state.storage.total}}",
        "variant": "{{state.storage.used / state.storage.total > 0.9 ? 'destructive' : 'default'}}"
      },
      {
        "type": "Text",
        "variant": "muted",
        "content": "{{state.storage.used}}GB of {{state.storage.total}}GB used"
      }
    ]
  }
}
```

**Multi-step wizard:**
```json
{
  "type": "Progress",
  "value": "{{state.currentStep}}",
  "max": "{{state.totalSteps}}",
  "showLabel": true
}
```

---

## Skeleton

Loading placeholder that mimics content layout.

```json
{
  "type": "Skeleton",
  "width": "100%",
  "height": "20px"
}
```

### Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `width` | string | `100%` | Skeleton width |
| `height` | string | `20px` | Skeleton height |
| `variant` | `text` \| `circular` \| `rectangular` | `text` | Shape variant |
| `count` | number | `1` | Number of lines |
| `className` | string | - | CSS classes |

### Variants

```json
{
  "type": "Flex",
  "gap": "1rem",
  "children": [
    { "type": "Skeleton", "variant": "circular", "width": "40px", "height": "40px" },
    {
      "type": "Flex",
      "direction": "column",
      "gap": "0.5rem",
      "className": "flex-1",
      "children": [
        { "type": "Skeleton", "variant": "text", "width": "60%" },
        { "type": "Skeleton", "variant": "text", "width": "40%" }
      ]
    }
  ]
}
```

### Examples

**Card skeleton:**
```json
{
  "type": "Card",
  "content": {
    "type": "Flex",
    "direction": "column",
    "gap": "1rem",
    "children": [
      { "type": "Skeleton", "variant": "rectangular", "height": "200px" },
      { "type": "Skeleton", "variant": "text", "width": "70%" },
      { "type": "Skeleton", "variant": "text", "width": "50%" }
    ]
  }
}
```

**List skeleton:**
```json
{
  "type": "Loop",
  "items": [1, 2, 3, 4, 5],
  "itemTemplate": {
    "type": "Flex",
    "gap": "1rem",
    "className": "p-4",
    "children": [
      { "type": "Skeleton", "variant": "circular", "width": "40px", "height": "40px" },
      {
        "type": "Flex",
        "direction": "column",
        "gap": "0.25rem",
        "className": "flex-1",
        "children": [
          { "type": "Skeleton", "width": "30%", "height": "16px" },
          { "type": "Skeleton", "width": "80%", "height": "14px" }
        ]
      }
    ]
  }
}
```

**Conditional loading:**
```json
{
  "type": "Conditional",
  "condition": "{{state.isLoading}}",
  "then": {
    "type": "Flex",
    "direction": "column",
    "gap": "1rem",
    "children": [
      { "type": "Skeleton", "height": "40px" },
      { "type": "Skeleton", "height": "200px" }
    ]
  },
  "else": {
    "type": "Container",
    "children": [{ "...actual content..." }]
  }
}
```

---

## LoadingOverlay

Full-screen or section loading state with spinner.

```json
{
  "type": "LoadingOverlay",
  "text": "Loading..."
}
```

### Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `text` | string | - | Loading text |
| `fullScreen` | boolean | `false` | Cover full viewport |
| `transparent` | boolean | `false` | Transparent background |
| `className` | string | - | CSS classes |

### Examples

**Section overlay:**
```json
{
  "type": "Container",
  "className": "relative",
  "children": [
    { "...content..." },
    {
      "type": "Conditional",
      "condition": "{{state.isLoading}}",
      "then": {
        "type": "LoadingOverlay",
        "text": "Saving changes..."
      }
    }
  ]
}
```

**Full-screen overlay:**
```json
{
  "type": "Conditional",
  "condition": "{{state.$loading}}",
  "then": {
    "type": "LoadingOverlay",
    "fullScreen": true,
    "text": "Please wait..."
  }
}
```

---

## EmptyState

Display when content is empty or unavailable.

```json
{
  "type": "EmptyState",
  "icon": "FileText",
  "title": "No Documents",
  "description": "You haven't created any documents yet.",
  "action": {
    "type": "Button",
    "label": "Create Document",
    "events": {
      "onClick": [{ "type": "navigate", "to": "/documents/new" }]
    }
  }
}
```

### Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `icon` | string | - | Icon name |
| `title` | string | - | Empty state title |
| `description` | string | - | Description text |
| `action` | object | - | Action button |
| `className` | string | - | CSS classes |

### Examples

**Search no results:**
```json
{
  "type": "Conditional",
  "condition": "{{state.searchResults.length}} === 0 && {{state.searchQuery}}",
  "then": {
    "type": "EmptyState",
    "icon": "Search",
    "title": "No Results",
    "description": "No items match '{{state.searchQuery}}'. Try a different search term.",
    "action": {
      "type": "Button",
      "label": "Clear Search",
      "variant": "outline",
      "events": {
        "onClick": [
          { "type": "updateState", "path": "searchQuery", "value": "" },
          { "type": "callApi", "api": "search", "args": { "query": "" } }
        ]
      }
    }
  }
}
```

**First-time user:**
```json
{
  "type": "EmptyState",
  "icon": "Rocket",
  "title": "Welcome to Orbis!",
  "description": "Get started by creating your first project.",
  "action": {
    "type": "Flex",
    "gap": "0.5rem",
    "children": [
      {
        "type": "Button",
        "label": "Create Project",
        "events": { "onClick": [{ "type": "navigate", "to": "/projects/new" }] }
      },
      {
        "type": "Button",
        "label": "View Tutorial",
        "variant": "outline",
        "events": { "onClick": [{ "type": "openUrl", "url": "/help/getting-started" }] }
      }
    ]
  }
}
```

**Error state:**
```json
{
  "type": "EmptyState",
  "icon": "AlertTriangle",
  "title": "Something went wrong",
  "description": "{{state.error.message}}",
  "className": "text-red-600",
  "action": {
    "type": "Button",
    "label": "Try Again",
    "events": {
      "onClick": [{ "type": "callApi", "api": "getData" }]
    }
  }
}
```

---

## Patterns

### Loading State Pattern

```json
{
  "state": {
    "isLoading": { "type": "boolean", "default": true },
    "data": { "type": "array", "default": [] }
  },
  "onMount": [
    {
      "type": "callApi",
      "api": "getData",
      "onSuccess": [
        { "type": "updateState", "path": "data", "value": "$response.data" },
        { "type": "updateState", "path": "isLoading", "value": false }
      ]
    }
  ],
  "layout": {
    "type": "Conditional",
    "condition": "{{state.isLoading}}",
    "then": { "type": "Skeleton", "count": 5 },
    "else": {
      "type": "Conditional",
      "condition": "{{state.data.length}} === 0",
      "then": { "type": "EmptyState", "title": "No Data" },
      "else": { "type": "List", "dataSource": "state:data" }
    }
  }
}
```

### Toast Notifications

Toasts are triggered via actions, not components:

```json
{
  "type": "Button",
  "label": "Save",
  "events": {
    "onClick": [
      {
        "type": "callApi",
        "api": "save",
        "onSuccess": [
          { "type": "showToast", "message": "Saved successfully!", "level": "success" }
        ],
        "onError": [
          { "type": "showToast", "message": "Failed to save", "level": "error" }
        ]
      }
    ]
  }
}
```
