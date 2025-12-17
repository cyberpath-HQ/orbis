---
sidebar_position: 7
title: Overlay Components
description: Modal and Tooltip components
---

# Overlay Components

Components that appear above other content.

## Modal

Dialog window for focused content or actions.

```json
{
  "type": "Modal",
  "open": "{{state.isModalOpen}}",
  "title": "Confirm Action",
  "description": "Are you sure you want to continue?",
  "content": {
    "type": "Text",
    "content": "This action cannot be undone."
  },
  "footer": {
    "type": "Flex",
    "justify": "end",
    "gap": "0.5rem",
    "children": [
      {
        "type": "Button",
        "label": "Cancel",
        "variant": "outline",
        "events": {
          "onClick": [{ "type": "updateState", "path": "isModalOpen", "value": false }]
        }
      },
      {
        "type": "Button",
        "label": "Confirm",
        "events": {
          "onClick": [
            { "type": "callApi", "api": "confirmAction" },
            { "type": "updateState", "path": "isModalOpen", "value": false }
          ]
        }
      }
    ]
  },
  "events": {
    "onClose": [{ "type": "updateState", "path": "isModalOpen", "value": false }]
  }
}
```

### Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `open` | boolean \| string | `false` | Modal visibility |
| `title` | string | - | Modal title |
| `description` | string | - | Subtitle/description |
| `content` | object | - | Modal body content |
| `footer` | object | - | Modal footer content |
| `size` | `sm` \| `md` \| `lg` \| `xl` \| `full` | `md` | Modal size |
| `closable` | boolean | `true` | Show close button |
| `closeOnOverlay` | boolean | `true` | Close on backdrop click |
| `closeOnEscape` | boolean | `true` | Close on Escape key |
| `className` | string | - | CSS classes |
| `events` | object | - | Event handlers |

### Events

| Event | Trigger |
|-------|---------|
| `onClose` | Modal close requested |
| `onOpen` | Modal opened |

### Sizes

| Size | Width |
|------|-------|
| `sm` | 400px |
| `md` | 500px |
| `lg` | 640px |
| `xl` | 800px |
| `full` | 100vw |

### Examples

**Confirmation dialog:**
```json
{
  "type": "Button",
  "label": "Delete Item",
  "variant": "destructive",
  "events": {
    "onClick": [{ "type": "updateState", "path": "deleteModalOpen", "value": true }]
  }
}
```

```json
{
  "type": "Modal",
  "open": "{{state.deleteModalOpen}}",
  "title": "Delete Item?",
  "description": "This action cannot be undone.",
  "content": {
    "type": "Alert",
    "variant": "destructive",
    "message": "The item '{{state.itemToDelete.name}}' will be permanently deleted."
  },
  "footer": {
    "type": "Flex",
    "justify": "end",
    "gap": "0.5rem",
    "children": [
      {
        "type": "Button",
        "label": "Cancel",
        "variant": "outline",
        "events": {
          "onClick": [{ "type": "updateState", "path": "deleteModalOpen", "value": false }]
        }
      },
      {
        "type": "Button",
        "label": "Delete",
        "variant": "destructive",
        "events": {
          "onClick": [
            { "type": "callApi", "api": "deleteItem", "args": { "id": "{{state.itemToDelete.id}}" } },
            { "type": "updateState", "path": "deleteModalOpen", "value": false },
            { "type": "showToast", "message": "Item deleted", "level": "success" }
          ]
        }
      }
    ]
  },
  "events": {
    "onClose": [{ "type": "updateState", "path": "deleteModalOpen", "value": false }]
  }
}
```

**Form modal:**
```json
{
  "type": "Modal",
  "open": "{{state.showEditModal}}",
  "title": "Edit Profile",
  "size": "lg",
  "content": {
    "type": "Form",
    "id": "edit-profile",
    "fields": [
      {
        "name": "name",
        "fieldType": "text",
        "label": "Display Name",
        "bindTo": "editForm.name",
        "validation": { "required": { "message": "Name is required" } }
      },
      {
        "name": "bio",
        "fieldType": "textarea",
        "label": "Bio",
        "bindTo": "editForm.bio",
        "rows": 4
      },
      {
        "name": "avatar",
        "fieldType": "file",
        "label": "Profile Picture",
        "accept": "image/*"
      }
    ]
  },
  "footer": {
    "type": "Flex",
    "justify": "end",
    "gap": "0.5rem",
    "children": [
      {
        "type": "Button",
        "label": "Cancel",
        "variant": "ghost",
        "events": {
          "onClick": [{ "type": "updateState", "path": "showEditModal", "value": false }]
        }
      },
      {
        "type": "Button",
        "label": "Save Changes",
        "events": {
          "onClick": [
            { "type": "validateForm", "form": "edit-profile" },
            { "type": "callApi", "api": "updateProfile", "args": "{{state.editForm}}" },
            { "type": "updateState", "path": "showEditModal", "value": false }
          ]
        }
      }
    ]
  },
  "events": {
    "onClose": [{ "type": "updateState", "path": "showEditModal", "value": false }]
  }
}
```

