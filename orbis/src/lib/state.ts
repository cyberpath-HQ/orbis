/**
 * Page-level state management using zustand with immer
 */

import { create } from 'zustand';
import { immer } from 'zustand/middleware/immer';
import type { StateDefinition } from '../types/schema';
import {
    getCachedExpression, setCachedExpression
} from './performance';

/**
 * Get nested value from object using dot notation path
 */
export function getNestedValue(obj: Record<string, unknown>, path: string): unknown {
    return path.split(`.`).reduce((acc, part) => {
        if (acc && typeof acc === `object` && part in acc) {
            return (acc as Record<string, unknown>)[part];
        }
        return undefined;
    }, obj as unknown);
}

/**
 * Set nested value in object using dot notation path (immutable)
 */
export function setNestedValue<T extends Record<string, unknown>>(
    obj: T,
    path: string,
    value: unknown
): T {
    const parts = path.split(`.`);
    const clone = structuredClone(obj);

    let current: Record<string, unknown> = clone;
    for (let i = 0; i < parts.length - 1; i++) {
        const part = parts[i];
        if (!(part in current) || typeof current[part] !== `object`) {
            current[part] = {};
        }
        current = current[part] as Record<string, unknown>;
    }

    current[parts[parts.length - 1]] = value;
    return clone;
}

/**
 * Merge values into nested object
 */
export function mergeNestedValue<T extends Record<string, unknown>>(
    obj: T,
    path: string,
    value: Record<string, unknown>
): T {
    const existing = getNestedValue(obj, path);
    const merged = typeof existing === `object` && existing !== null
        ? {
            ...existing as Record<string, unknown>,
            ...value,
        }
        : value;
    return setNestedValue(obj, path, merged);
}

/**
 * Initialize state from state definition
 */
export function initializeState(definition: StateDefinition): Record<string, unknown> {
    const state: Record<string, unknown> = {};

    for (const [
        key,
        field,
    ] of Object.entries(definition)) {
        if (field.default !== undefined) {
            state[key] = structuredClone(field.default);
        }
        else {
            switch (field.type) {
                case `string`:
                    state[key] = ``;
                    break;
                case `number`:
                    state[key] = 0;
                    break;
                case `boolean`:
                    state[key] = false;
                    break;
                case `array`:
                    state[key] = [];
                    break;
                case `object`:
                    state[key] = {};
                    break;
            }
        }
    }

    return state;
}

/**
 * Save state to localStorage
 */
export function saveStateToStorage(key: string, state: Record<string, unknown>): void {
    try {
        localStorage.setItem(`orbis_state_${ key }`, JSON.stringify(state));
    }
    catch (error) {
        console.warn(`Failed to save state to localStorage:`, error);
    }
}

/**
 * Load state from localStorage
 */
export function loadStateFromStorage(key: string): Record<string, unknown> | null {
    try {
        const stored = localStorage.getItem(`orbis_state_${ key }`);
        if (stored) {
            return JSON.parse(stored) as Record<string, unknown>;
        }
    }
    catch (error) {
        console.warn(`Failed to load state from localStorage:`, error);
    }
    return null;
}

/**
 * Clear state from localStorage
 */
export function clearStateFromStorage(key: string): void {
    try {
        localStorage.removeItem(`orbis_state_${ key }`);
    }
    catch (error) {
        console.warn(`Failed to clear state from localStorage:`, error);
    }
}

/**
 * Page state store interface
 */
export interface PageStateStore {
    state:   Record<string, unknown>
    loading: Record<string, boolean>
    errors:  Record<string, string>

    // Actions
    setState:    (path: string, value: unknown) => void
    mergeState:  (path: string, value: Record<string, unknown>) => void
    resetState:  (definition: StateDefinition) => void
    setLoading:  (key: string, loading: boolean) => void
    setError:    (key: string, error: string | null) => void
    clearErrors: () => void

    // Note: getState returns ONLY the state data, not the full store
    getState: () => Record<string, unknown>
    getValue: (path: string) => unknown
}

/**
 * Page state store hook type (returned by createPageStateStore)
 * This is a Zustand hook that can be called to get state or use selectors
 */
export interface PageStateStoreHook {
    (): PageStateStore
    <T>(selector: (state: PageStateStore) => T): T

    // Zustand's getState returns the full store
    getState:    () => PageStateStore

