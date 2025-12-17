---
sidebar_position: 3
title: Form Components
description: Form and Field components for user input
---

# Form Components

Form components handle user input with validation and state binding.

## Form

A container for form fields with validation and submission handling.

```json
{
  "type": "Form",
  "id": "login-form",
  "fields": [
    {
      "name": "email",
      "fieldType": "text",
      "label": "Email",
      "placeholder": "you@example.com",
      "bindTo": "formData.email",
      "validation": {
        "required": { "message": "Email is required" },
        "email": { "message": "Please enter a valid email" }
      }
    },
    {
      "name": "password",
      "fieldType": "password",
      "label": "Password",
      "bindTo": "formData.password",
      "validation": {
        "required": { "message": "Password is required" },
        "minLength": { "value": 8, "message": "Password must be at least 8 characters" }
      }
    }
  ],
  "events": {
    "onSubmit": [
      { "type": "callApi", "api": "auth.login", "args": "{{state.formData}}" }
    ]
  }
}
```

### Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `id` | string | - | Form identifier |
| `fields` | array | `[]` | Form field definitions |
| `layout` | `vertical` \| `horizontal` \| `inline` | `vertical` | Form layout |
| `className` | string | - | CSS classes |
| `events` | object | - | Event handlers |

### Events

| Event | Trigger | Payload |
|-------|---------|---------|
| `onSubmit` | Form submission | Form data |
| `onChange` | Any field change | `{ field, value }` |

---

## Field

Individual form field with various input types.

```json
{
  "name": "username",
  "fieldType": "text",
  "label": "Username",
  "placeholder": "Enter username",
  "bindTo": "formData.username"
}
```

### Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `name` | string | - | Field name (required) |
| `fieldType` | string | `text` | Input type |
| `label` | string | - | Field label |
| `placeholder` | string | - | Placeholder text |
| `bindTo` | string | - | State path for binding |
| `defaultValue` | any | - | Default value |
| `disabled` | boolean \| string | `false` | Disabled state |
| `required` | boolean | `false` | Required field |
| `validation` | object | - | Validation rules |
| `options` | array | - | Options (for select, radio, checkbox-group) |
| `events` | object | - | Event handlers |
| `className` | string | - | CSS classes |

### Field Types

#### Text Input

```json
{
  "name": "name",
  "fieldType": "text",
  "label": "Full Name",
  "placeholder": "John Doe",
  "bindTo": "form.name"
}
```

#### Password

```json
{
  "name": "password",
  "fieldType": "password",
  "label": "Password",
  "bindTo": "form.password"
}
```

#### Email

```json
{
  "name": "email",
  "fieldType": "email",
  "label": "Email Address",
  "placeholder": "you@example.com",
  "bindTo": "form.email",
  "validation": {
    "email": { "message": "Invalid email format" }
  }
}
```

#### Number

```json
{
  "name": "quantity",
  "fieldType": "number",
  "label": "Quantity",
  "bindTo": "form.quantity",
  "min": 1,
  "max": 100,
  "step": 1
}
```

#### Textarea

```json
{
  "name": "description",
  "fieldType": "textarea",
  "label": "Description",
  "placeholder": "Enter a description...",
  "bindTo": "form.description",
  "rows": 4
}
```

#### Select

```json
{
  "name": "category",
  "fieldType": "select",
  "label": "Category",
  "bindTo": "form.category",
  "placeholder": "Select a category",
  "options": [
    { "value": "electronics", "label": "Electronics" },
    { "value": "clothing", "label": "Clothing" },
    { "value": "books", "label": "Books" }
  ]
}
```

**Dynamic options from state:**
```json
{
  "name": "category",
  "fieldType": "select",
  "label": "Category",
  "bindTo": "form.category",
  "optionsSource": "state:categories"
}
```

#### Checkbox

```json
{
  "name": "agree",
  "fieldType": "checkbox",
  "label": "I agree to the terms and conditions",
  "bindTo": "form.agree"
}
```

#### Checkbox Group

```json
{
  "name": "features",
  "fieldType": "checkbox-group",
  "label": "Features",
  "bindTo": "form.features",
  "options": [
    { "value": "notifications", "label": "Email notifications" },
    { "value": "newsletter", "label": "Weekly newsletter" },
    { "value": "updates", "label": "Product updates" }
  ]
}
```

#### Radio Group

