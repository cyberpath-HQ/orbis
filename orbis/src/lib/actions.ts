/**
 * Action executor for the JSON UI Schema event system
 */

import { toast } from 'sonner';
import type { NavigateFunction } from 'react-router-dom';
import type {
    Action,
    UpdateStateAction,
    CallApiAction,
    NavigateAction,
    ShowToastAction,
    DebouncedAction,
    ConditionalAction,
    SequenceAction
} from '../types/schema';
import {
    type PageStateStoreHook,
    getNestedValue,
    interpolateExpression,
    evaluateBooleanExpression,
    evaluateMathExpression
} from './state';

// Debounce timers storage
const DEBOUNCE_TIMERS = new Map<string, ReturnType<typeof setTimeout>>();

// Action execution context
export interface ActionContext {
    state:     PageStateStoreHook
    navigate:  NavigateFunction
    apiClient: ApiClient
    event?:    unknown
    row?:      Record<string, unknown>
    item?:     unknown
    index?:    number
    response?: unknown
    error?:    unknown
}

// API client interface
export interface ApiClient {
    call: (api: string, method: string, args?: Record<string, unknown>) => Promise<unknown>
}

/**
 * Resolve a value that might reference special variables
 */
function resolveValue(
    value: unknown,
    context: ActionContext
): unknown {
    if (typeof value !== `string`) {
        return value;
    }

    // Check for special variables
    if (value === `$event`) {
        return context.event;
    }
    if (value === `$event.value`) {
        const event = context.event as { target?: { value?: unknown }
            value?:                               unknown } | undefined;
        return event?.value ?? (event?.target as { value?: unknown })?.value;
    }
    if (value === `$row`) {
        return context.row;
    }
    if (value === `$item`) {
        return context.item;
    }
    if (value === `$index`) {
        return context.index;
    }
    if (value === `$response`) {
        return context.response;
    }
    if (value === `$response.data`) {
        return (context.response as { data?: unknown })?.data;
    }
    if (value === `$error`) {
        return context.error;
    }

    // Check for path references starting with special variables
    if (value.startsWith(`$row.`)) {
        return getNestedValue(context.row ?? {}, value.slice(5));
    }
    if (value.startsWith(`$item.`)) {
        return getNestedValue(
            typeof context.item === `object` && context.item !== null
                ? context.item as Record<string, unknown>
                : {},
            value.slice(6)
        );
    }
    if (value.startsWith(`$response.`)) {
        return getNestedValue(
            typeof context.response === `object` && context.response !== null
                ? context.response as Record<string, unknown>
                : {},
            value.slice(10)
        );
    }

    // Check if this is a template expression (contains {{...}})
    const stateData = context.state.getState().state;
    const contextData = {
        state:     stateData,
        $event:    context.event,
        $row:      context.row,
        $item:     context.item,
        $index:    context.index,
        $response: context.response,
        $error:    context.error,
    };

    // If the value contains template expressions, process them
    if (value.includes(`{{`)) {
        // Check if this might be an arithmetic expression by looking for operators
        // within or after the template expression
        const has_arithmetic_ops = /[+\-*/%]/.test(value);

        if (has_arithmetic_ops) {
            // Try to evaluate as a math expression
            // evaluateMathExpression will interpolate {{...}} first, then evaluate
            try {
                const mathResult = evaluateMathExpression(value, stateData, contextData);
                return mathResult;
            }
            catch (error) {
                console.warn(
                    `[resolveValue] Failed to evaluate as math, falling back to interpolation:`,
                    error
                );

                // Fall through to regular interpolation
            }
        }

        // Regular string interpolation
        const interpolated = interpolateExpression(value, stateData, contextData);
        return interpolated;
    }

    // No template expressions, return as-is
    return value;
}

/**
 * Execute a single action
 */
export async function executeAction(
    action: Action,
    context: ActionContext
): Promise<void> {
    switch (action.type) {
        case `update_state`:
            executeUpdateState(action, context);
            break;

        case `call_api`:
            await executeCallApi(action, context);
            break;

        case `navigate`:
            executeNavigate(action, context);
            break;

        case `show_toast`:
            executeShowToast(action, context);
            break;

        case `debounced_action`:
            executeDebouncedAction(action, context);
            break;

        case `conditional`:
            await executeConditional(action, context);
            break;

        case `sequence`:
            await executeSequence(action, context);
            break;

        case `set_loading`:
            context.state.setLoading(action.target ?? `global`, action.loading);
            break;

        case `show_dialog`:
            // Dialog handling will be done through state
            context.state.setState(`__dialogs.${ action.dialogId }`, {
                open: true,
                data: action.data ? resolveObjectValues(action.data, context) : {},
            });
            break;

        case `close_dialog`:
            context.state.setState(`__dialogs.${ action.dialogId ?? `current` }`, {
                open: false,
            });
            break;

        case `copy`:
            await executeCopy(action, context);
            break;

        case `open_url`:
            executeOpenUrl(action, context);
            break;

        case `download`:
            await executeDownload(action, context);
            break;

        case `validate_form`:
            await executeValidateForm(action, context);
            break;

        case `reset_form`:
            executeResetForm(action, context);
            break;

        case `emit`:
            executeEmit(action, context);
            break;
    }
}

