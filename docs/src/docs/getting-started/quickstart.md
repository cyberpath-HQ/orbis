---
sidebar_position: 2
title: Quickstart
description: Build your first Orbis plugin in 5 minutes
---

# Quickstart

Build your first Orbis plugin in 5 minutes. This tutorial walks you through creating a simple "Hello World" plugin with interactive UI.

## What You'll Build

A plugin that:
- Displays a greeting message
- Has a button that updates state
- Shows a counter of button clicks

## Step 1: Create the Plugin Directory

```bash
mkdir -p plugins/my-first-plugin
cd plugins/my-first-plugin
```

## Step 2: Create the Manifest

Create `manifest.json`:

```json
{
  "name": "my-first-plugin",
  "version": "1.0.0",
  "description": "My first Orbis plugin",
  "author": "Your Name",
  "pages": [
    {
      "id": "home",
      "title": "My First Plugin",
      "route": "/my-first-plugin",
      "icon": "Home",
      "state": {
        "username": {
          "type": "string",
          "default": "World"
        },
        "clickCount": {
          "type": "number",
          "default": 0
        }
      },
      "layout": {
        "type": "Container",
        "className": "p-6 max-w-2xl mx-auto",
        "children": [
          {
            "type": "Heading",
            "level": 1,
            "text": "Hello, {{state.username}}!"
          },
          {
            "type": "Text",
            "content": "Welcome to your first Orbis plugin. Click the button below to see reactivity in action.",
            "className": "text-muted-foreground mt-2"
          },
          {
            "type": "Card",
            "className": "mt-6",
            "content": {
              "type": "Flex",
              "direction": "column",
              "gap": "1rem",
              "children": [
                {
                  "type": "Field",
                  "id": "username-input",
                  "name": "username",
                  "fieldType": "text",
                  "label": "Your Name",
                  "placeholder": "Enter your name...",
                  "bindTo": "username"
                },
                {
                  "type": "Flex",
                  "align": "center",
                  "gap": "1rem",
                  "children": [
                    {
                      "type": "Button",
                      "label": "Click Me!",
                      "variant": "default",
                      "events": {
                        "onClick": [
                          {
                            "type": "updateState",
                            "path": "clickCount",
                            "value": "{{state.clickCount + 1}}"
                          },
                          {
                            "type": "showToast",
                            "message": "Button clicked {{state.clickCount + 1}} times!",
                            "level": "success"
                          }
                        ]
                      }
                    },
                    {
                      "type": "Badge",
                      "text": "Clicks: {{state.clickCount}}",
                      "variant": "secondary"
                    }
                  ]
                }
              ]
            }
          },
          {
            "type": "Alert",
            "variant": "default",
            "title": "How it works",
            "message": "This plugin uses the Orbis schema system. The UI is defined in JSON and rendered automatically. State changes trigger reactive updates.",
            "className": "mt-6"
          }
        ]
      }
    }
  ]
}
```

## Step 3: Load the Plugin

Copy your plugin to the Orbis plugins directory:

```bash
# From the project root
cp -r plugins/my-first-plugin plugins/
```

## Step 4: Start the Development Server

```bash
cd orbis
bun run tauri dev
```

## Step 5: View Your Plugin

Navigate to your plugin's route in the Orbis app. You should see:

1. A heading that greets you by name
2. An input field to change your name
3. A button that counts clicks
4. Toast notifications on each click

## Understanding the Code

### The Manifest Structure

```json
{
  "name": "my-first-plugin",     // Unique plugin identifier
  "version": "1.0.0",            // Semantic version
  "description": "...",          // Human-readable description
  "pages": [...]                 // Array of page definitions
}
```

### Page Definition

Each page has:

| Field | Description |
|-------|-------------|
| `id` | Unique page identifier |
| `title` | Display title in navigation |
| `route` | URL path for the page |
| `icon` | Icon from lucide-react |
| `state` | Initial state definition |
| `layout` | Root component schema |

### State Definition

State fields define reactive data:

```json
{
  "state": {
    "username": {
      "type": "string",    // string, number, boolean, object, array
      "default": "World"   // Initial value
    }
  }
}
```

### Component Schema

Components are defined declaratively:

```json
{
  "type": "Button",           // Component type
  "label": "Click Me!",       // Props
  "events": {                 // Event handlers
    "onClick": [/* actions */]
  }
}
```

### Expressions

Use `{{...}}` for dynamic values:

```json
{
  "text": "Hello, {{state.username}}!",
  "value": "{{state.clickCount + 1}}"
}
```

### Actions

Actions respond to events:

```json
{
  "type": "updateState",
  "path": "clickCount",
  "value": "{{state.clickCount + 1}}"
}
```

## Next Steps

Now that you've built your first plugin, explore:

- **[Project Structure](./project-structure)** - Understand the codebase
- **[Schema System](../core-concepts/schema-system)** - Deep dive into UI schemas
- **[Components](../components/overview)** - All available components
- **[Actions](../actions/overview)** - All action types

## Full Plugin Example

For a complete example, check the [hello-plugin](https://github.com/cyberpath-HQ/orbis/tree/main/plugins/hello-plugin) in the repository.
