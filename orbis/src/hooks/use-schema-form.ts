/* eslint-disable @typescript-eslint/no-explicit-any */
/**
 * Custom hook for TanStack Form integration with schema-based forms
 */

import {
    useCallback,
    useMemo
} from 'react';
import { useForm } from '@tanstack/react-form';
import type {
    FieldSchema, FormSchema
} from '@/types/schema/components';
import {
    buildFormSchema,
    getInitialFormValues
} from '@/lib/form-utils';
import type { PageStateStore } from '@/lib/state';

/**
 * Form field state from TanStack Form
 */
export interface FormFieldState<T = unknown> {
    value:        T
    error?:       string
    isTouched:    boolean
    isValidating: boolean
    isDirty:      boolean
}

/**
 * Form state from TanStack Form
 */
export interface SchemaFormState {
    isSubmitting: boolean
    isValid:      boolean
    isDirty:      boolean
    errors:       Map<string, string>
}

/**
 * Options for useSchemaForm hook
 */
export interface UseSchemaFormOptions {
    schema:             FormSchema
    pageState:          PageStateStore
    onSubmit?:          (values: Record<string, unknown>) => void | Promise<void>
    onValidationError?: (errors: Record<string, Array<string>>) => void
}

/**
 * Return type for useSchemaForm hook
 */
export interface UseSchemaFormReturn {

    form:          any
    formState:     SchemaFormState
    handleSubmit:  () => void
    resetForm:     () => void
    getFieldProps: (fieldName: string) => FieldProps
    getFieldState: (fieldName: string) => FormFieldState
    validateField: (fieldName: string) => Promise<void>
    setFieldValue: (fieldName: string, value: unknown) => void
}

/**
 * Props passed to form field components
 */
export interface FieldProps {
    name:         string
    value:        unknown
    onChange:     (value: unknown) => void
    onBlur:       () => void
    error?:       string
    isTouched:    boolean
    isValidating: boolean
}

/**
 * Custom hook for TanStack Form with Zod validation
 */
export function useSchemaForm({
    schema,
    pageState,
    onSubmit,
    onValidationError,
}: UseSchemaFormOptions): UseSchemaFormReturn {
    // Build Zod schema from form fields
    const zodSchema = useMemo(() => buildFormSchema(schema.fields), [ schema.fields ]);

    // Get initial values
    const initialValues = useMemo(
        () => getInitialFormValues(schema.fields, pageState.getState()),
        [
            schema.fields,
            pageState,
        ]
    );

    // Create form instance - using zod standard schema validation directly (no adapter needed with zod 4+)
    const form = useForm({
        defaultValues: initialValues,
        validators:    {
            onChange: zodSchema,
            onBlur:   zodSchema,
        },
        onSubmit: async({
            value,
        }) => {
            // Sync form values to page state
            syncToPageState(value, schema.fields, pageState);

            // Call onSubmit callback
            if (onSubmit) {
                await onSubmit(value);
            }
        },
        onSubmitInvalid: ({
            formApi,
        }) => {
            if (onValidationError) {
                const errors: Record<string, Array<string>> = {};
                const fieldErrors = formApi.state.fieldMeta;

                for (const [
                    fieldName,
                    meta,
                ] of Object.entries(fieldErrors)) {
                    if (meta?.errors && meta.errors.length > 0) {
                        errors[fieldName] = meta.errors.map((e) => String(e));
                    }
                }

                onValidationError(errors);
            }
        },
    });

    // Compute form state
    const formState = useMemo<SchemaFormState>(() => {
        const errors = new Map<string, string>();
        const {
            fieldMeta,
        } = form.state;

        for (const [
            fieldName,
            meta,
        ] of Object.entries(fieldMeta)) {
            if (meta?.errors && meta.errors.length > 0) {
                errors.set(fieldName, String(meta.errors[0]));
            }
        }

        return {
            isSubmitting: form.state.isSubmitting,
            isValid:      form.state.isValid,
            isDirty:      form.state.isDirty,
            errors,
        };
    }, [
        form.state.isSubmitting,
        form.state.isValid,
        form.state.isDirty,
        form.state.fieldMeta,
    ]);

    // Handle form submission
    const handleSubmit = useCallback(() => {
        void form.handleSubmit();
    }, [ form ]);

    // Reset form to initial values
    const resetForm = useCallback(() => {
        form.reset();
    }, [ form ]);

    // Get field props for a specific field
    const getFieldProps = useCallback((fieldName: string): FieldProps => {
        const fieldMeta = form.state.fieldMeta[fieldName];
        const fieldValue = form.state.values[fieldName];

        return {
            name:         fieldName,
            value:        fieldValue,
            onChange:     (value: unknown) => form.setFieldValue(fieldName, value),
            onBlur:       async() => form.validateField(fieldName, `blur`),
            error:        fieldMeta?.errors?.[0] ? String(fieldMeta.errors[0]) : undefined,
            isTouched:    fieldMeta?.isTouched ?? false,
            isValidating: fieldMeta?.isValidating ?? false,
        };
    }, [ form ]);

    // Get field state
    const getFieldState = useCallback((fieldName: string): FormFieldState => {
        const fieldMeta = form.state.fieldMeta[fieldName];
        const fieldValue = form.state.values[fieldName];

        return {
            value:        fieldValue,
            error:        fieldMeta?.errors?.[0] ? String(fieldMeta.errors[0]) : undefined,
            isTouched:    fieldMeta?.isTouched ?? false,
            isValidating: fieldMeta?.isValidating ?? false,
            isDirty:      fieldMeta?.isDirty ?? false,
        };
    }, [
        form.state.fieldMeta,
        form.state.values,
    ]);

    // Validate a specific field
    const validateField = useCallback(async(fieldName: string): Promise<void> => {
        await form.validateField(fieldName, `change`);
    }, [ form ]);

    // Set field value
    const setFieldValue = useCallback((fieldName: string, value: unknown) => {
        form.setFieldValue(fieldName, value);
    }, [ form ]);

    return {
        form,
        formState,
        handleSubmit,
        resetForm,
        getFieldProps,
        getFieldState,
        validateField,
        setFieldValue,
    };
}