/**
 * Execute multiple actions in sequence
 */
export async function executeActions(
    actions: Array<Action>,
    context: ActionContext
): Promise<void> {
    for (const action of actions) {
        await executeAction(action, context);
    }
}

// Individual action executors

function executeUpdateState(action: UpdateStateAction, context: ActionContext): void {
    const {
        from, value: actionValue, merge, path, mode,
    } = action;
    let value: unknown;

    if (from !== undefined) {
        value = resolveValue(from, context);
    }
    else {
        value = actionValue;
    }

    const should_merge = merge === true || mode === `merge`;
    if (should_merge && typeof value === `object` && value !== null) {
        context.state.mergeState(path, value as Record<string, unknown>);
    }
    else if (mode === `append`) {
        const existing = context.state.getValue(path);
        if (Array.isArray(existing)) {
            context.state.setState(path, [
                ...existing,
                value,
            ]);
        }
        else {
            context.state.setState(path, [ value ]);
        }
    }
    else if (mode === `prepend`) {
        const existing = context.state.getValue(path);
        if (Array.isArray(existing)) {
            context.state.setState(path, [
                value,
                ...existing,
            ]);
        }
        else {
            context.state.setState(path, [ value ]);
        }
    }
    else if (mode === `remove`) {
        const existing = context.state.getValue(path);
        if (Array.isArray(existing)) {
            context.state.setState(
                path,
                existing.filter((item) => item !== value)
            );
        }
    }
    else {
        // Default to set
        context.state.setState(path, value);
    }
}

async function executeCallApi(action: CallApiAction, context: ActionContext): Promise<void> {
    const method = action.method ?? `GET`;

    // Build arguments
    const args: Record<string, unknown> = {};

    if (action.args_from_state) {
        for (const statePath of action.args_from_state) {
            const value = context.state.getValue(statePath);
            args[statePath] = value;
        }
    }

    if (action.map_args) {
        for (const mapping of action.map_args) {
            const value = resolveValue(mapping.from, context);
            args[mapping.to] = value;
        }
    }

    if (action.body) {
        if (typeof action.body === `string`) {
            Object.assign(args, JSON.parse(resolveValue(action.body, context) as string));
        }
        else {
            Object.assign(args, resolveObjectValues(action.body as Record<string, unknown>, context));
        }
    }

    try {
        const response = await context.apiClient.call(action.api, method, args);

        if (action.on_success) {
            await executeActions(action.on_success, {
                ...context,
                response,
            });
        }
    }
    catch (error) {
        if (action.on_error) {
            await executeActions(action.on_error, {
                ...context,
                error,
            });
        }
    }
    finally {
        if (action.on_finally) {
            await executeActions(action.on_finally, context);
        }
    }
}

function executeNavigate(action: NavigateAction, context: ActionContext): void {
    let to = resolveValue(action.to, context) as string;

    // Handle params in the route
    if (action.params) {
        for (const [
            key,
            value,
        ] of Object.entries(action.params)) {
            const resolved = resolveValue(value, context);
            to = to.replace(`$${ key }`, String(resolved));
        }
    }

    if (action.replace) {
        context.navigate(to, {
            replace: true,
        });
    }
    else {
        context.navigate(to);
    }
}

function executeShowToast(action: ShowToastAction, context: ActionContext): void {
    const message = resolveValue(action.message, context) as string;
    const title = action.title ? resolveValue(action.title, context) as string : undefined;

    const options: { description?: string
        duration?:                 number } = {};
    if (title) {
        options.description = message;
    }
    if (action.duration) {
        options.duration = action.duration;
    }

    switch (action.level) {
        case `success`:
            toast.success(title ?? message, options);
            break;
        case `error`:
            toast.error(title ?? message, options);
            break;
        case `warning`:
            toast.warning(title ?? message, options);
            break;
        case `info`:
        default:
            toast.info(title ?? message, options);
            break;
    }
}

function executeDebouncedAction(action: DebouncedAction, context: ActionContext): void {
    const key = action.key ?? `default`;

    // Clear existing timer
    const existingTimer = DEBOUNCE_TIMERS.get(key);
    if (existingTimer) {
        clearTimeout(existingTimer);
    }

    // Set new timer
    const timer = setTimeout(() => {
        DEBOUNCE_TIMERS.delete(key);
        void executeAction(action.action, context);
    }, action.delayMs);

    DEBOUNCE_TIMERS.set(key, timer);
}