    // Store action methods (same as PageStateStore)
    setState:    (path: string, value: unknown) => void
    mergeState:  (path: string, value: Record<string, unknown>) => void
    resetState:  (definition: StateDefinition) => void
    setLoading:  (key: string, loading: boolean) => void
    setError:    (key: string, error: string | null) => void
    clearErrors: () => void
    getValue:    (path: string) => unknown
}

/**
 * Create a page state store
 * @param initialDefinition - State definition with default values
 * @param persistenceKey - Optional key for localStorage persistence (e.g., "plugin_name:page_route")
 */
export function createPageStateStore(initialDefinition?: StateDefinition, persistenceKey?: string): PageStateStoreHook {
    // Try to load persisted state if key provided
    const persistedState = persistenceKey ? loadStateFromStorage(persistenceKey) : null;
    
    // Initialize state - merge persisted state with defaults
    const initialState = initialDefinition ? initializeState(initialDefinition) : {};
    const mergedInitialState = persistedState 
        ? { ...initialState, ...persistedState }
        : initialState;

    const store = create<PageStateStore>()(
        immer((set, get) => ({
            state:   mergedInitialState,
            loading: {},
            errors:  {},

            setState: (path, value) => set((draft) => {
                const parts = path.split(`.`);
                let current: Record<string, unknown> = draft.state;

                for (let i = 0; i < parts.length - 1; i++) {
                    const part = parts[i];
                    if (!(part in current) || typeof current[part] !== `object`) {
                        current[part] = {};
                    }
                    current = current[part] as Record<string, unknown>;
                }

                current[parts[parts.length - 1]] = value;
                
                // Persist state if key provided
                if (persistenceKey) {
                    saveStateToStorage(persistenceKey, draft.state);
                }
            }),

            mergeState: (path, value) => set((draft) => {
                const existing = getNestedValue(draft.state, path);
                const merged = typeof existing === `object` && existing !== null
                    ? {
                        ...existing as Record<string, unknown>,
                        ...value,
                    }
                    : value;

                const parts = path.split(`.`);
                let current: Record<string, unknown> = draft.state;

                for (let i = 0; i < parts.length - 1; i++) {
                    const part = parts[i];
                    if (!(part in current) || typeof current[part] !== `object`) {
                        current[part] = {};
                    }
                    current = current[part] as Record<string, unknown>;
                }

                current[parts[parts.length - 1]] = merged;
                
                // Persist state if key provided
                if (persistenceKey) {
                    saveStateToStorage(persistenceKey, draft.state);
                }
            }),

            resetState: (definition) => set((draft) => {
                draft.state = initializeState(definition);
                draft.errors = {};
                
                // Clear persisted state when resetting
                if (persistenceKey) {
                    clearStateFromStorage(persistenceKey);
                }
            }),

            setLoading: (key, loading) => set((draft) => {
                draft.loading[key] = loading;
            }),

            setError: (key, error) => set((draft) => {
                if (error === null) {
                    delete draft.errors[key];
                }
                else {
                    draft.errors[key] = error;
                }
            }),

            clearErrors: () => set((draft) => {
                draft.errors = {};
            }),

            getState: () => get().state,

            getValue: (path) => getNestedValue(get().state, path),
        }))
    );

    // Expose store methods directly on the hook for easier access
    // This allows calling stateStore.setState() directly
    const hook = store as PageStateStoreHook;
    hook.setState = (path: string, value: unknown) => store.getState().setState(path, value);
    hook.mergeState = (path: string, value: Record<string, unknown>) => store.getState().mergeState(path, value);
    hook.resetState = (definition: StateDefinition) => store.getState().resetState(definition);
    hook.setLoading = (key: string, loading: boolean) => store.getState().setLoading(key, loading);
    hook.setError = (key: string, error: string | null) => store.getState().setError(key, error);
    hook.clearErrors = () => store.getState().clearErrors();
    hook.getValue = (path: string) => store.getState().getValue(path);

    return hook;
}

/**
 * Expression interpolation - replaces {{path}} with values from state
 * Uses caching for performance optimization
 */
