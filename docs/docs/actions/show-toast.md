---
sidebar_position: 5
title: showToast
description: Display toast notifications
---

## showToast

Displays a toast notification to the user.

### Syntax

```json
{
  "type": "showToast",
  "message": "Operation completed!",
  "level": "success"
}
```

### Properties

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `message` | string | ✅ | Notification message |
| `level` | string | - | Toast level (default: `info`) |
| `title` | string | - | Toast title |
| `duration` | number | - | Display duration (ms) |
| `action` | object | - | Action button |

### Levels

| Level | Color | Use Case |
|-------|-------|----------|
| `info` | Blue | General information |
| `success` | Green | Success confirmations |
| `warning` | Yellow | Warnings |
| `error` | Red | Error messages |

### Examples

**Basic toast:**
```json
{
  "type": "showToast",
  "message": "Settings saved"
}
```

**Success toast:**
```json
{
  "type": "showToast",
  "message": "Item created successfully!",
  "level": "success"
}
```

**Error toast:**
```json
{
  "type": "showToast",
  "message": "Failed to save changes",
  "level": "error"
}
```

**Warning toast:**
```json
{
  "type": "showToast",
  "message": "Your session will expire in 5 minutes",
  "level": "warning"
}
```

**With title:**
```json
{
  "type": "showToast",
  "title": "Success",
  "message": "Your profile has been updated",
  "level": "success"
}
```

**With duration:**
```json
{
  "type": "showToast",
  "message": "Copied to clipboard",
  "level": "success",
  "duration": 2000
}
```

**With dynamic message:**
```json
{
  "type": "showToast",
  "message": "Welcome back, {{state.user.name}}!",
  "level": "success"
}
```

**From error response:**
```json
{
  "type": "callApi",
  "api": "saveData",
  "onError": [
    {
      "type": "showToast",
      "message": "Error: {{$error.message}}",
      "level": "error"
    }
  ]
}
```

### Common Patterns

**CRUD Operations:**

```json
{ "type": "showToast", "message": "Item created", "level": "success" }
{ "type": "showToast", "message": "Changes saved", "level": "success" }
{ "type": "showToast", "message": "Item deleted", "level": "success" }
{ "type": "showToast", "message": "Operation failed", "level": "error" }
```

**Copy to clipboard:**

```json
{
  "type": "Button",
  "icon": "Copy",
  "events": {
    "onClick": [
      { "type": "copy", "text": "{{state.shareUrl}}" },
      { "type": "showToast", "message": "Link copied!", "level": "success", "duration": 2000 }
    ]
  }
}
```

**After navigation:**

```json
{
  "onMount": [
    {
      "type": "conditional",
      "condition": "{{params.created}}",
      "then": [
        { "type": "showToast", "message": "Item created successfully!", "level": "success" }
      ]
    }
  ]
}
```

### Best Practices

1. **Be concise** - Keep messages short and clear
2. **Use appropriate levels** - Match level to message importance
3. **Confirm actions** - Show success after operations
4. **Include details in errors** - Help users understand what went wrong
5. **Don't overuse** - Avoid toast fatigue