async function executeConditional(action: ConditionalAction, context: ActionContext): Promise<void> {
    const stateData = context.state.getState().state;
    const is_condition_met = evaluateBooleanExpression(action.condition, stateData);

    if (is_condition_met) {
        await executeActions(action.then, context);
    }
    else if (action.else) {
        await executeActions(action.else, context);
    }
}

async function executeSequence(action: SequenceAction, context: ActionContext): Promise<void> {
    for (const subAction of action.actions) {
        try {
            await executeAction(subAction, context);
        }
        catch (error) {
            if (action.stopOnError) {
                throw error;
            }
        }
    }
}

async function executeCopy(action: { text: string
    showNotification?:                     boolean }, context: ActionContext): Promise<void> {
    const text = resolveValue(action.text, context) as string;
    await navigator.clipboard.writeText(text);

    if (action.showNotification !== false) {
        toast.success(`Copied to clipboard`);
    }
}

function executeOpenUrl(action: { url: string
    newTab?:                           boolean }, context: ActionContext): void {
    const url = resolveValue(action.url, context) as string;

    if (action.newTab !== false) {
        window.open(url, `_blank`, `noopener,noreferrer`);
    }
    else {
        window.location.href = url;
    }
}

// Helper to resolve all values in an object
function resolveObjectValues(
    obj: Record<string, unknown>,
    context: ActionContext
): Record<string, unknown> {
    const result: Record<string, unknown> = {};

    for (const [
        key,
        value,
    ] of Object.entries(obj)) {
        if (typeof value === `string`) {
            result[key] = resolveValue(value, context);
        }
        else if (typeof value === `object` && value !== null) {
            result[key] = resolveObjectValues(value as Record<string, unknown>, context);
        }
        else {
            result[key] = value;
        }
    }

    return result;
}

/**
 * Download a file from URL or blob
 */
async function executeDownload(action: { url: string
    filename?:                                string }, context: ActionContext): Promise<void> {
    const url = resolveValue(action.url, context) as string;
    const filename = action.filename
        ? resolveValue(action.filename, context) as string
        : url.split(`/`).pop() ?? `download`;

    try {
        // Check if it's a data URL or blob
        if (url.startsWith(`data:`) || url.startsWith(`blob:`)) {
            const link = document.createElement(`a`);
            link.href = url;
            link.download = filename;
            document.body.appendChild(link);
            link.click();
            document.body.removeChild(link);
        }
        else {
            // Fetch the file and download
            const response = await fetch(url);
            const blob = await response.blob();
            const blobUrl = URL.createObjectURL(blob);

            const link = document.createElement(`a`);
            link.href = blobUrl;
            link.download = filename;
            document.body.appendChild(link);
            link.click();
            document.body.removeChild(link);

            URL.revokeObjectURL(blobUrl);
        }

        toast.success(`Download started: ${ filename }`);
    }
    catch (error) {
        toast.error(`Download failed: ${ error instanceof Error ? error.message : String(error) }`);
    }
}

/**
 * Form state tracking for validation
 */
const FORM_STATES = new Map<string, {
    fields:     Map<string, { value: unknown
        errors:                      Array<string>
        touched:                     boolean }>
    isValid:     boolean
    isSubmitted: boolean
}>();

/**
 * Register a form field for tracking
 */
export function registerFormField(
    formId: string,
    fieldName: string,
    value: unknown
): void {
    if (!FORM_STATES.has(formId)) {
        FORM_STATES.set(formId, {
            fields:      new Map(),
            isValid:     true,
            isSubmitted: false,
        });
    }

    const formState = FORM_STATES.get(formId)!;
    formState.fields.set(fieldName, {
        value,
        errors:  [],
        touched: false,
    });
}

/**
 * Update a form field value
 */
export function updateFormField(
    formId: string,
    fieldName: string,
    value: unknown
): void {
    const formState = FORM_STATES.get(formId);
    if (formState) {
        const field = formState.fields.get(fieldName);
        if (field) {
            field.value = value;
            field.touched = true;
        }
    }
}

/**
 * Set field errors
 */
export function setFieldErrors(
    formId: string,
    fieldName: string,
    errors: Array<string>
): void {
    const formState = FORM_STATES.get(formId);
    if (formState) {
        const field = formState.fields.get(fieldName);
        if (field) {
            field.errors = errors;
        }
    }
}

/**
 * Get form field errors
 */
export function getFieldErrors(formId: string, fieldName: string): Array<string> {
    const formState = FORM_STATES.get(formId);
    if (formState) {
        const field = formState.fields.get(fieldName);
        if (field) {
            return field.errors;
        }
    }
    return [];
}