export function interpolateExpression(
    expression: string,
    state: Record<string, unknown>,
    context?: Record<string, unknown>
): string {
    // Generate cache key based on expression and relevant state
    const cacheKey = `${ expression }::${ JSON.stringify(state) }::${ context ? JSON.stringify(context) : `` }`;

    // Check cache first
    const cached = getCachedExpression(cacheKey, {});
    if (cached !== undefined && typeof cached === `string`) {
        return cached;
    }

    const combined = {
        ...state,
        ...context,
    };

    const result = expression.replace(/\{\{([^}]+)\}\}/g, (_, path: string) => {
        const trimmedPath = path.trim();

        // Check if this looks like an arithmetic expression
        const has_arithmetic_ops = /[+\-*/%()]/.test(trimmedPath);

        if (has_arithmetic_ops) {
            // First replace any state references in the expression
            // e.g., "state.count + 1" -> "5 + 1"
            const interpolated_path = trimmedPath.replace(/[a-zA-Z_$][a-zA-Z0-9_$.]*/g, (ref) => {
                const value = getNestedValue(combined, ref);
                if (value !== undefined) {
                    if (typeof value === `number` || typeof value === `boolean`) {
                        return String(value);
                    }
                    if (typeof value === `string`) {
                        return value;
                    }
                }
                return ref;
            });

            // Now evaluate the arithmetic expression
            try {
                const math_result = evaluateMathExpressionInternal(interpolated_path);
                return String(math_result);
            }
            catch (error) {
                console.warn(`Failed to evaluate arithmetic expression "${ interpolated_path }":`, error);
                return interpolated_path;
            }
        }

        // Regular path lookup
        const value = getNestedValue(combined, trimmedPath);

        if (value !== undefined) {
            if (typeof value === `number` || typeof value === `boolean`) {
                return value;
            }
            if (typeof value === `string`) {
                return value;
            }
            return JSON.stringify(value);
        }
        return ``;
    });

    // Cache the result
    setCachedExpression(cacheKey, {}, result);

    return result;
}

/**
 * Evaluate a boolean expression with support for complex logic
 * Supports: ==, ===, !=, !==, >, >=, <, <=, &&, ||, !, parentheses
 */
export function evaluateBooleanExpression(
    expression: boolean | string,
    state: Record<string, unknown>,
    context?: Record<string, unknown>
): boolean {
    if (typeof expression === `boolean`) {
        return expression;
    }

    const combined = {
        ...state,
        ...context,
    };
    const interpolated = interpolateExpression(expression, combined);

    if (interpolated === `true`) {
        return true;
    }
    if (interpolated === `false`) {
        return false;
    }

    // Try to evaluate the expression using the expression parser
    try {
        return evaluateComplexExpression(interpolated);
    }
    catch {
        // Fallback to simple comparison for backward compatibility
        const comparisonMatch = /^(.+?)\s*(===?|!==?|>=?|<=?)\s*(.+)$/.exec(interpolated);
        if (comparisonMatch) {
            const [
                , left,
                op,
                right,
            ] = comparisonMatch;
            const leftVal = parseValue(left.trim());
            const rightVal = parseValue(right.trim());

            switch (op) {
                case `==`:
                case `===`:
                    return leftVal === rightVal;
                case `!=`:
                case `!==`:
                    return leftVal !== rightVal;
                case `>`:
                    return Number(leftVal) > Number(rightVal);
                case `>=`:
                    return Number(leftVal) >= Number(rightVal);
                case `<`:
                    return Number(leftVal) < Number(rightVal);
                case `<=`:
                    return Number(leftVal) <= Number(rightVal);
            }
        }

        // Check for truthy value
        return Boolean(interpolated);
    }
}

/**
 * Evaluate complex boolean expressions with AND, OR, NOT and parentheses
 */
