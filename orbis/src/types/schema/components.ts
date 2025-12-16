/**
 * Component schema definitions for the JSON UI system
 */

import type {
    BaseComponentProps,
    BooleanExpression,
    DataSource,
    Expression,
    Size,
    ButtonVariant,
    AlertVariant,
    BadgeVariant,
    TextVariant,
    HeadingLevel,
    InputType,
    FlexDirection,
    FlexJustify,
    FlexAlign,
    IconName,
    ValidationRule
} from './base';
import type {
    Action, EventHandlers
} from './actions';

// Container component
export interface ContainerSchema extends BaseComponentProps {
    type:     `Container`
    children: Array<ComponentSchema>
    events?:  Pick<EventHandlers, `onClick` | `onMouseEnter` | `onMouseLeave`>
}

// Text component
export interface TextSchema extends BaseComponentProps {
    type:     `Text`
    content:  Expression
    variant?: TextVariant
}

// Heading component
export interface HeadingSchema extends BaseComponentProps {
    type:   `Heading`
    level?: HeadingLevel
    text:   Expression
}

// Button component
export interface ButtonSchema extends BaseComponentProps {
    type:          `Button`
    label:         Expression
    variant?:      ButtonVariant
    size?:         Size
    disabled?:     BooleanExpression
    loading?:      BooleanExpression
    icon?:         IconName
    iconPosition?: `left` | `right`
    events?:       Pick<EventHandlers, `onClick`>
}

// Select option
export interface SelectOption {
    value:     string
    label:     Expression
    disabled?: boolean
}

// Field schema for forms and standalone inputs
export interface FieldSchema extends BaseComponentProps {
    type:          `Field`
    id:            string
    name:          string
    fieldType:     InputType
    label?:        Expression
    placeholder?:  Expression
    description?:  Expression
    defaultValue?: Expression
    bindTo?:       string
    required?:     boolean
    disabled?:     BooleanExpression
    readOnly?:     boolean
    options?:      Array<SelectOption>
    validation?:   ValidationRule
    events?:       Pick<EventHandlers, `onChange` | `onFocus` | `onBlur`>
}

// Form schema
export interface FormSchema extends BaseComponentProps {
    type:         `Form`
    id:           string
    fields:       Array<FieldSchema>
    layout?:      `vertical` | `horizontal` | `inline`
    submitLabel?: Expression
    cancelLabel?: Expression
    showReset?:   boolean
    actions?:     Array<FormAction>
    events?:      Pick<EventHandlers, `onSubmit`>
}

// Form action definition
export interface FormAction {
    id:       string
    type:     `callApi`
    api:      string
    method?:  `GET` | `POST` | `PUT` | `PATCH` | `DELETE`
    mapArgs?:   Array<{ from: string
        to:                   string }>
    onSuccess?: Array<Action>
    onError?:   Array<Action>
}

// Table column definition
export interface TableColumnSchema {
    key:           string
    label:         Expression
    sortable?:     boolean
    width?:        string
    align?:        `left` | `center` | `right`
    render?:       ComponentSchema
    headerRender?: ComponentSchema
}

// Table component
export interface TableSchema extends BaseComponentProps {
    type:        `Table`
    id?:         string
    columns:     Array<TableColumnSchema>
    dataSource:  DataSource
    rowKey?:     string
    pagination?: boolean | {
        pageSize?:             number
        showTotal?:            boolean
        showPageSizeSelector?: boolean
    }
    selectable?: boolean | `single` | `multiple`
    sortable?:   boolean
    searchable?: boolean
    emptyText?:  Expression
    loading?:    BooleanExpression
    events?:     Pick<EventHandlers, `onRowClick` | `onRowDoubleClick` | `onSelect` | `onPageChange` | `onSortChange`>
}

// Card component
export interface CardSchema extends BaseComponentProps {
    type:       `Card`
    title?:     Expression
    subtitle?:  Expression
    header?:    ComponentSchema
    content:    ComponentSchema
    footer?:    ComponentSchema
    hoverable?: boolean
    events?:    Pick<EventHandlers, `onClick`>
}

// List component
export interface ListSchema extends BaseComponentProps {
    type:           `List`
    dataSource:     DataSource
    itemTemplate:   ComponentSchema
    emptyTemplate?: ComponentSchema
    emptyText?:     Expression
    loading?:       BooleanExpression
    events?:        Pick<EventHandlers, `onRowClick`>
}

