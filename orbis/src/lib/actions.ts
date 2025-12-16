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
    type PageStateStore,
    getNestedValue,
    interpolateExpression,
    evaluateBooleanExpression
} from './state';

// Debounce timers storage
const DEBOUNCE_TIMERS = new Map<string, ReturnType<typeof setTimeout>>();

// Action execution context
export interface ActionContext {
    state:     PageStateStore
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

    // Interpolate expressions
    const stateData = context.state.getState();
    return interpolateExpression(value, stateData, {
        $event:    context.event,
        $row:      context.row,
        $item:     context.item,
        $index:    context.index,
        $response: context.response,
        $error:    context.error,
    });
}

/**
 * Execute a single action
 */
export async function executeAction(
    action: Action,
    context: ActionContext
): Promise<void> {
    switch (action.type) {
        case `updateState`:
            executeUpdateState(action, context);
            break;

        case `callApi`:
            await executeCallApi(action, context);
            break;

        case `navigate`:
            executeNavigate(action, context);
            break;

        case `showToast`:
            executeShowToast(action, context);
            break;

        case `debouncedAction`:
            executeDebouncedAction(action, context);
            break;

        case `conditional`:
            await executeConditional(action, context);
            break;

        case `sequence`:
            await executeSequence(action, context);
            break;

        case `setLoading`:
            context.state.setLoading(action.target ?? `global`, action.loading);
            break;

        case `showDialog`:
            // Dialog handling will be done through state
            context.state.setState(`__dialogs.${ action.dialogId }`, {
                open: true,
                data: action.data ? resolveObjectValues(action.data, context) : {},
            });
            break;

        case `closeDialog`:
            context.state.setState(`__dialogs.${ action.dialogId ?? `current` }`, {
                open: false,
            });
            break;

        case `copy`:
            await executeCopy(action, context);
            break;

        case `openUrl`:
            executeOpenUrl(action, context);
            break;

        case `download`:
        case `validateForm`:
        case `resetForm`:
        case `emit`:
            // These will be implemented as needed
            console.warn(`Action type not yet implemented: ${ action.type }`);
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
        from, value: actionValue, merge: should_merge, path,
    } = action;
    let value: unknown;

    if (from !== undefined) {
        value = resolveValue(from, context);
    }
    else {
        value = actionValue;
    }

    if (should_merge && typeof value === `object` && value !== null) {
        context.state.mergeState(path, value as Record<string, unknown>);
    }
    else {
        context.state.setState(path, value);
    }
}

async function executeCallApi(action: CallApiAction, context: ActionContext): Promise<void> {
    const method = action.method ?? `GET`;

    // Build arguments
    const args: Record<string, unknown> = {};

    if (action.argsFromState) {
        for (const statePath of action.argsFromState) {
            const value = context.state.getValue(statePath);
            args[statePath] = value;
        }
    }

    if (action.mapArgs) {
        for (const mapping of action.mapArgs) {
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

        if (action.onSuccess) {
            await executeActions(action.onSuccess, {
                ...context,
                response,
            });
        }
    }
    catch (error) {
        if (action.onError) {
            await executeActions(action.onError, {
                ...context,
                error,
            });
        }
    }
    finally {
        if (action.onFinally) {
            await executeActions(action.onFinally, context);
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
    const stateData = context.state.getState();
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
