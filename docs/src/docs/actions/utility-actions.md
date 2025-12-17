---
sidebar_position: 8
title: Utility Actions
description: Loading states, downloads, clipboard, URLs, and events
---

## Utility Actions

General-purpose actions for common operations.

## setLoading

Sets a loading state flag.

### Syntax

```json
{
  "type": "setLoading",
  "key": "saving",
  "value": true
}
```

### Properties

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `key` | string | ✅ | Loading state identifier |
| `value` | boolean | ✅ | Loading state value |

### Examples

**Basic usage:**

```json
{
  "type": "Button",
  "text": "Save",
  "loading": "{{loading.saving}}",
  "events": {
    "onClick": [
      { "type": "setLoading", "key": "saving", "value": true },
      {
        "type": "callApi",
        "api": "saveData",
        "onComplete": [
          { "type": "setLoading", "key": "saving", "value": false }
        ]
      }
    ]
  }
}
```

**Loading overlay:**

```json
{
  "type": "Conditional",
  "conditions": [
    {
      "when": "{{loading.pageLoading}}",
      "render": {
        "type": "LoadingOverlay",
        "message": "Loading..."
      }
    }
  ]
}
```

**Multiple loading states:**

```json
{
  "type": "Flex",
  "gap": "md",
  "children": [
    {
      "type": "Button",
      "text": "Save Draft",
      "loading": "{{loading.savingDraft}}",
      "disabled": "{{loading.publishing}}",
      "events": {
        "onClick": [
          { "type": "setLoading", "key": "savingDraft", "value": true },
          { "type": "callApi", "api": "saveDraft" },
          { "type": "setLoading", "key": "savingDraft", "value": false }
        ]
      }
    },
    {
      "type": "Button",
      "text": "Publish",
      "loading": "{{loading.publishing}}",
      "disabled": "{{loading.savingDraft}}",
      "events": {
        "onClick": [
          { "type": "setLoading", "key": "publishing", "value": true },
          { "type": "callApi", "api": "publish" },
          { "type": "setLoading", "key": "publishing", "value": false }
        ]
      }
    }
  ]
}
```

---

## download

Downloads a file.

### Syntax

```json
{
  "type": "download",
  "url": "/api/export/data.csv",
  "filename": "export.csv"
}
```

### Properties

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `url` | string | ✅ | File URL to download |
| `filename` | string | - | Suggested filename |

### Examples

**Download file:**

```json
{
  "type": "Button",
  "text": "Download Report",
  "icon": "Download",
  "events": {
    "onClick": [
      { "type": "download", "url": "/api/reports/monthly.pdf", "filename": "report.pdf" }
    ]
  }
}
```

**Dynamic download:**

```json
{
  "type": "Button",
  "text": "Export",
  "events": {
    "onClick": [
      {
        "type": "download",
        "url": "/api/export?format={{state.exportFormat}}&ids={{state.selectedIds}}",
        "filename": "export-{{$now | date:'YYYY-MM-DD'}}.{{state.exportFormat}}"
      }
    ]
  }
}
```

**Download from API response:**

```json
{
  "type": "callApi",
  "api": "generateExport",
  "onSuccess": [
    { "type": "download", "url": "{{$response.downloadUrl}}", "filename": "{{$response.filename}}" }
  ]
}
```

---

## copy

Copies text to clipboard.

### Syntax

```json
{
  "type": "copy",
  "text": "Text to copy"
}
```

### Properties

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `text` | string | ✅ | Text to copy |

### Examples

**Copy value:**

```json
{
  "type": "Button",
  "text": "Copy",
  "icon": "Copy",
  "events": {
    "onClick": [
      { "type": "copy", "text": "{{state.apiKey}}" },
      { "type": "showToast", "message": "Copied!", "level": "success", "duration": 2000 }
    ]
  }
}
```

**Copy link:**

```json
{
  "type": "Button",
  "text": "Share",
  "icon": "Share",
  "events": {
    "onClick": [
      { "type": "copy", "text": "{{$window.location.origin}}/share/{{state.item.id}}" },
      { "type": "showToast", "message": "Link copied to clipboard", "level": "success" }
    ]
  }
}
```

**Copy formatted text:**

