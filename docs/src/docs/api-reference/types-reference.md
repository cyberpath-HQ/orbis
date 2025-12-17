---
sidebar_position: 8
title: Type Reference
description: TypeScript type definitions for Orbis schemas
---

## Type Reference

Complete TypeScript type definitions for Orbis plugin development.

## Core Types

### PageDefinition

Complete page schema definition.

```typescript
interface PageDefinition {
  id: string;
  title: string;
  description?: string;
  icon?: string;
  initialState?: Record<string, any>;
  stateConfig?: StateConfig;
  onMount?: ActionSchema[];
  onUnmount?: ActionSchema[];
  layout?: LayoutConfig;
  children: ComponentSchema[];
}
```

### PluginManifest

Plugin configuration.

```typescript
interface PluginManifest {
  id: string;
  name: string;
  version: string;
  description?: string;
  author?: string;
  license?: string;
  homepage?: string;
  repository?: string;
  permissions?: Permission[];
  routes?: RouteDefinition[];
  apis?: ApiDefinition[];
  settings?: SettingsDefinition;
}
```

### StateConfig

State configuration options.

```typescript
interface StateConfig {
  persist?: string[];      // Fields to persist across sessions
  sync?: boolean;          // Enable cross-tab sync
}
```

## Component Types

### ComponentSchema

Base component schema (union type).

```typescript
type ComponentSchema =
  | ContainerSchema
  | FlexSchema
  | GridSchema
  | TextSchema
  | HeadingSchema
  | ButtonSchema
  | LinkSchema
  | FormSchema
  | FieldSchema
  | TableSchema
  | CardSchema
  | AlertSchema
  | ModalSchema
  | TabsSchema
  | LoopSchema
  | ConditionalSchema
  | IconSchema
  | ImageSchema
  | BadgeSchema
  | AvatarSchema
  | ProgressSchema
  | SkeletonSchema
  | TooltipSchema
  | DropdownSchema
  | BreadcrumbSchema
  | PageHeaderSchema
  | AccordionSchema
  | DividerSchema
  | SpacerSchema
  | SectionSchema
  | ListSchema
  | StatCardSchema
  | ChartSchema
  | LoadingOverlaySchema
  | EmptyStateSchema
  | DataDisplaySchema
  | FragmentSchema
  | SlotSchema
  | CustomSchema;
```

### BaseComponentSchema

Common properties for all components.

```typescript
interface BaseComponentSchema {
  type: string;
  id?: string;
  className?: string;
  style?: Record<string, string>;
  visible?: string | boolean;
  testId?: string;
}
```

## Layout Components

### ContainerSchema

```typescript
interface ContainerSchema extends BaseComponentSchema {
  type: 'Container';
  maxWidth?: 'sm' | 'md' | 'lg' | 'xl' | '2xl' | 'full';
  padding?: Size;
  center?: boolean;
  children?: ComponentSchema[];
}
```

### FlexSchema

```typescript
interface FlexSchema extends BaseComponentSchema {
  type: 'Flex';
  direction?: 'row' | 'column' | 'row-reverse' | 'column-reverse';
  justify?: 'start' | 'end' | 'center' | 'between' | 'around' | 'evenly';
  align?: 'start' | 'end' | 'center' | 'stretch' | 'baseline';
  wrap?: boolean | 'wrap' | 'nowrap' | 'wrap-reverse';
  gap?: Size;
  children?: ComponentSchema[];
}
```

### GridSchema

```typescript
interface GridSchema extends BaseComponentSchema {
  type: 'Grid';
  columns?: number | string;
  rows?: number | string;
  gap?: Size;
  columnGap?: Size;
  rowGap?: Size;
  children?: ComponentSchema[];
}
```

## Typography Components

### TextSchema

```typescript
interface TextSchema extends BaseComponentSchema {
  type: 'Text';
  text: string;
  variant?: 'default' | 'muted' | 'lead' | 'small' | 'large';
  weight?: 'normal' | 'medium' | 'semibold' | 'bold';
  align?: 'left' | 'center' | 'right' | 'justify';
  truncate?: boolean;
  lines?: number;
}
```

### HeadingSchema

```typescript
interface HeadingSchema extends BaseComponentSchema {
  type: 'Heading';
  text: string;
  level?: 1 | 2 | 3 | 4 | 5 | 6;
  variant?: 'default' | 'gradient';
}
```

## Form Components

### FormSchema

```typescript
interface FormSchema extends BaseComponentSchema {
  type: 'Form';
  id: string;
  children?: ComponentSchema[];
  events?: {
    onSubmit?: ActionSchema[];
    onChange?: ActionSchema[];
  };
}
```

### FieldSchema

```typescript
interface FieldSchema extends BaseComponentSchema {
  type: 'Field';
  fieldType: FieldType;
  name: string;
  label?: string;
  placeholder?: string;
  value?: any;
  defaultValue?: any;
  required?: boolean;
  requiredMessage?: string;
  disabled?: string | boolean;
  readOnly?: boolean;
  validation?: ValidationRule[];
  validateOn?: 'change' | 'blur' | 'submit';
  options?: SelectOption[];
  events?: {
    onChange?: ActionSchema[];
    onBlur?: ActionSchema[];
    onFocus?: ActionSchema[];
  };
}

type FieldType =
  | 'text'
  | 'email'
  | 'password'
  | 'number'
  | 'tel'
  | 'url'
  | 'textarea'
  | 'select'
  | 'checkbox'
  | 'radio'
  | 'switch'
  | 'date'
  | 'time'
  | 'datetime'
  | 'file'
  | 'color'
  | 'range'
  | 'hidden';

interface SelectOption {
  value: string;
  label: string;
  disabled?: boolean;
}
```