**Full-screen modal:**
```json
{
  "type": "Modal",
  "open": "{{state.previewOpen}}",
  "size": "full",
  "title": "Document Preview",
  "content": {
    "type": "Container",
    "className": "h-[80vh] overflow-auto",
    "children": [
      { "type": "Image", "src": "{{state.previewUrl}}", "className": "w-full" }
    ]
  },
  "events": {
    "onClose": [{ "type": "updateState", "path": "previewOpen", "value": false }]
  }
}
```

**Using showDialog action:**
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
        "content": "This cannot be undone.",
        "variant": "destructive",
        "confirmText": "Delete",
        "cancelText": "Keep",
        "onConfirm": [
          { "type": "callApi", "api": "deleteItem" },
          { "type": "showToast", "message": "Deleted" }
        ]
      }
    ]
  }
}
```

---

## Tooltip

Informational popup on hover.

```json
{
  "type": "Tooltip",
  "content": "This is helpful information",
  "trigger": {
    "type": "Button",
    "icon": "HelpCircle",
    "variant": "ghost",
    "size": "icon"
  }
}
```

### Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `content` | string \| object | - | Tooltip content |
| `trigger` | object | - | Trigger component |
| `side` | `top` \| `bottom` \| `left` \| `right` | `top` | Tooltip position |
| `align` | `start` \| `center` \| `end` | `center` | Alignment |
| `delayDuration` | number | `200` | Show delay (ms) |
| `className` | string | - | CSS classes |

### Examples

**Simple tooltip:**
```json
{
  "type": "Tooltip",
  "content": "Click to edit",
  "trigger": {
    "type": "Button",
    "icon": "Edit",
    "variant": "ghost",
    "size": "icon"
  }
}
```

**With rich content:**
```json
{
  "type": "Tooltip",
  "content": {
    "type": "Flex",
    "direction": "column",
    "gap": "0.25rem",
    "children": [
      { "type": "Text", "content": "Keyboard Shortcuts", "className": "font-semibold" },
      { "type": "Text", "variant": "small", "content": "⌘K - Search" },
      { "type": "Text", "variant": "small", "content": "⌘S - Save" }
    ]
  },
  "trigger": {
    "type": "Button",
    "icon": "Keyboard",
    "variant": "ghost"
  }
}
```

**On icon:**
```json
{
  "type": "Flex",
  "align": "center",
  "gap": "0.5rem",
  "children": [
    { "type": "Text", "content": "API Rate Limit" },
    {
      "type": "Tooltip",
      "content": "Maximum 100 requests per minute",
      "side": "right",
      "trigger": {
        "type": "Icon",
        "name": "Info",
        "className": "w-4 h-4 text-gray-400 cursor-help"
      }
    }
  ]
}
```

**Different positions:**
```json
{
  "type": "Flex",
  "gap": "1rem",
  "children": [
    {
      "type": "Tooltip",
      "content": "Above",
      "side": "top",
      "trigger": { "type": "Button", "label": "Top" }
    },
    {
      "type": "Tooltip",
      "content": "Below",
      "side": "bottom",
      "trigger": { "type": "Button", "label": "Bottom" }
    },
    {
      "type": "Tooltip",
      "content": "Left side",
      "side": "left",
      "trigger": { "type": "Button", "label": "Left" }
    },
    {
      "type": "Tooltip",
      "content": "Right side",
      "side": "right",
      "trigger": { "type": "Button", "label": "Right" }
    }
  ]
}
```

**On disabled button:**
```json
{
  "type": "Tooltip",
  "content": "You need admin permissions to perform this action",
  "trigger": {
    "type": "Container",
    "children": [
      {
        "type": "Button",
        "label": "Delete All",
        "variant": "destructive",
        "disabled": true
      }
    ]
  }
}
```

---

## Overlay Patterns

### Modal with Loading State

```json
{
  "type": "Modal",
  "open": "{{state.modalOpen}}",
  "title": "Processing",
  "closable": false,
  "closeOnOverlay": false,
  "closeOnEscape": false,
  "content": {
    "type": "Flex",
    "direction": "column",
    "align": "center",
    "gap": "1rem",
    "className": "py-4",
    "children": [
      { "type": "LoadingOverlay" },
      { "type": "Text", "content": "Please wait while we process your request..." }
    ]
  }
}
```

### Cascading Modals

```json
{
  "type": "Modal",
  "open": "{{state.level1Open}}",
  "title": "Select Item",
  "content": {
    "type": "Container",
    "children": [
      { "type": "List", "dataSource": "state:items" },
      {
        "type": "Button",
        "label": "Create New",
        "events": {
          "onClick": [{ "type": "updateState", "path": "level2Open", "value": true }]
        }
      }
    ]
  }
}
```

```json
{
  "type": "Modal",
  "open": "{{state.level2Open}}",
  "title": "Create Item",
  "content": { "type": "Form", "..." },
  "events": {
    "onClose": [{ "type": "updateState", "path": "level2Open", "value": false }]
  }
}
```

### Tooltip on Table Cell

```json
{
  "type": "Table",
  "dataSource": "state:items",
  "columns": [
    { "key": "name", "label": "Name" },
    {
      "key": "status",
      "label": "Status",
      "render": {
        "type": "Tooltip",
        "content": "{{$row.statusDetails}}",
        "trigger": {
          "type": "Badge",
          "text": "{{$cell}}",
          "variant": "{{$cell === 'active' ? 'success' : 'secondary'}}"
        }
      }
    }
  ]
}
```
