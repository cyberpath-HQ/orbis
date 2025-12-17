/**
 * Form utilities for TanStack Form integration
 * Converts schema ValidationRule to Zod schemas and provides form helpers
 */

import { z } from 'zod';
import type { ValidationRule } from '@/types/schema/base';
import type { FieldSchema } from '@/types/schema/components';

// Value with message type for validation rules
interface ValueWithMessage {
    value:   number
    message: string
}

// Field array item type
interface FieldArrayItem<T> {
    id:    string
    value: T
}

/**
 * Get base Zod schema for a field type
 */
function getBaseSchema(
    fieldType: FieldSchema[`fieldType`],
    rule: ValidationRule | undefined
): z.ZodTypeAny {
    switch (fieldType) {
        case `number`:
            return z.coerce.number();
        case `checkbox`:
        case `switch`:
            return z.boolean();
        case `email`:
            return z.string().email(getMessageFromRule(rule?.email, `Invalid email address`));
        case `url`:
            return z.string().url(getMessageFromRule(rule?.url, `Invalid URL`));
        case `date`:
        case `time`:
        case `datetime-local`:
            return z.string();
        case `file`:
            return z.any();
        default:
            return z.string();
    }
}

/**
 * Apply string validations to a schema
 */
function applyStringValidations(
    schema: z.ZodString,
    rule: ValidationRule
): z.ZodString {
    let result = schema;

    if (rule.minLength) {
        const {
            value,
            message,
        } = getValueAndMessage(rule.minLength, `Minimum length is`);
        result = result.min(value, message);
    }

    if (rule.maxLength) {
        const {
            value,
            message,
        } = getValueAndMessage(rule.maxLength, `Maximum length is`);
        result = result.max(value, message);
    }

    if (rule.pattern) {
        const patternValue = typeof rule.pattern === `string` ? rule.pattern : rule.pattern.value;
        const patternMessage = typeof rule.pattern === `string` ? `Invalid format` : rule.pattern.message;
        result = result.regex(new RegExp(patternValue), patternMessage);
    }

    return result;
}

/**
 * Apply number validations to a schema
 */
function applyNumberValidations(
    schema: z.ZodNumber,
    rule: ValidationRule
): z.ZodNumber {
    let result = schema;

    if (rule.min !== undefined) {
        const {
            value,
            message,
        } = getValueAndMessage(rule.min, `Minimum value is`);
        result = result.min(value, message);
    }

    if (rule.max !== undefined) {
        const {
            value,
            message,
        } = getValueAndMessage(rule.max, `Maximum value is`);
        result = result.max(value, message);
    }

    return result;
}

/**
 * Convert a ValidationRule to a Zod schema
 */
export function validationRuleToZod(
    rule: ValidationRule | undefined,
    fieldType: FieldSchema[`fieldType`]
): z.ZodTypeAny {
    let schema = getBaseSchema(fieldType, rule);

    // Apply validation rules if provided
    if (!rule) {
        return schema.optional();
    }

    // Required
    if (rule.required) {
        const message = getMessageFromRule(rule.required, `This field is required`);
        if (schema instanceof z.ZodString) {
            schema = (schema as z.ZodString).min(1, message);
        }
    }
    else {
        schema = schema.optional();
    }

    // Apply type-specific validations
    if (schema instanceof z.ZodString) {
        schema = applyStringValidations(schema, rule);
    }
    else if (schema instanceof z.ZodNumber) {
        schema = applyNumberValidations(schema, rule);
    }

    return schema;
}

/**
 * Extract message from a validation rule value
 */
function getMessageFromRule(
    ruleValue: boolean | { message: string } | undefined,
    defaultMessage: string
): string {
    if (typeof ruleValue === `object` && ruleValue.message) {
        return ruleValue.message;
    }
    return defaultMessage;
}

/**
 * Extract value and message from a validation rule
 */
function getValueAndMessage(
    ruleValue: number | ValueWithMessage,
    defaultMessagePrefix: string
): ValueWithMessage {
    if (typeof ruleValue === `number`) {
        return {
            value:   ruleValue,
            message: `${ defaultMessagePrefix } ${ ruleValue }`,
        };
    }
    return ruleValue;
}

/**
 * Build a Zod schema from a form schema's fields
 */
export function buildFormSchema(fields: Array<FieldSchema>): z.ZodObject<Record<string, z.ZodTypeAny>> {
    const shape: Record<string, z.ZodTypeAny> = {};

    for (const field of fields) {
        shape[field.name] = validationRuleToZod(field.validation, field.fieldType);
    }

    return z.object(shape);
}

/**
 * Get nested value from object using dot notation
 */
function getNestedValue(obj: Record<string, unknown>, path: string): unknown {
    return path.split(`.`).reduce((acc, part) => {
        if (acc && typeof acc === `object` && part in acc) {
            return (acc as Record<string, unknown>)[part];
        }
        return undefined;
    }, obj as unknown);
}

/**
 * Get initial form values from field schemas
 */
export function getInitialFormValues(
    fields: Array<FieldSchema>,
    state?: Record<string, unknown>
): Record<string, unknown> {
    const values: Record<string, unknown> = {};

    for (const field of fields) {
        // Get value from state binding if available
        if (field.bindTo && state) {
            const stateValue = getNestedValue(state, field.bindTo);
            if (stateValue !== undefined) {
                values[field.name] = stateValue;
                continue;
            }
        }

        // Otherwise use default value or type-appropriate defaults
        values[field.name] = getDefaultFieldValue(field);
    }

    return values;
}

/**
 * Get default value for a field based on its type
 */
function getDefaultFieldValue(field: FieldSchema): unknown {
    if (field.defaultValue !== undefined) {
        return field.defaultValue;
    }

    switch (field.fieldType) {
        case `number`:
            return 0;
        case `checkbox`:
        case `switch`:
            return false;
        default:
            return ``;
    }
}

/**
 * Format validation errors for display
 */
export function formatValidationErrors(
    errors: Record<string, Array<string>> | undefined
): Map<string, string> {
    const formatted = new Map<string, string>();

    if (!errors) {
        return formatted;
    }

    for (const [
        field,
        messages,
    ] of Object.entries(errors)) {
        if (messages.length > 0) {
            formatted.set(field, messages[0]);
        }
    }

    return formatted;
}

/**
 * Check if a form has any validation errors
 */
export function hasValidationErrors(
    errors: Record<string, Array<string>> | undefined
): boolean {
    if (!errors) {
        return false;
    }

    return Object.values(errors).some((messages) => messages.length > 0);
}

/**
 * Field array utilities
 */
export interface FieldArrayHelpers<T> {
    fields:  Array<FieldArrayItem<T>>
    append:  (value: T) => void
    prepend: (value: T) => void
    remove:  (index: number) => void
    move:    (from: number, to: number) => void
    insert:  (index: number, value: T) => void
    update:  (index: number, value: T) => void
    replace: (values: Array<T>) => void
    swap:    (indexA: number, indexB: number) => void
}

/**
 * Create a unique field ID for field arrays
 */
let FieldIdCounter = 0;
export function createFieldId(): string {
    FieldIdCounter += 1;
    return `field_${ FieldIdCounter }`;
}

/**
 * Reset field ID counter (useful for testing)
 */
export function resetFieldIdCounter(): void {
    FieldIdCounter = 0;
}
