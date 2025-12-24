# Orbis DSL Component Reference

> **Auto-generated documentation** - Do not edit manually.
> Generated from `build.rs` component definitions.

## Table of Contents

- [Container](#container)
- [Grid](#grid)
- [Flex](#flex)
- [Text](#text)
- [Heading](#heading)
- [Field](#field)
- [Button](#button)
- [Form](#form)
- [Card](#card)
- [Table](#table)
- [List](#list)
- [Badge](#badge)
- [StatCard](#statcard)
- [Alert](#alert)
- [Link](#link)
- [Dropdown](#dropdown)
- [Progress](#progress)
- [LoadingOverlay](#loadingoverlay)
- [Skeleton](#skeleton)
- [EmptyState](#emptystate)
- [Icon](#icon)
- [Modal](#modal)
- [Tooltip](#tooltip)

---

## Container

A generic container element for grouping and layout purposes.

### Usage

```orbis
<Container />

// With children:
<Container>
    // content
</Container>
```

### Attributes

| Attribute | Description | Allowed Values |
|-----------|-------------|----------------|
| `id` | Unique identifier for the element | *expression* |
| `className` | CSS class name(s) for styling | *expression* |
| `visible` | Expression controlling visibility | *expression* |

### Events

| Event | Description |
|-------|-------------|
| `@click` | Triggered when the container is clicked |
| `@mouseEnter` | Triggered when mouse enters the container |
| `@mouseLeave` | Triggered when mouse leaves the container |

### Example

```orbis
<Container id="example" className={state.value} visible={state.value} @click => [state.clicked = true] />
```

---

## Grid

A CSS Grid-based layout component for creating grid layouts.

### Usage

```orbis
<Grid />

// With children:
<Grid>
    // content
</Grid>
```

### Attributes

| Attribute | Description | Allowed Values |
|-----------|-------------|----------------|
| `id` | Unique identifier for the element | *expression* |
| `className` | CSS class name(s) for styling | *expression* |
| `cols` | Number of columns (expression) | *expression* |
| `gap` | Gap between grid items | *expression* |
| `visible` | Expression controlling visibility | *expression* |

### Events

| Event | Description |
|-------|-------------|
| `@click` | Triggered when the grid is clicked |

### Example

```orbis
<Grid id="example" className={state.value} cols={state.value} @click => [state.clicked = true] />
```

---

## Flex

A Flexbox-based layout component for flexible layouts.

### Usage

```orbis
<Flex />

// With children:
<Flex>
    // content
</Flex>
```

### Attributes

| Attribute | Description | Allowed Values |
|-----------|-------------|----------------|
| `id` | Unique identifier for the element | *expression* |
| `className` | CSS class name(s) for styling | *expression* |
| `direction` | Flex direction | `row`, `column`, `row-reverse`, `column-reverse` |
| `justify` | Justify content | `start`, `end`, `center`, `between`, `around`, `evenly` |
| `align` | Align items | `start`, `end`, `center`, `stretch`, `baseline` |
| `gap` | Gap between flex items | *expression* |
| `visible` | Expression controlling visibility | *expression* |

### Events

| Event | Description |
|-------|-------------|
| `@click` | Triggered when the flex container is clicked |

### Example

```orbis
<Flex id="example" className={state.value} direction="row" @click => [state.clicked = true] />
```

---

## Text

A text display component for paragraphs and inline text.

### Usage

```orbis
<Text />

// With children:
<Text>
    // content
</Text>
```

### Attributes

| Attribute | Description | Allowed Values |
|-----------|-------------|----------------|
| `id` | Unique identifier for the element | *expression* |
| `className` | CSS class name(s) for styling | *expression* |
| `content` | Text content to display (supports expressions) | *expression* |
| `visible` | Expression controlling visibility | *expression* |

### Events

| Event | Description |
|-------|-------------|
| `@click` | Triggered when the text is clicked |

### Example

```orbis
<Text id="example" className={state.value} content={state.value} @click => [state.clicked = true] />
```

---

## Heading

A heading component for titles (h1-h6).

### Usage

```orbis
<Heading />

// With children:
<Heading>
    // content
</Heading>
```

### Attributes

| Attribute | Description | Allowed Values |
|-----------|-------------|----------------|
| `id` | Unique identifier for the element | *expression* |
| `className` | CSS class name(s) for styling | *expression* |
| `content` | Heading text content | *expression* |
| `level` | Heading level | `1`, `2`, `3`, `4`, `5`, `6` |
| `visible` | Expression controlling visibility | *expression* |

### Events

| Event | Description |
|-------|-------------|
| `@click` | Triggered when the heading is clicked |

### Example

```orbis
<Heading id="example" className={state.value} content={state.value} @click => [state.clicked = true] />
```

---

## Field

A form input field component supporting various input types.

### Usage

```orbis
<Field />

// With children:
<Field>
    // content
</Field>
```

### Attributes

| Attribute | Description | Allowed Values |
|-----------|-------------|----------------|
| `id` | Unique identifier for the element | *expression* |
| `className` | CSS class name(s) for styling | *expression* |
| `type` | Input type | `text`, `password`, `email`, `number`, `tel`, `url`, `date`, `time`, `datetime-local`, `month`, `week`, `color`, `file`, `hidden`, `textarea`, `select`, `checkbox`, `radio` |
| `fieldName` | Name of the field for form submission | *expression* |
| `label` | Label text for the field | *expression* |
| `placeholder` | Placeholder text when empty | *expression* |
| `bind` | State path to bind the value to | *expression* |
| `value` | Current value (expression) | *expression* |
| `defaultValue` | Default value on mount | *expression* |
| `disabled` | Whether the field is disabled (expression) | *expression* |
| `required` | Whether the field is required (expression) | *expression* |
| `visible` | Expression controlling visibility | *expression* |

### Events

| Event | Description |
|-------|-------------|
| `@input` | Triggered on every input change |
| `@change` | Triggered when value changes and loses focus |
| `@focus` | Triggered when field gains focus |
| `@blur` | Triggered when field loses focus |
| `@keyDown` | Triggered on key press |
| `@keyUp` | Triggered on key release |

### Example

```orbis
<Field id="example" className={state.value} type="text" @input => [state.clicked = true] />
```

---

## Button

A clickable button component.

### Usage

```orbis
<Button />

// With children:
<Button>
    // content
</Button>
```

### Attributes

| Attribute | Description | Allowed Values |
|-----------|-------------|----------------|
| `id` | Unique identifier for the element | *expression* |
| `className` | CSS class name(s) for styling | *expression* |
| `label` | Button text label | *expression* |
| `type` | Button type | `button`, `submit`, `reset` |
| `variant` | Visual variant | `primary`, `secondary`, `outline`, `ghost`, `destructive`, `link` |
| `disabled` | Whether the button is disabled (expression) | *expression* |
| `visible` | Expression controlling visibility | *expression* |

### Events

| Event | Description |
|-------|-------------|
| `@click` | Triggered when the button is clicked |
| `@mouseEnter` | Triggered when mouse enters the button |
| `@mouseLeave` | Triggered when mouse leaves the button |

### Example

```orbis
<Button id="example" className={state.value} label={state.value} @click => [state.clicked = true] />
```

---

## Form

A form container that handles submission.

### Usage

```orbis
<Form />

// With children:
<Form>
    // content
</Form>
```

### Attributes

| Attribute | Description | Allowed Values |
|-----------|-------------|----------------|
| `id` | Unique identifier for the element | *expression* |
| `className` | CSS class name(s) for styling | *expression* |
| `visible` | Expression controlling visibility | *expression* |

### Events

| Event | Description |
|-------|-------------|
| `@submit` | Triggered when the form is submitted |

### Example

```orbis
<Form id="example" className={state.value} visible={state.value} @submit => [state.clicked = true] />
```

---

## Card

A card container for grouping related content.

### Usage

```orbis
<Card />

// With children:
<Card>
    // content
</Card>
```

### Attributes

| Attribute | Description | Allowed Values |
|-----------|-------------|----------------|
| `id` | Unique identifier for the element | *expression* |
| `className` | CSS class name(s) for styling | *expression* |
| `title` | Card title (expression) | *expression* |
| `visible` | Expression controlling visibility | *expression* |

### Events

| Event | Description |
|-------|-------------|
| `@click` | Triggered when the card is clicked |

### Example

```orbis
<Card id="example" className={state.value} title={state.value} @click => [state.clicked = true] />
```

---

## Table

A data table component for displaying tabular data.

### Usage

```orbis
<Table />

// With children:
<Table>
    // content
</Table>
```

### Attributes

| Attribute | Description | Allowed Values |
|-----------|-------------|----------------|
| `id` | Unique identifier for the element | *expression* |
| `className` | CSS class name(s) for styling | *expression* |
| `data` | Array of data objects to display | *expression* |
| `columns` | Column definitions | *expression* |
| `visible` | Expression controlling visibility | *expression* |

### Events

| Event | Description |
|-------|-------------|
| `@rowClick` | Triggered when a row is clicked |
| `@cellClick` | Triggered when a cell is clicked |

### Example

```orbis
<Table id="example" className={state.value} data={state.value} @rowClick => [state.clicked = true] />
```

---

## List

A list component for displaying items.

### Usage

```orbis
<List />

// With children:
<List>
    // content
</List>
```

### Attributes

| Attribute | Description | Allowed Values |
|-----------|-------------|----------------|
| `id` | Unique identifier for the element | *expression* |
| `className` | CSS class name(s) for styling | *expression* |
| `items` | Array of items to display | *expression* |
| `visible` | Expression controlling visibility | *expression* |

### Events

| Event | Description |
|-------|-------------|
| `@itemClick` | Triggered when an item is clicked |

### Example

```orbis
<List id="example" className={state.value} items={state.value} @itemClick => [state.clicked = true] />
```

---

## Badge

A small status indicator or label component.

### Usage

```orbis
<Badge />

// With children:
<Badge>
    // content
</Badge>
```

### Attributes

| Attribute | Description | Allowed Values |
|-----------|-------------|----------------|
| `id` | Unique identifier for the element | *expression* |
| `className` | CSS class name(s) for styling | *expression* |
| `content` | Badge text content | *expression* |
| `variant` | Visual variant | `default`, `primary`, `secondary`, `success`, `warning`, `error`, `info`, `outline` |
| `visible` | Expression controlling visibility | *expression* |

### Events

| Event | Description |
|-------|-------------|
| `@click` | Triggered when the badge is clicked |

### Example

```orbis
<Badge id="example" className={state.value} content={state.value} @click => [state.clicked = true] />
```

---

## StatCard

A statistics display card with value, label, and optional trend indicator.

### Usage

```orbis
<StatCard />

// With children:
<StatCard>
    // content
</StatCard>
```

### Attributes

| Attribute | Description | Allowed Values |
|-----------|-------------|----------------|
| `id` | Unique identifier for the element | *expression* |
| `className` | CSS class name(s) for styling | *expression* |
| `title` | Stat label/title | *expression* |
| `value` | Main statistic value | *expression* |
| `change` | Change value (e.g., '+5%') | *expression* |
| `icon` | Icon name to display | *expression* |
| `trend` | Trend direction | `up`, `down`, `neutral` |
| `visible` | Expression controlling visibility | *expression* |

### Events

| Event | Description |
|-------|-------------|
| `@click` | Triggered when the stat card is clicked |

### Example

```orbis
<StatCard id="example" className={state.value} title={state.value} @click => [state.clicked = true] />
```

---

## Alert

An alert/notification message component.

### Usage

```orbis
<Alert />

// With children:
<Alert>
    // content
</Alert>
```

### Attributes

| Attribute | Description | Allowed Values |
|-----------|-------------|----------------|
| `id` | Unique identifier for the element | *expression* |
| `className` | CSS class name(s) for styling | *expression* |
| `type` | Alert type/severity | `info`, `success`, `warning`, `error` |
| `title` | Alert title | *expression* |
| `message` | Alert message content | *expression* |
| `visible` | Expression controlling visibility | *expression* |

### Events

| Event | Description |
|-------|-------------|
| `@close` | Triggered when the alert is dismissed |

### Example

```orbis
<Alert id="example" className={state.value} type="info" @close => [state.clicked = true] />
```

---

## Link

A navigation link component.

### Usage

```orbis
<Link />

// With children:
<Link>
    // content
</Link>
```

### Attributes

| Attribute | Description | Allowed Values |
|-----------|-------------|----------------|
| `id` | Unique identifier for the element | *expression* |
| `className` | CSS class name(s) for styling | *expression* |
| `href` | URL to navigate to | *expression* |
| `content` | Link text content | *expression* |
| `visible` | Expression controlling visibility | *expression* |

### Events

| Event | Description |
|-------|-------------|
| `@click` | Triggered when the link is clicked |

### Example

```orbis
<Link id="example" className={state.value} href={state.value} @click => [state.clicked = true] />
```

---

## Dropdown

A dropdown select component.

### Usage

```orbis
<Dropdown />

// With children:
<Dropdown>
    // content
</Dropdown>
```

### Attributes

| Attribute | Description | Allowed Values |
|-----------|-------------|----------------|
| `id` | Unique identifier for the element | *expression* |
| `className` | CSS class name(s) for styling | *expression* |
| `options` | Array of options to select from | *expression* |
| `value` | Currently selected value | *expression* |
| `placeholder` | Placeholder text when no selection | *expression* |
| `disabled` | Whether the dropdown is disabled | *expression* |
| `visible` | Expression controlling visibility | *expression* |

### Events

| Event | Description |
|-------|-------------|
| `@change` | Triggered when selection changes |

### Example

```orbis
<Dropdown id="example" className={state.value} options={state.value} @change => [state.clicked = true] />
```

---

## Progress

A progress indicator component.

### Usage

```orbis
<Progress />

// With children:
<Progress>
    // content
</Progress>
```

### Attributes

| Attribute | Description | Allowed Values |
|-----------|-------------|----------------|
| `id` | Unique identifier for the element | *expression* |
| `className` | CSS class name(s) for styling | *expression* |
| `value` | Current progress value | *expression* |
| `max` | Maximum progress value | *expression* |
| `visible` | Expression controlling visibility | *expression* |

### Events

*No events.*

### Example

```orbis
<Progress id="example" className={state.value} value={state.value} />
```

---

## LoadingOverlay

A loading overlay that covers its container.

### Usage

```orbis
<LoadingOverlay />

// With children:
<LoadingOverlay>
    // content
</LoadingOverlay>
```

### Attributes

| Attribute | Description | Allowed Values |
|-----------|-------------|----------------|
| `id` | Unique identifier for the element | *expression* |
| `className` | CSS class name(s) for styling | *expression* |
| `message` | Loading message to display | *expression* |
| `visible` | Expression controlling visibility | *expression* |

### Events

*No events.*

### Example

```orbis
<LoadingOverlay id="example" className={state.value} message={state.value} />
```

---

## Skeleton

A skeleton loading placeholder component.

### Usage

```orbis
<Skeleton />

// With children:
<Skeleton>
    // content
</Skeleton>
```

### Attributes

| Attribute | Description | Allowed Values |
|-----------|-------------|----------------|
| `id` | Unique identifier for the element | *expression* |
| `className` | CSS class name(s) for styling | *expression* |
| `variant` | Skeleton shape variant | `text`, `circular`, `rectangular`, `rounded` |
| `visible` | Expression controlling visibility | *expression* |

### Events

*No events.*

### Example

```orbis
<Skeleton id="example" className={state.value} variant="text" />
```

---

## EmptyState

An empty state placeholder with icon and message.

### Usage

```orbis
<EmptyState />

// With children:
<EmptyState>
    // content
</EmptyState>
```

### Attributes

| Attribute | Description | Allowed Values |
|-----------|-------------|----------------|
| `id` | Unique identifier for the element | *expression* |
| `className` | CSS class name(s) for styling | *expression* |
| `icon` | Icon to display | *expression* |
| `title` | Empty state title | *expression* |
| `description` | Empty state description | *expression* |
| `visible` | Expression controlling visibility | *expression* |

### Events

*No events.*

### Example

```orbis
<EmptyState id="example" className={state.value} icon={state.value} />
```

---

## Icon

An icon component.

### Usage

```orbis
<Icon />

// With children:
<Icon>
    // content
</Icon>
```

### Attributes

| Attribute | Description | Allowed Values |
|-----------|-------------|----------------|
| `id` | Unique identifier for the element | *expression* |
| `className` | CSS class name(s) for styling | *expression* |
| `name` | Icon name from the icon set | *expression* |
| `size` | Icon size | `xs`, `sm`, `md`, `lg`, `xl`, `2xl` |
| `visible` | Expression controlling visibility | *expression* |

### Events

| Event | Description |
|-------|-------------|
| `@click` | Triggered when the icon is clicked |

### Example

```orbis
<Icon id="example" className={state.value} name={state.value} @click => [state.clicked = true] />
```

---

## Modal

A modal dialog component.

### Usage

```orbis
<Modal />

// With children:
<Modal>
    // content
</Modal>
```

### Attributes

| Attribute | Description | Allowed Values |
|-----------|-------------|----------------|
| `id` | Unique identifier for the element | *expression* |
| `className` | CSS class name(s) for styling | *expression* |
| `title` | Modal title | *expression* |
| `open` | Whether the modal is open (expression) | *expression* |
| `visible` | Expression controlling visibility | *expression* |

### Events

| Event | Description |
|-------|-------------|
| `@close` | Triggered when the modal is closed |

### Example

```orbis
<Modal id="example" className={state.value} title={state.value} @close => [state.clicked = true] />
```

---

## Tooltip

A tooltip component that appears on hover.

### Usage

```orbis
<Tooltip />

// With children:
<Tooltip>
    // content
</Tooltip>
```

### Attributes

| Attribute | Description | Allowed Values |
|-----------|-------------|----------------|
| `id` | Unique identifier for the element | *expression* |
| `className` | CSS class name(s) for styling | *expression* |
| `content` | Tooltip content text | *expression* |
| `position` | Tooltip position | `top`, `bottom`, `left`, `right` |
| `visible` | Expression controlling visibility | *expression* |

### Events

*No events.*

### Example

```orbis
<Tooltip id="example" className={state.value} content={state.value} />
```

---