```json
{
  "name": "plan",
  "fieldType": "radio",
  "label": "Select Plan",
  "bindTo": "form.plan",
  "options": [
    { "value": "free", "label": "Free - $0/month" },
    { "value": "pro", "label": "Pro - $10/month" },
    { "value": "enterprise", "label": "Enterprise - $50/month" }
  ]
}
```

#### Switch/Toggle

```json
{
  "name": "darkMode",
  "fieldType": "switch",
  "label": "Dark Mode",
  "bindTo": "settings.darkMode"
}
```

#### Date

```json
{
  "name": "startDate",
  "fieldType": "date",
  "label": "Start Date",
  "bindTo": "form.startDate"
}
```

#### Date Range

```json
{
  "name": "dateRange",
  "fieldType": "daterange",
  "label": "Date Range",
  "bindTo": "form.dateRange"
}
```

#### File Upload

```json
{
  "name": "avatar",
  "fieldType": "file",
  "label": "Profile Picture",
  "bindTo": "form.avatar",
  "accept": "image/*",
  "maxSize": 5242880
}
```

#### Color Picker

```json
{
  "name": "accentColor",
  "fieldType": "color",
  "label": "Accent Color",
  "bindTo": "form.accentColor"
}
```

#### Slider/Range

```json
{
  "name": "volume",
  "fieldType": "range",
  "label": "Volume",
  "bindTo": "form.volume",
  "min": 0,
  "max": 100,
  "step": 5
}
```

---

## Validation Rules

### Built-in Rules

```json
{
  "validation": {
    "required": { "message": "This field is required" },
    "minLength": { "value": 3, "message": "Minimum 3 characters" },
    "maxLength": { "value": 100, "message": "Maximum 100 characters" },
    "min": { "value": 0, "message": "Must be at least 0" },
    "max": { "value": 999, "message": "Must be at most 999" },
    "pattern": { "value": "^[A-Za-z]+$", "message": "Letters only" },
    "email": { "message": "Invalid email format" },
    "url": { "message": "Invalid URL format" }
  }
}
```

### Rule Reference

| Rule | Parameters | Description |
|------|------------|-------------|
| `required` | `message` | Field must have a value |
| `minLength` | `value`, `message` | Minimum string length |
| `maxLength` | `value`, `message` | Maximum string length |
| `min` | `value`, `message` | Minimum number value |
| `max` | `value`, `message` | Maximum number value |
| `pattern` | `value` (regex), `message` | Must match regex |
| `email` | `message` | Valid email format |
| `url` | `message` | Valid URL format |
| `custom` | `expression`, `message` | Custom validation expression |

### Custom Validation

```json
{
  "name": "confirmPassword",
  "fieldType": "password",
  "label": "Confirm Password",
  "bindTo": "form.confirmPassword",
  "validation": {
    "required": { "message": "Please confirm password" },
    "custom": {
      "expression": "{{state.form.password}} === {{state.form.confirmPassword}}",
      "message": "Passwords must match"
    }
  }
}
```

---

## Field Events

| Event | Trigger | Payload |
|-------|---------|---------|
| `onChange` | Value changes | `{ value, name }` |
| `onBlur` | Field loses focus | `{ value, name }` |
| `onFocus` | Field gains focus | `{ name }` |

### Example with Events

```json
{
  "name": "search",
  "fieldType": "text",
  "label": "Search",
  "bindTo": "filters.search",
  "events": {
    "onChange": [
      {
        "type": "debouncedAction",
        "delay": 300,
        "action": {
          "type": "callApi",
          "api": "search",
          "args": { "query": "{{state.filters.search}}" }
        }
      }
    ]
  }
}
```

---

## Form Patterns

### Login Form

```json
{
  "type": "Card",
  "className": "w-full max-w-md",
  "title": "Sign In",
  "content": {
    "type": "Form",
    "id": "login",
    "fields": [
      {
        "name": "email",
        "fieldType": "email",
        "label": "Email",
        "placeholder": "you@example.com",
        "bindTo": "login.email",
        "validation": {
          "required": { "message": "Email is required" },
          "email": { "message": "Invalid email" }
        }
      },
      {
        "name": "password",
        "fieldType": "password",
        "label": "Password",
        "bindTo": "login.password",
        "validation": {
          "required": { "message": "Password is required" }
        }
      },
      {
        "name": "remember",
        "fieldType": "checkbox",
        "label": "Remember me",
        "bindTo": "login.remember"
      }
    ],
    "events": {
      "onSubmit": [
        { "type": "setLoading", "loading": true },
        {
          "type": "callApi",
          "api": "auth.login",
          "args": "{{state.login}}",
          "onSuccess": [
            { "type": "navigate", "to": "/dashboard" }
          ],
          "onError": [
            { "type": "showToast", "message": "Invalid credentials", "level": "error" }
          ]
        },
        { "type": "setLoading", "loading": false }
      ]
    }
  },
  "footer": {
    "type": "Button",
    "label": "Sign In",
    "type": "submit",
    "className": "w-full"
  }
}
```