// Image component
export interface ImageSchema extends BaseComponentProps {
    type:      `Image`
    src:       Expression
    alt?:      Expression
    width?:    string | number
    height?:   string | number
    fit?:      `contain` | `cover` | `fill` | `none` | `scale-down`
    fallback?: string
    loading?:  `lazy` | `eager`
}

// Icon component
export interface IconSchema extends BaseComponentProps {
    type:    `Icon`
    name:    IconName
    size?:   Size
    color?:  string
    events?: Pick<EventHandlers, `onClick`>
}

// Link component
export interface LinkSchema extends BaseComponentProps {
    type:      `Link`
    href:      Expression
    text:      Expression
    external?: boolean
    icon?:     IconName
}

// Badge component
export interface BadgeSchema extends BaseComponentProps {
    type:     `Badge`
    text:     Expression
    variant?: BadgeVariant
}

// Alert component
export interface AlertSchema extends BaseComponentProps {
    type:         `Alert`
    variant:      AlertVariant
    title?:       Expression
    message:      Expression
    dismissible?: boolean
    icon?:        IconName
    events?:      Pick<EventHandlers, `onClose`>
}

// Progress component
export interface ProgressSchema extends BaseComponentProps {
    type:       `Progress`
    value:      Expression
    max?:       number
    showLabel?: boolean
    size?:      Size
}

// Tab item
export interface TabItemSchema {
    key:       string
    label:     Expression
    icon?:     IconName
    disabled?: BooleanExpression
    content:   ComponentSchema
}

// Tabs component
export interface TabsSchema extends BaseComponentProps {
    type:         `Tabs`
    items:        Array<TabItemSchema>
    defaultTab?:  string
    orientation?: `horizontal` | `vertical`
    events?:      Pick<EventHandlers, `onChange`>
}

// Accordion item
export interface AccordionItemSchema {
    key:       string
    title:     Expression
    content:   ComponentSchema
    disabled?: BooleanExpression
}

// Accordion component
export interface AccordionSchema extends BaseComponentProps {
    type:         `Accordion`
    items:        Array<AccordionItemSchema>
    type_:        `single` | `multiple`
    defaultOpen?: Array<string>
    collapsible?: boolean
}

// Modal component
export interface ModalSchema extends BaseComponentProps {
    type:         `Modal`
    id:           string
    title?:       Expression
    description?: Expression
    content:      ComponentSchema
    footer?:      ComponentSchema
    size?:        `sm` | `md` | `lg` | `xl` | `full`
    closable?:    boolean
    events?:      Pick<EventHandlers, `onOpen` | `onClose`>
}

// Dropdown item
export interface DropdownItemSchema {
    key:        string
    label:      Expression
    icon?:      IconName
    disabled?:  BooleanExpression
    danger?:    boolean
    separator?: boolean
    events?:    Pick<EventHandlers, `onClick`>
}

// Dropdown component
export interface DropdownSchema extends BaseComponentProps {
    type:    `Dropdown`
    trigger: ComponentSchema
    items:   Array<DropdownItemSchema>
    align?:  `start` | `center` | `end`
}

// Tooltip component
export interface TooltipSchema extends BaseComponentProps {
    type:     `Tooltip`
    content:  Expression
    children: ComponentSchema
    side?:    `top` | `bottom` | `left` | `right`
    delayMs?: number
}

// Grid component
export interface GridSchema extends BaseComponentProps {
    type:    `Grid`
    columns:  number | { sm?: number
        md?:                  number
        lg?:                  number
        xl?:                  number }
    gap?:     string
    children: Array<ComponentSchema>
}

// Flex component
export interface FlexSchema extends BaseComponentProps {
    type:       `Flex`
    direction?: FlexDirection
    justify?:   FlexJustify
    align?:     FlexAlign
    gap?:       string
    wrap?:      boolean
    children:   Array<ComponentSchema>
}

// Spacer component
export interface SpacerSchema extends BaseComponentProps {
    type: `Spacer`
    size: Size
}

// Divider component
export interface DividerSchema extends BaseComponentProps {
    type:         `Divider`
    orientation?: `horizontal` | `vertical`
    label?:       Expression
}

// Skeleton component
export interface SkeletonSchema extends BaseComponentProps {
    type:     `Skeleton`
    width?:   string
    height?:  string
    variant?: `text` | `circular` | `rectangular`
}

