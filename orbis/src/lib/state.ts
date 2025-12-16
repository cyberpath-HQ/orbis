/**
 * Page-level state management using zustand with immer
 */

import { create } from 'zustand';
import { immer } from 'zustand/middleware/immer';
import type { StateDefinition } from '../types/schema';

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
    getState:    () => Record<string, unknown>
    getValue:    (path: string) => unknown
}

/**
 * Create a page state store
 */
export function createPageStateStore(initialDefinition?: StateDefinition) {
    return create<PageStateStore>()(
        immer((set, get) => ({
            state:   initialDefinition ? initializeState(initialDefinition) : {},
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
            }),

            resetState: (definition) => set((draft) => {
                draft.state = initializeState(definition);
                draft.errors = {};
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
}

/**
 * Expression interpolation - replaces {{path}} with values from state
 */
export function interpolateExpression(
    expression: string,
    state: Record<string, unknown>,
    context?: Record<string, unknown>
): string {
    const combined = {
        ...state,
        ...context,
    };

    return expression.replace(/\{\{([^}]+)\}\}/g, (_, path: string) => {
        const trimmedPath = path.trim();
        const value = getNestedValue(combined, trimmedPath);
        return value !== undefined ? String(value) : ``;
    });
}

/**
 * Evaluate a boolean expression
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

    // Try to evaluate simple comparisons
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