/**
 * Validate form using schema validation rules from state
 */
async function executeValidateForm(
    action: {
        formId:     string
        onValid?:   Array<Action>
        onInvalid?: Array<Action>
    },
    context: ActionContext
): Promise<void> {
    const {
        formId,
    } = action;
    const formState = FORM_STATES.get(formId);

    if (!formState) {
        // No form state, try to validate from DOM
        const form = document.getElementById(formId) as HTMLFormElement | null;
        if (form) {
            const isValid = form.checkValidity();

            if (isValid) {
                if (action.onValid) {
                    await executeActions(action.onValid, context);
                }
            }
            else {
                form.reportValidity();
                if (action.onInvalid) {
                    await executeActions(action.onInvalid, context);
                }
            }
            return;
        }

        console.warn(`Form not found: ${ formId }`);
        return;
    }

    // Validate all fields
    let isValid = true;
    const stateData = context.state.getState().state;

    for (const [
        fieldName,
        field,
    ] of formState.fields) {
        const errors: Array<string> = [];

        // Get validation rules from state if defined
        const validationPath = `__validation.${ formId }.${ fieldName }`;
        const rules = stateData[validationPath] as {
            required?:  boolean
            minLength?: number
            maxLength?: number
            pattern?:   string
            min?:       number
            max?:       number
            custom?:    string // Expression to evaluate
        } | undefined;

        if (rules) {
            const {
                value,
            } = field;

            if (rules.required && (value === null || value === undefined || value === ``)) {
                errors.push(`This field is required`);
            }

            if (rules.minLength && typeof value === `string` && value.length < rules.minLength) {
                errors.push(`Minimum length is ${ rules.minLength }`);
            }

            if (rules.maxLength && typeof value === `string` && value.length > rules.maxLength) {
                errors.push(`Maximum length is ${ rules.maxLength }`);
            }

            if (rules.pattern && typeof value === `string`) {
                const regex = new RegExp(rules.pattern);
                if (!regex.test(value)) {
                    errors.push(`Invalid format`);
                }
            }

            if (rules.min !== undefined && typeof value === `number` && value < rules.min) {
                errors.push(`Minimum value is ${ rules.min }`);
            }

            if (rules.max !== undefined && typeof value === `number` && value > rules.max) {
                errors.push(`Maximum value is ${ rules.max }`);
            }
        }

        field.errors = errors;
        if (errors.length > 0) {
            isValid = false;
        }
    }

    formState.isValid = isValid;
    formState.isSubmitted = true;

    if (isValid) {
        if (action.onValid) {
            await executeActions(action.onValid, context);
        }
    }
    else if (action.onInvalid) {
        await executeActions(action.onInvalid, context);
    }
}

/**
 * Reset form to initial state
 */
function executeResetForm(
    action: { formId: string },
    context: ActionContext
): void {
    const {
        formId,
    } = action;

    // Clear form state tracking
    FORM_STATES.delete(formId);

    // Reset DOM form if exists
    const form = document.getElementById(formId) as HTMLFormElement | null;
    if (form) {
        form.reset();
    }

    // Clear any form-related state
    const formStatePath = `__forms.${ formId }`;
    context.state.setState(formStatePath, {});
}

/**
 * Custom event emitter for inter-component communication
 */
const EVENT_LISTENERS = new Map<string, Set<(payload: unknown) => void>>();

/**
 * Subscribe to a custom event
 */
export function subscribeToEvent(
    eventName: string,
    callback: (payload: unknown) => void
): () => void {
    if (!EVENT_LISTENERS.has(eventName)) {
        EVENT_LISTENERS.set(eventName, new Set());
    }

    EVENT_LISTENERS.get(eventName)!.add(callback);

    // Return unsubscribe function
    return () => {
        EVENT_LISTENERS.get(eventName)?.delete(callback);
    };
}

/**
 * Emit a custom event
 */
function executeEmit(
    action: {
        event:    string
        payload?: Record<string, unknown>
    },
    context: ActionContext
): void {
    const eventName = action.event;
    const payload = action.payload
        ? resolveObjectValues(action.payload, context)
        : {};

    const listeners = EVENT_LISTENERS.get(eventName);
    if (listeners) {
        for (const callback of listeners) {
            try {
                callback(payload);
            }
            catch (error) {
                console.error(`Error in event listener for "${ eventName }":`, error);
            }
        }
    }

    // Also dispatch a DOM custom event for external integration
    const customEvent = new CustomEvent(`orbis:${ eventName }`, {
        detail:     payload,
        bubbles:    true,
        cancelable: true,
    });
    document.dispatchEvent(customEvent);
}