// Avatar component
export interface AvatarSchema extends BaseComponentProps {
    type:      `Avatar`
    src?:      Expression
    alt?:      Expression
    fallback?: Expression
    size?:     Size
}

// Breadcrumb item
export interface BreadcrumbItemSchema {
    label: Expression
    href?: Expression
    icon?: IconName
}

// Breadcrumb component
export interface BreadcrumbSchema extends BaseComponentProps {
    type:       `Breadcrumb`
    items:      Array<BreadcrumbItemSchema>
    separator?: string
}

// Stat card component
export interface StatCardSchema extends BaseComponentProps {
    type:         `StatCard`
    title:        Expression
    value:        Expression
    change?:      Expression
    changeType?:  `increase` | `decrease` | `neutral`
    icon?:        IconName
    description?: Expression
}

// Chart component
export interface ChartSchema extends BaseComponentProps {
    type:       `Chart`
    chartType:  `line` | `bar` | `pie` | `doughnut` | `area` | `scatter`
    dataSource: DataSource
    options?:   Record<string, unknown>
}

// Empty state component
export interface EmptyStateSchema extends BaseComponentProps {
    type:         `EmptyState`
    title:        Expression
    description?: Expression
    icon?:        IconName
    action?:      ButtonSchema
}

// Loading overlay component
export interface LoadingOverlaySchema extends BaseComponentProps {
    type:     `LoadingOverlay`
    loading:  BooleanExpression
    text?:    Expression
    children: ComponentSchema
}

// Conditional component
export interface ConditionalSchema extends BaseComponentProps {
    type:      `Conditional`
    condition: Expression
    then:      ComponentSchema
    else?:     ComponentSchema
}

// Loop component
export interface LoopSchema extends BaseComponentProps {
    type:           `Loop`
    dataSource:     DataSource
    itemVar?:       string
    indexVar?:      string
    template:       ComponentSchema
    emptyTemplate?: ComponentSchema
}

// Slot component (for layout slots)
export interface SlotSchema extends BaseComponentProps {
    type:      `Slot`
    name:      string
    fallback?: ComponentSchema
}

// Fragment component (invisible container)
export interface FragmentSchema extends BaseComponentProps {
    type:     `Fragment`
    children: Array<ComponentSchema>
}

// Custom component (for plugin-defined components)
export interface CustomSchema extends BaseComponentProps {
    type:      `Custom`
    component: string
    props?:    Record<string, unknown>
}

// Section component (semantic grouping with title)
export interface SectionSchema extends BaseComponentProps {
    type:              `Section`
    title?:            Expression
    description?:      Expression
    children:          Array<ComponentSchema>
    collapsible?:      boolean
    defaultCollapsed?: boolean
}

// Page header component
export interface PageHeaderSchema extends BaseComponentProps {
    type:        `PageHeader`
    title:       Expression
    subtitle?:   Expression
    breadcrumb?: Array<BreadcrumbItemSchema>
    actions?:    Array<ButtonSchema>
    backLink?:   Expression
}

// Data display component
export interface DataDisplaySchema extends BaseComponentProps {
    type:      `DataDisplay`
    label:     Expression
    value:     Expression
    copyable?: boolean
    prefix?:   ComponentSchema
    suffix?:   ComponentSchema
}

// Union of all component schemas
export type ComponentSchema =
    | ContainerSchema
    | TextSchema
    | HeadingSchema
    | ButtonSchema
    | FieldSchema
    | FormSchema
    | TableSchema
    | CardSchema
    | ListSchema
    | ImageSchema
    | IconSchema
    | LinkSchema
    | BadgeSchema
    | AlertSchema
    | ProgressSchema
    | TabsSchema
    | AccordionSchema
    | ModalSchema
    | DropdownSchema
    | TooltipSchema
    | GridSchema
    | FlexSchema
    | SpacerSchema
    | DividerSchema
    | SkeletonSchema
    | AvatarSchema
    | BreadcrumbSchema
    | StatCardSchema
    | ChartSchema
    | EmptyStateSchema
    | LoadingOverlaySchema
    | ConditionalSchema
    | LoopSchema
    | SlotSchema
    | FragmentSchema
    | CustomSchema
    | SectionSchema
    | PageHeaderSchema
    | DataDisplaySchema;
