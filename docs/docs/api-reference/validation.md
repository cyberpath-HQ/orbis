---
sidebar_position: 6
title: Validation Rules
description: Form validation reference
---

## Validation Rules

Complete reference for form field validation in Orbis.

## Basic Validation

Add validation to fields:

```json
{
  "type": "Field",
  "fieldType": "text",
  "name": "email",
  "label": "Email",
  "required": true,
  "validation": [
    { "type": "email", "message": "Enter a valid email address" }
  ]
}
```

## Required Fields

```json
{
  "type": "Field",
  "name": "username",
  "required": true,
  "requiredMessage": "Username is required"
}
```

## Built-in Validators

### email

Validates email format.

```json
{ "type": "email", "message": "Invalid email format" }
```

### url

Validates URL format.

```json
{ "type": "url", "message": "Enter a valid URL" }
```

### minLength

Minimum string length.

```json
{ "type": "minLength", "value": 8, "message": "At least 8 characters required" }
```

### maxLength

Maximum string length.

```json
{ "type": "maxLength", "value": 100, "message": "Maximum 100 characters" }
```

### min

Minimum numeric value.

```json
{ "type": "min", "value": 0, "message": "Must be 0 or greater" }
```

### max

Maximum numeric value.

```json
{ "type": "max", "value": 100, "message": "Must be 100 or less" }
```

### pattern

Regular expression pattern.

```json
{ "type": "pattern", "value": "^[A-Z0-9]+$", "message": "Uppercase letters and numbers only" }
```

### match

Match another field's value.

```json
{ "type": "match", "field": "password", "message": "Passwords must match" }
```

### custom

Custom validation function.

```json
{ "type": "custom", "validate": "{{$value.length >= 3 && $value.includes('@')}}", "message": "Invalid input" }
```

## Validation Examples

### Text Field

```json
{
  "type": "Field",
  "fieldType": "text",
  "name": "username",
  "label": "Username",
  "required": true,
  "validation": [
    { "type": "minLength", "value": 3, "message": "At least 3 characters" },
    { "type": "maxLength", "value": 20, "message": "Maximum 20 characters" },
    { "type": "pattern", "value": "^[a-zA-Z0-9_]+$", "message": "Letters, numbers, and underscores only" }
  ]
}
```

### Email Field

```json
{
  "type": "Field",
  "fieldType": "email",
  "name": "email",
  "label": "Email Address",
  "required": true,
  "requiredMessage": "Email is required",
  "validation": [
    { "type": "email", "message": "Enter a valid email address" }
  ]
}
```

### Password Field

```json
{
  "type": "Field",
  "fieldType": "password",
  "name": "password",
  "label": "Password",
  "required": true,
  "validation": [
    { "type": "minLength", "value": 8, "message": "At least 8 characters" },
    { "type": "pattern", "value": "(?=.*[0-9])", "message": "Must contain a number" },
    { "type": "pattern", "value": "(?=.*[A-Z])", "message": "Must contain uppercase letter" }
  ]
}
```

### Confirm Password

```json
{
  "type": "Field",
  "fieldType": "password",
  "name": "confirmPassword",
  "label": "Confirm Password",
  "required": true,
  "validation": [
    { "type": "match", "field": "password", "message": "Passwords must match" }
  ]
}
```

### Number Field

```json
{
  "type": "Field",
  "fieldType": "number",
  "name": "age",
  "label": "Age",
  "required": true,
  "validation": [
    { "type": "min", "value": 18, "message": "Must be 18 or older" },
    { "type": "max", "value": 120, "message": "Invalid age" }
  ]
}
```

### URL Field

```json
{
  "type": "Field",
  "fieldType": "url",
  "name": "website",
  "label": "Website",
  "validation": [
    { "type": "url", "message": "Enter a valid URL" },
    { "type": "pattern", "value": "^https://", "message": "Must use HTTPS" }
  ]
}
```

### Phone Number

```json
{
  "type": "Field",
  "fieldType": "tel",
  "name": "phone",
  "label": "Phone Number",
  "validation": [
    { "type": "pattern", "value": "^\\+?[1-9]\\d{1,14}$", "message": "Enter a valid phone number" }
  ]
}
```

### Textarea

