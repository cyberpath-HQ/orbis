# Orbis DSL Cheat Sheet

> Quick reference for common patterns. See [COMPONENTS_REFERENCE.md](COMPONENTS_REFERENCE.md) for full details.

## Quick Component Reference


### Layout

| Component | Key Attributes | Events |
|-----------|----------------|--------|
| `Container` |  | @click, @mouseEnter |
| `Grid` |  |  |
| `Flex` |  |  |
| `Spacer` | size |  |
| `Divider` | label |  |

### Typography

| Component | Key Attributes | Events |
|-----------|----------------|--------|
| `Text` | variant | @click |
| `Heading` |  | @click |

### Forms

| Component | Key Attributes | Events |
|-----------|----------------|--------|
| `Form` |  | @submit |
| `Field` | label | @change, @focus |
| `Button` | label, variant, size | @click |
| `Dropdown` |  |  |

### Data Display

| Component | Key Attributes | Events |
|-----------|----------------|--------|
| `Card` |  | @click |
| `Table` |  | @rowClick, @rowDoubleClick |
| `List` |  | @rowClick |
| `Badge` | variant |  |
| `StatCard` |  |  |

### Feedback

| Component | Key Attributes | Events |
|-----------|----------------|--------|
| `Alert` | variant | @close |
| `Progress` | size |  |
| `LoadingOverlay` |  |  |

### Navigation

| Component | Key Attributes | Events |
|-----------|----------------|--------|
| `Link` |  |  |
| `Tabs` |  | @change |
| `Breadcrumb` |  |  |


## Common Patterns

### Button with action
```orbis
<Button label="Save" variant="default" @click=>{ api.saveData() } />
```

### Form with validation
```orbis
<Form id="loginForm" @submit=>{ api.login(state.formData) }>
  <Field name="email" type="email" required=true label="Email" />
  <Field name="password" type="password" required=true label="Password" />
  <Button type="submit" label="Login" />
</Form>
```

### Conditional rendering
```orbis
if state.isLoading {
  <LoadingOverlay visible=true />
} else {
  <Container>
    <Text>Content loaded!</Text>
  </Container>
}
```

