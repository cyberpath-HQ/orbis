/**
 * Base types and utilities for the JSON UI Schema system
 */

// State Definition Types
export type StateFieldType =
  | `string`
  | `number`
  | `boolean`
  | `object`
  | `array`;

export interface StateFieldDefinition {
    type:         StateFieldType
    default?:     unknown
    nullable?:    boolean
    description?: string
}

export type StateDefinition = Record<string, StateFieldDefinition>;

// Expression Types - uses {{path.to.value}} syntax for interpolation
export type Expression = string;

// Boolean expression for conditions
export type BooleanExpression = boolean | Expression;

// Data source reference for components that consume data
// Formats: "state:path.to.data", "prop:propName", "context:contextKey"
export type DataSource = string;

// Base properties shared by all components
export interface BaseComponentProps {
    id?:        string
    className?: string
    style?:     Record<string, string | number>
    visible?:   BooleanExpression
    testId?:    string
}

// Special values that can be used in event handlers
export type SpecialValue =
  | `$event`
  | `$event.value`
  | `$event.target`
  | `$row`
  | `$item`
  | `$index`
  | `$response`
  | `$response.data`
  | `$error`;

// Field validation rules
export interface ValidationRule {
    required?: boolean | { message: string }
    min?:       number | { value: number
        message:                  string }
    max?:       number | { value: number
        message:                  string }
    minLength?: number | { value: number
        message:                  string }
    maxLength?: number | { value: number
        message:                  string }
    pattern?:   string | { value: string
        message:                  string }
    email?:  boolean | { message: string }
    url?:    boolean | { message: string }
    custom?: {
        expression: Expression
        message:    string
    }
}

// Size and Variant Types
export type Size = `xs` | `sm` | `md` | `lg` | `xl`;

export type ButtonVariant = `default` | `destructive` | `outline` | `secondary` | `ghost` | `link`;

export type AlertVariant = `default` | `destructive`;

export type BadgeVariant = `default` | `secondary` | `destructive` | `outline`;

export type TextVariant = `body` | `caption` | `label` | `code` | `muted`;

export type HeadingLevel = 1 | 2 | 3 | 4 | 5 | 6;

export type InputType =
  | `text`
  | `password`
  | `email`
  | `number`
  | `tel`
  | `url`
  | `date`
  | `time`
  | `datetime-local`
  | `textarea`
  | `checkbox`
  | `radio`
  | `select`
  | `file`
  | `hidden`
  | `switch`;

export type FlexDirection = `row` | `column` | `row-reverse` | `column-reverse`;

export type FlexJustify = `start` | `end` | `center` | `between` | `around` | `evenly`;

export type FlexAlign = `start` | `end` | `center` | `stretch` | `baseline`;

export type TableSortDirection = `asc` | `desc`;

// Icon name from lucide-react
export type IconName = string;