```json
{
  "type": "Field",
  "fieldType": "textarea",
  "name": "bio",
  "label": "Bio",
  "validation": [
    { "type": "maxLength", "value": 500, "message": "Maximum 500 characters" }
  ]
}
```

## Validation Patterns

### Common Patterns

| Pattern | Use Case |
|---------|----------|
| `^[a-zA-Z]+$` | Letters only |
| `^[0-9]+$` | Numbers only |
| `^[a-zA-Z0-9]+$` | Alphanumeric |
| `^[a-zA-Z0-9_]+$` | Alphanumeric + underscore |
| `^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$` | Email |
| `^https?://` | URL with protocol |
| `^\\d{5}(-\\d{4})?$` | US ZIP code |
| `^\\d{4}-\\d{2}-\\d{2}$` | Date (YYYY-MM-DD) |

### Password Patterns

| Pattern | Requirement |
|---------|-------------|
| `(?=.*[0-9])` | At least one number |
| `(?=.*[a-z])` | At least one lowercase |
| `(?=.*[A-Z])` | At least one uppercase |
| `(?=.*[@#$%^&+=])` | At least one special char |
| `.{8,}` | At least 8 characters |

## Form-Level Validation

### Validate on Submit

```json
{
  "type": "Button",
  "text": "Submit",
  "events": {
    "onClick": [
      {
        "type": "validateForm",
        "formId": "myForm",
        "onValid": [
          { "type": "callApi", "api": "submit" }
        ],
        "onInvalid": [
          { "type": "showToast", "message": "Please fix errors", "level": "warning" }
        ]
      }
    ]
  }
}
```

### Conditional Submit Button

```json
{
  "type": "Button",
  "text": "Submit",
  "disabled": "{{!form.myForm.$valid}}"
}
```

## Error Display

### Per-Field Errors

```json
{
  "type": "Conditional",
  "conditions": [
    {
      "when": "{{form.myForm.$errors.email}}",
      "render": {
        "type": "Text",
        "text": "{{form.myForm.$errors.email}}",
        "className": "text-red-500 text-sm"
      }
    }
  ]
}
```

### Summary Errors

```json
{
  "type": "Conditional",
  "conditions": [
    {
      "when": "{{!form.myForm.$valid && form.myForm.$dirty}}",
      "render": {
        "type": "Alert",
        "variant": "destructive",
        "title": "Validation Errors",
        "message": "Please correct the highlighted fields"
      }
    }
  ]
}
```

## Validation Timing

### On Change (Default)

Validation runs on every change:

```json
{
  "type": "Field",
  "name": "email",
  "validateOn": "change",
  "validation": [{ "type": "email" }]
}
```

### On Blur

Validation runs when field loses focus:

```json
{
  "type": "Field",
  "name": "email",
  "validateOn": "blur",
  "validation": [{ "type": "email" }]
}
```

### On Submit

Validation runs only on form submit:

```json
{
  "type": "Field",
  "name": "email",
  "validateOn": "submit",
  "validation": [{ "type": "email" }]
}
```

## Custom Validation

### Expression-Based

```json
{
  "type": "Field",
  "name": "endDate",
  "validation": [
    {
      "type": "custom",
      "validate": "{{new Date($value) > new Date(form.myForm.startDate)}}",
      "message": "End date must be after start date"
    }
  ]
}
```

### Complex Rules

```json
{
  "type": "Field",
  "name": "username",
  "validation": [
    {
      "type": "custom",
      "validate": "{{!['admin', 'root', 'system'].includes($value.toLowerCase())}}",
      "message": "This username is reserved"
    }
  ]
}
```

## Form State

### $valid

True if all fields pass validation:

```json
"{{form.myForm.$valid}}"
```

### $dirty

True if any field has been modified:

```json
"{{form.myForm.$dirty}}"
```

### $errors

Object containing all field errors:

```json
"{{form.myForm.$errors}}"           // All errors
"{{form.myForm.$errors.email}}"     // Specific field error
```

## Best Practices

1. **Validate early** - Provide immediate feedback
2. **Clear messages** - Tell users how to fix errors
3. **Match UX expectations** - Use appropriate timing
4. **Test edge cases** - Empty, null, boundary values
5. **Accessible errors** - Screen reader compatible
6. **Consistent patterns** - Same rules across forms