function evaluateComplexExpression(expr: string): boolean {
    expr = expr.trim();

    // Handle parentheses first
    let openParen = expr.lastIndexOf(`(`);
    while (openParen !== -1) {
        const closeParen = expr.indexOf(`)`, openParen);
        if (closeParen === -1) {
            throw new Error(`Unmatched parenthesis`);
        }
        const inner = expr.slice(openParen + 1, closeParen);
        const result = evaluateComplexExpression(inner);
        expr = expr.slice(0, openParen) + String(result) + expr.slice(closeParen + 1);
        openParen = expr.lastIndexOf(`(`);
    }

    // Handle OR (lowest precedence)
    if (expr.includes(`||`)) {
        const parts = splitByOperator(expr, `||`);
        return parts.some((part) => evaluateComplexExpression(part.trim()));
    }

    // Handle AND (higher precedence than OR)
    if (expr.includes(`&&`)) {
        const parts = splitByOperator(expr, `&&`);
        return parts.every((part) => evaluateComplexExpression(part.trim()));
    }

    // Handle NOT (highest precedence)
    if (expr.startsWith(`!`) && !expr.startsWith(`!=`)) {
        return !evaluateComplexExpression(expr.slice(1).trim());
    }

    // Handle comparison operators
    const comparisonOps = [
        `===`,
        `!==`,
        `==`,
        `!=`,
        `>=`,
        `<=`,
        `>`,
        `<`,
    ];
    for (const op of comparisonOps) {
        const opIndex = expr.indexOf(op);
        if (opIndex !== -1) {
            const left = parseValue(expr.slice(0, opIndex).trim());
            const right = parseValue(expr.slice(opIndex + op.length).trim());

            switch (op) {
                case `===`:
                case `==`:
                    return left === right;
                case `!==`:
                case `!=`:
                    return left !== right;
                case `>`:
                    return Number(left) > Number(right);
                case `>=`:
                    return Number(left) >= Number(right);
                case `<`:
                    return Number(left) < Number(right);
                case `<=`:
                    return Number(left) <= Number(right);
            }
        }
    }

    // Handle simple boolean values
    if (expr === `true`) {
        return true;
    }
    if (expr === `false`) {
        return false;
    }

    // Truthy evaluation for other values
    const value = parseValue(expr);
    return Boolean(value);
}

/**
 * Split expression by operator, respecting parentheses
 */
function splitByOperator(expr: string, op: string): Array<string> {
    const parts: Array<string> = [];
    let depth = 0;
    let current = ``;

    for (let i = 0; i < expr.length; i++) {
        const char = expr[i];
        if (char === `(`) {
            depth++;
            current += char;
        }
        else if (char === `)`) {
            depth--;
            current += char;
        }
        else if (depth === 0 && expr.slice(i, i + op.length) === op) {
            parts.push(current);
            current = ``;
            i += op.length - 1;
        }
        else {
            current += char;
        }
    }
    parts.push(current);
    return parts;
}

/**
 * Evaluate a mathematical expression
 * Supports: +, -, *, /, %, parentheses
 */
export function evaluateMathExpression(
    expression: string,
    state: Record<string, unknown>,
    context?: Record<string, unknown>
): number {
    const combined = {
        ...state,
        ...context,
    };
    const interpolated = interpolateExpression(expression, combined);

    try {
        return evaluateMathExpressionInternal(interpolated);
    }
    catch {
        return 0;
    }
}

function evaluateMathExpressionInternal(expr: string): number {
    expr = expr.trim();

    // Handle parentheses
    let openParen = expr.lastIndexOf(`(`);
    while (openParen !== -1) {
        const closeParen = expr.indexOf(`)`, openParen);
        if (closeParen === -1) {
            throw new Error(`Unmatched parenthesis`);
        }
        const inner = expr.slice(openParen + 1, closeParen);
        const result = evaluateMathExpressionInternal(inner);
        expr = expr.slice(0, openParen) + String(result) + expr.slice(closeParen + 1);
        openParen = expr.lastIndexOf(`(`);
    }

    // Handle addition and subtraction (lowest precedence)
    const addMatch = /^(.+?)\s*([+-])\s*([^+-]+)$/.exec(expr);
    if (addMatch) {
        const [
            , left,
            op,
            right,
        ] = addMatch;
        const leftVal = evaluateMathExpressionInternal(left);
        const rightVal = evaluateMathExpressionInternal(right);
        return op === `+` ? leftVal + rightVal : leftVal - rightVal;
    }

    // Handle multiplication, division, modulo (higher precedence)
    const mulMatch = /^(.+?)\s*([*/%])\s*([^*/%]+)$/.exec(expr);
    if (mulMatch) {
        const [
            , left,
            op,
            right,
        ] = mulMatch;
        const leftVal = evaluateMathExpressionInternal(left);
        const rightVal = evaluateMathExpressionInternal(right);
        switch (op) {
            case `*`:
                return leftVal * rightVal;
            case `/`:
                return rightVal !== 0 ? leftVal / rightVal : 0;
            case `%`:
                return rightVal !== 0 ? leftVal % rightVal : 0;
        }
    }

    // Parse as number
    const num = Number(expr);
    if (!isNaN(num)) {
        return num;
    }

    return 0;
}

