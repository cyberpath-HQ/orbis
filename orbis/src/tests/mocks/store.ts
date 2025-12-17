/* eslint-disable */
/**
 * Mock utilities for Zustand stores in testing
 */

import { vi } from 'vitest';
import type { PageStateStore } from '@/lib/state';
import type { StateDefinition } from '@/types/schema/base';

/**
 * Create a mock page state store for testing
 */
export function createMockPageStateStore(
    initialState: Record<string, unknown> = {}
): PageStateStore {
    let state = { ...initialState };
    let loading: Record<string, boolean> = {};
    let errors: Record<string, string> = {};

    const store: PageStateStore = {
        state,
        loading,
        errors,
        setState: vi.fn((path: string, value: unknown) => {
            const parts = path.split('.');
            let current: Record<string, unknown> = state;
            for (let i = 0; i < parts.length - 1; i++) {
                const part = parts[i];
                if (!(part in current) || typeof current[part] !== 'object') {
                    current[part] = {};
                }
                current = current[part] as Record<string, unknown>;
            }
            current[parts[parts.length - 1]] = value;
        }),
        mergeState: vi.fn((path: string, value: Record<string, unknown>) => {
            const existing = store.getValue(path);
            const merged = typeof existing === 'object' && existing !== null
                ? { ...existing as Record<string, unknown>, ...value }
                : value;
            store.setState(path, merged);
        }),
        resetState: vi.fn((definition: StateDefinition) => {
            state = {};
            for (const [key, field] of Object.entries(definition)) {
                if (field.default !== undefined) {
                    state[key] = structuredClone(field.default);
                } else {
                    switch (field.type) {
                        case 'string': state[key] = ''; break;
                        case 'number': state[key] = 0; break;
                        case 'boolean': state[key] = false; break;
                        case 'array': state[key] = []; break;
                        case 'object': state[key] = {}; break;
                    }
                }
            }
            errors = {};
        }),
        setLoading: vi.fn((key: string, isLoading: boolean) => {
            loading[key] = isLoading;
        }),
        setError: vi.fn((key: string, error: string | null) => {
            if (error === null) {
                delete errors[key];
            } else {
                errors[key] = error;
            }
        }),
        clearErrors: vi.fn(() => {
            errors = {};
        }),
        getState: vi.fn(() => state),
        getValue: vi.fn((path: string) => {
            return path.split('.').reduce((acc, part) => {
                if (acc && typeof acc === 'object' && part in acc) {
                    return (acc as Record<string, unknown>)[part];
                }
                return undefined;
            }, state as unknown);
        }),
    };

    return store;
}

/**
 * Create a mock API client for testing
 */
export function createMockApiClient() {
    const responses: Record<string, unknown> = {};
    const errors: Record<string, Error> = {};

    return {
        call: vi.fn(async (api: string, _method: string, _args?: Record<string, unknown>) => {
            if (errors[api]) {
                throw errors[api];
            }
            return responses[api];
        }),
        setResponse: (api: string, response: unknown) => {
            responses[api] = response;
        },
        setError: (api: string, error: Error) => {
            errors[api] = error;
        },
        reset: () => {
            Object.keys(responses).forEach(key => delete responses[key]);
            Object.keys(errors).forEach(key => delete errors[key]);
        },
    };
}

/**
 * Create a mock navigate function for testing
 */
export function createMockNavigate() {
    const navigate = vi.fn() as ReturnType<typeof vi.fn> & { 
        history: Array<{ to: string; options?: Record<string, unknown> }>;
        reset: () => void;
    };
    navigate.history = [];
    
    navigate.mockImplementation((to: string, options?: Record<string, unknown>) => {
        navigate.history.push({ to, options });
    });

    navigate.reset = () => {
        navigate.mockClear();
        navigate.history = [];
    };

    return navigate;
}