### ValidationRule

```typescript
type ValidationRule =
  | { type: 'email'; message?: string }
  | { type: 'url'; message?: string }
  | { type: 'minLength'; value: number; message?: string }
  | { type: 'maxLength'; value: number; message?: string }
  | { type: 'min'; value: number; message?: string }
  | { type: 'max'; value: number; message?: string }
  | { type: 'pattern'; value: string; message?: string }
  | { type: 'match'; field: string; message?: string }
  | { type: 'custom'; validate: string; message?: string };
```

## Interactive Components

### ButtonSchema

```typescript
interface ButtonSchema extends BaseComponentSchema {
  type: 'Button';
  text?: string;
  icon?: string;
  iconPosition?: 'left' | 'right';
  variant?: 'default' | 'destructive' | 'outline' | 'secondary' | 'ghost' | 'link';
  size?: 'default' | 'sm' | 'lg' | 'icon';
  disabled?: string | boolean;
  loading?: string | boolean;
  events?: {
    onClick?: ActionSchema[];
  };
}
```

### ModalSchema

```typescript
interface ModalSchema extends BaseComponentSchema {
  type: 'Modal';
  id: string;
  title?: string;
  description?: string;
  size?: 'sm' | 'md' | 'lg' | 'xl' | 'full';
  closable?: boolean;
  children?: ComponentSchema[];
  footer?: ComponentSchema[];
  events?: {
    onClose?: ActionSchema[];
  };
}
```

## Action Types

### ActionSchema

```typescript
type ActionSchema =
  | UpdateStateAction
  | CallApiAction
  | NavigateAction
  | ShowToastAction
  | ShowDialogAction
  | CloseDialogAction
  | ValidateFormAction
  | ResetFormAction
  | SetLoadingAction
  | DownloadAction
  | CopyAction
  | OpenUrlAction
  | EmitAction
  | DebouncedAction
  | ConditionalAction
  | SequenceAction;
```

### UpdateStateAction

```typescript
interface UpdateStateAction {
  type: 'updateState';
  path: string;
  value: any;
}
```

### CallApiAction

```typescript
interface CallApiAction {
  type: 'callApi';
  api: string;
  method?: 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';
  params?: Record<string, any>;
  storeAs?: string;
  onStart?: ActionSchema[];
  onSuccess?: ActionSchema[];
  onError?: ActionSchema[];
  onComplete?: ActionSchema[];
}
```

### NavigateAction

```typescript
interface NavigateAction {
  type: 'navigate';
  route: string;
  params?: Record<string, any>;
  query?: Record<string, any>;
  replace?: boolean;
}
```

### ShowToastAction

```typescript
interface ShowToastAction {
  type: 'showToast';
  message: string;
  level?: 'info' | 'success' | 'warning' | 'error';
  title?: string;
  duration?: number;
}
```

### ConditionalAction

```typescript
interface ConditionalAction {
  type: 'conditional';
  condition: string;
  then: ActionSchema[];
  else?: ActionSchema[];
}
```

### SequenceAction

```typescript
interface SequenceAction {
  type: 'sequence';
  actions: ActionSchema[];
  stopOnError?: boolean;
}
```

## Utility Types

### Size

```typescript
type Size = 'none' | 'xs' | 'sm' | 'md' | 'lg' | 'xl' | '2xl';
```

### Variant

```typescript
type Variant = 'default' | 'destructive' | 'outline' | 'secondary' | 'ghost' | 'link';
```

### AlertVariant

```typescript
type AlertVariant = 'default' | 'info' | 'success' | 'warning' | 'destructive';
```

## Context Types

### EventContext

```typescript
interface EventContext {
  $value?: any;
  $event?: Event;
  $index?: number;
  $item?: any;
}
```

### ApiContext

```typescript
interface ApiContext {
  $response?: any;
  $error?: {
    message: string;
    code?: string;
    status?: number;
    details?: any;
  };
}
```

### NavigationContext

```typescript
interface NavigationContext {
  params: Record<string, string>;
  query: Record<string, string>;
  route: {
    path: string;
    hash: string;
  };
}
```

### FormContext

```typescript
interface FormContext {
  [formId: string]: {
    [fieldName: string]: any;
    $valid: boolean;
    $dirty: boolean;
    $errors: Record<string, string>;
  };
}
```

## Usage Examples

### Complete Page

```typescript
const page: PageDefinition = {
  id: 'users',
  title: 'Users',
  initialState: {
    users: [],
    loading: false
  },
  onMount: [
    { type: 'callApi', api: 'getUsers', storeAs: 'users' }
  ],
  children: [
    {
      type: 'PageHeader',
      title: 'User Management',
      actions: [
        {
          type: 'Button',
          text: 'Add User',
          events: {
            onClick: [{ type: 'navigate', route: '/users/new' }]
          }
        }
      ]
    },
    {
      type: 'Table',
      data: '{{state.users}}',
      columns: [
        { key: 'name', header: 'Name' },
        { key: 'email', header: 'Email' }
      ]
    }
  ]
};
```

### Type Guards

```typescript
function isButtonSchema(schema: ComponentSchema): schema is ButtonSchema {
  return schema.type === 'Button';
}

function isFormSchema(schema: ComponentSchema): schema is FormSchema {
  return schema.type === 'Form';
}
```