/**
 * Sync form values to page state based on field bindings
 */
function syncToPageState(
    values: Record<string, unknown>,
    fields: Array<FieldSchema>,
    pageState: PageStateStore
): void {
    for (const field of fields) {
        if (field.bindTo) {
            const value = values[field.name];
            pageState.setState(field.bindTo, value);
        }
    }
}

/**
 * Hook for field arrays with TanStack Form
 */
export interface UseFieldArrayOptions<T> {

    form:        any
    fieldName:   string
    defaultItem: T
}

export interface UseFieldArrayReturn<T> {
    fields:  Array<{ id: string
        index:           number }>
    append:  (value?: T) => void
    prepend: (value?: T) => void
    remove:  (index: number) => void
    move:    (from: number, to: number) => void
}

/**
 * Hook for managing field arrays
 */
export function useFieldArray<T>({
    form,
    fieldName,
    defaultItem,
}: UseFieldArrayOptions<T>): UseFieldArrayReturn<T> {
    const formValues = form.state.values as Record<string, unknown>;
    const fieldValue = (formValues[fieldName] ?? []) as Array<T>;

    const fields = useMemo(() => fieldValue.map((_, index) => ({
        id:    `${ fieldName }_${ index }`,
        index,
    })), [
        fieldValue,
        fieldName,
    ]);

    const append = useCallback((value?: T) => {
        const newValue = [
            ...fieldValue,
            value ?? defaultItem,
        ];
        form.setFieldValue(fieldName, newValue);
    }, [
        form,
        fieldName,
        fieldValue,
        defaultItem,
    ]);

    const prepend = useCallback((value?: T) => {
        const newValue = [
            value ?? defaultItem,
            ...fieldValue,
        ];
        form.setFieldValue(fieldName, newValue);
    }, [
        form,
        fieldName,
        fieldValue,
        defaultItem,
    ]);

    const remove = useCallback((index: number) => {
        const newValue = fieldValue.filter((_, i) => i !== index);
        form.setFieldValue(fieldName, newValue);
    }, [
        form,
        fieldName,
        fieldValue,
    ]);

    const move = useCallback((from: number, to: number) => {
        const newValue = [ ...fieldValue ];
        const item = newValue.splice(from, 1)[0];
        newValue.splice(to, 0, item);
        form.setFieldValue(fieldName, newValue);
    }, [
        form,
        fieldName,
        fieldValue,
    ]);

    return {
        fields,
        append,
        prepend,
        remove,
        move,
    };
}