### Settings Form

```json
{
  "type": "Form",
  "id": "settings",
  "layout": "vertical",
  "fields": [
    {
      "name": "displayName",
      "fieldType": "text",
      "label": "Display Name",
      "bindTo": "settings.displayName"
    },
    {
      "name": "bio",
      "fieldType": "textarea",
      "label": "Bio",
      "placeholder": "Tell us about yourself",
      "bindTo": "settings.bio",
      "rows": 3
    },
    {
      "name": "timezone",
      "fieldType": "select",
      "label": "Timezone",
      "bindTo": "settings.timezone",
      "optionsSource": "state:timezones"
    },
    {
      "name": "emailNotifications",
      "fieldType": "switch",
      "label": "Email Notifications",
      "bindTo": "settings.emailNotifications"
    },
    {
      "name": "theme",
      "fieldType": "radio",
      "label": "Theme",
      "bindTo": "settings.theme",
      "options": [
        { "value": "light", "label": "Light" },
        { "value": "dark", "label": "Dark" },
        { "value": "system", "label": "System" }
      ]
    }
  ],
  "events": {
    "onSubmit": [
      {
        "type": "callApi",
        "api": "user.updateSettings",
        "args": "{{state.settings}}",
        "onSuccess": [
          { "type": "showToast", "message": "Settings saved!", "level": "success" }
        ]
      }
    ]
  }
}
```

### Inline Search

```json
{
  "type": "Form",
  "layout": "inline",
  "fields": [
    {
      "name": "query",
      "fieldType": "text",
      "placeholder": "Search...",
      "bindTo": "search.query",
      "className": "w-64"
    },
    {
      "name": "category",
      "fieldType": "select",
      "placeholder": "All categories",
      "bindTo": "search.category",
      "options": [
        { "value": "all", "label": "All" },
        { "value": "docs", "label": "Documents" },
        { "value": "images", "label": "Images" }
      ]
    }
  ],
  "events": {
    "onSubmit": [
      { "type": "callApi", "api": "search", "args": "{{state.search}}" }
    ]
  }
}
```

### Multi-Step Form

```json
{
  "type": "Container",
  "children": [
    {
      "type": "Conditional",
      "condition": "{{state.step}} === 1",
      "then": {
        "type": "Form",
        "fields": [
          { "name": "name", "fieldType": "text", "label": "Name", "bindTo": "wizard.name" },
          { "name": "email", "fieldType": "email", "label": "Email", "bindTo": "wizard.email" }
        ],
        "events": {
          "onSubmit": [
            { "type": "validateForm", "form": "step1" },
            { "type": "updateState", "path": "step", "value": 2 }
          ]
        }
      }
    },
    {
      "type": "Conditional",
      "condition": "{{state.step}} === 2",
      "then": {
        "type": "Form",
        "fields": [
          { "name": "address", "fieldType": "text", "label": "Address", "bindTo": "wizard.address" },
          { "name": "city", "fieldType": "text", "label": "City", "bindTo": "wizard.city" }
        ],
        "events": {
          "onSubmit": [
            { "type": "callApi", "api": "submit", "args": "{{state.wizard}}" }
          ]
        }
      }
    }
  ]
}
```

---

## Form Actions

### validateForm

Validate form without submission:

```json
{
  "type": "validateForm",
  "form": "myForm"
}
```

### resetForm

Reset form to defaults:

```json
{
  "type": "resetForm",
  "form": "myForm"
}
```

### Usage

```json
{
  "type": "Flex",
  "gap": "0.5rem",
  "children": [
    {
      "type": "Button",
      "label": "Reset",
      "variant": "outline",
      "events": {
        "onClick": [{ "type": "resetForm", "form": "myForm" }]
      }
    },
    {
      "type": "Button",
      "label": "Submit",
      "events": {
        "onClick": [
          { "type": "validateForm", "form": "myForm" },
          { "type": "callApi", "api": "submit" }
        ]
      }
    }
  ]
}
```