```json
{
  "type": "copy",
  "text": "Name: {{state.user.name}}\nEmail: {{state.user.email}}\nPhone: {{state.user.phone}}"
}
```

---

## openUrl

Opens a URL in a new browser tab.

### Syntax

```json
{
  "type": "openUrl",
  "url": "https://example.com"
}
```

### Properties

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `url` | string | ✅ | URL to open |
| `target` | string | - | Target window (default: `_blank`) |

### Examples

**Open external link:**

```json
{
  "type": "Button",
  "text": "Documentation",
  "icon": "ExternalLink",
  "events": {
    "onClick": [
      { "type": "openUrl", "url": "https://docs.example.com" }
    ]
  }
}
```

**Dynamic URL:**

```json
{
  "type": "Button",
  "text": "View Profile",
  "events": {
    "onClick": [
      { "type": "openUrl", "url": "https://github.com/{{state.user.githubUsername}}" }
    ]
  }
}
```

**Open in same tab:**

```json
{
  "type": "openUrl",
  "url": "https://example.com",
  "target": "_self"
}
```

---

## emit

Emits a custom event for inter-component communication.

### Syntax

```json
{
  "type": "emit",
  "event": "itemSelected",
  "data": { "id": "{{state.item.id}}" }
}
```

### Properties

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `event` | string | ✅ | Event name |
| `data` | any | - | Event payload |

### Examples

**Emit selection event:**

```json
{
  "type": "Button",
  "text": "Select",
  "events": {
    "onClick": [
      { "type": "emit", "event": "itemSelected", "data": "{{state.currentItem}}" }
    ]
  }
}
```

**Listen for event:**

```json
{
  "events": {
    "onEvent:itemSelected": [
      { "type": "updateState", "path": "selectedItem", "value": "{{$event.data}}" }
    ]
  }
}
```

**Parent-child communication:**

```json
{
  "type": "emit",
  "event": "formSubmitted",
  "data": {
    "formId": "contactForm",
    "values": "{{form.contactForm}}"
  }
}
```

---

## Common Patterns

### Loading with Error Handling

```json
{
  "type": "Button",
  "text": "Submit",
  "loading": "{{loading.submit}}",
  "events": {
    "onClick": [
      { "type": "setLoading", "key": "submit", "value": true },
      {
        "type": "callApi",
        "api": "submit",
        "onSuccess": [
          { "type": "showToast", "message": "Success!", "level": "success" }
        ],
        "onError": [
          { "type": "showToast", "message": "{{$error.message}}", "level": "error" }
        ],
        "onComplete": [
          { "type": "setLoading", "key": "submit", "value": false }
        ]
      }
    ]
  }
}
```

### Copy with Feedback

```json
{
  "type": "Flex",
  "align": "center",
  "gap": "sm",
  "children": [
    {
      "type": "Text",
      "text": "{{state.code}}",
      "className": "font-mono bg-muted px-2 py-1 rounded"
    },
    {
      "type": "Button",
      "variant": "ghost",
      "size": "sm",
      "icon": "{{state.copied ? 'Check' : 'Copy'}}",
      "events": {
        "onClick": [
          { "type": "copy", "text": "{{state.code}}" },
          { "type": "updateState", "path": "copied", "value": true },
          { "type": "showToast", "message": "Copied!", "level": "success", "duration": 1500 }
        ]
      }
    }
  ]
}
```

### Export Options

```json
{
  "type": "Dropdown",
  "trigger": { "type": "Button", "text": "Export", "icon": "Download" },
  "items": [
    {
      "label": "CSV",
      "action": [{ "type": "download", "url": "/api/export?format=csv", "filename": "data.csv" }]
    },
    {
      "label": "Excel",
      "action": [{ "type": "download", "url": "/api/export?format=xlsx", "filename": "data.xlsx" }]
    },
    {
      "label": "PDF",
      "action": [{ "type": "download", "url": "/api/export?format=pdf", "filename": "data.pdf" }]
    }
  ]
}
```

### Best Practices

1. **Always reset loading states** - Use `onComplete` to ensure loading stops
2. **Show copy feedback** - Confirm clipboard actions with toasts
3. **Use meaningful event names** - Make events self-documenting
4. **Handle download errors** - Wrap downloads in try/catch patterns
5. **Validate URLs** - Ensure URLs are valid before opening