/**
 * String manipulation functions for expressions
 */
export const stringFunctions = {
    uppercase:   (str: string) => str.toUpperCase(),
    lowercase:   (str: string) => str.toLowerCase(),
    capitalize:  (str: string) => str.charAt(0).toUpperCase() + str.slice(1).toLowerCase(),
    trim:        (str: string) => str.trim(),
    length:      (str: string) => str.length,
    includes:    (str: string, search: string) => str.includes(search),
    startsWith:  (str: string, search: string) => str.startsWith(search),
    endsWith:    (str: string, search: string) => str.endsWith(search),
    replace:     (str: string, search: string, replacement: string) => str.replace(search, replacement),
    replaceAll:  (str: string, search: string, replacement: string) => str.split(search).join(replacement),
    substring:   (str: string, start: number, end?: number) => str.substring(start, end),
    split:       (str: string, separator: string) => str.split(separator),
    join:        (arr: Array<string>, separator: string) => arr.join(separator),
    concat:      (...args: Array<string>) => args.join(``),
    padStart:    (str: string, length: number, fillChar = ` `) => str.padStart(length, fillChar),
    padEnd:      (str: string, length: number, fillChar = ` `) => str.padEnd(length, fillChar),
    repeat:      (str: string, count: number) => str.repeat(count),
    reverse:     (str: string) => str.split(``).reverse()
        .join(``),
    truncate:    (str: string, length: number, suffix = `...`) => str.length > length ? str.slice(0, length - suffix.length) + suffix : str,
};

/**
 * Array manipulation functions for expressions
 */
export const arrayFunctions = {
    length:   (arr: Array<unknown>) => arr.length,
    first:    (arr: Array<unknown>) => arr[0],
    last:     (arr: Array<unknown>) => arr[arr.length - 1],
    isEmpty:  (arr: Array<unknown>) => arr.length === 0,
    includes: (arr: Array<unknown>, item: unknown) => arr.includes(item),
    indexOf:  (arr: Array<unknown>, item: unknown) => arr.indexOf(item),
    slice:    (arr: Array<unknown>, start: number, end?: number) => arr.slice(start, end),
    reverse:  (arr: Array<unknown>) => [ ...arr ].reverse(),
    unique:   (arr: Array<unknown>) => [ ...new Set(arr) ],
    flatten:  (arr: Array<unknown>) => arr.flat(),
    count:    (arr: Array<unknown>, predicate?: (item: unknown) => boolean) => predicate ? arr.filter(predicate).length : arr.length,
};

/**
 * Math utility functions for expressions
 */
export const mathFunctions = {
    abs:     (n: number) => Math.abs(n),
    ceil:    (n: number) => Math.ceil(n),
    floor:   (n: number) => Math.floor(n),
    round:   (n: number, decimals = 0) => {
        const factor = 10 ** decimals;
        return Math.round(n * factor) / factor;
    },
    min:     (...args: Array<number>) => Math.min(...args),
    max:     (...args: Array<number>) => Math.max(...args),
    sum:     (...args: Array<number>) => args.reduce((a, b) => a + b, 0),
    average: (...args: Array<number>) => args.reduce((a, b) => a + b, 0) / args.length,
    clamp:   (n: number, min: number, max: number) => Math.min(Math.max(n, min), max),
    random:  (min = 0, max = 1) => Math.random() * (max - min) + min,
    pow:     (base: number, exponent: number) => base ** exponent,
    sqrt:    (n: number) => Math.sqrt(n),
};

function parseValue(str: string): unknown {
    if (str === `true`) {
        return true;
    }
    if (str === `false`) {
        return false;
    }
    if (str === `null`) {
        return null;
    }
    if (str === `undefined`) {
        return undefined;
    }

    const num = Number(str);
    if (!isNaN(num)) {
        return num;
    }

    // Remove quotes if present
    if ((str.startsWith(`"`) && str.endsWith(`"`)) ||
        (str.startsWith(`'`) && str.endsWith(`'`)) ||
        (str.startsWith("`") && str.endsWith("`"))) {
        return str.slice(1, -1);
    }

    return str;
}
