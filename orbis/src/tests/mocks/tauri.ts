/* eslint-disable */
/**
 * Mock implementation of Tauri API for testing
 */

import { vi } from 'vitest';

type InvokeHandler = (cmd: string, args?: Record<string, unknown>) => unknown;

// Store for custom invoke handlers
let invokeHandlers: Record<string, InvokeHandler> = {};

// Default responses for common commands
const defaultResponses: Record<string, unknown> = {
    health_check: {
        status:    `ok`,
        mode:      `Standalone`,
        timestamp: new Date().toISOString(),
    },
    get_mode: {
        mode:          `standalone`,
        is_standalone: true,
        is_client:     false,
        is_server:     false,
    },
    get_plugins: {
        plugins: [],
        count:   0,
    },
    get_plugin_pages: {
        pages: [],
        count: 0,
    },
    get_session:    null,
    verify_session: false,
    login: {
        success: true,
        message: `Login successful`,
        session: null,
    },
    logout: {
        success: true,
        message: `Logged out successfully`,
    },
};

/**
 * Mock invoke function
 */
export const invoke = vi.fn(async <T>(cmd: string, args?: Record<string, unknown>): Promise<T> => {
    // Check for custom handler first
    if (invokeHandlers[cmd]) {
        return invokeHandlers[cmd](cmd, args) as T;
    }

    // Return default response if available
    if (cmd in defaultResponses) {
        return defaultResponses[cmd] as T;
    }

    // Throw for unknown commands
    throw new Error(`Unknown Tauri command: ${ cmd }`);
});

/**
 * Set a custom handler for a specific command
 */
export function setInvokeHandler(cmd: string, handler: InvokeHandler): void {
    invokeHandlers[cmd] = handler;
}

/**
 * Set multiple handlers at once
 */
export function setInvokeHandlers(handlers: Record<string, InvokeHandler>): void {
    invokeHandlers = {
        ...invokeHandlers,
        ...handlers,
    };
}

/**
 * Reset all custom handlers
 */
export function resetInvokeHandlers(): void {
    invokeHandlers = {};
    invoke.mockClear();
}

/**
 * Set a mock response for a specific command
 */
export function setMockResponse(cmd: string, response: unknown): void {
    invokeHandlers[cmd] = () => response;
}

/**
 * Make a command throw an error
 */
export function setMockError(cmd: string, error: string | Error): void {
    invokeHandlers[cmd] = () => {
        throw typeof error === `string` ? new Error(error) : error;
    };
}

// Re-export other Tauri API mocks
export const transformCallback = vi.fn();
export const convertFileSrc = vi.fn((path: string) => `asset://localhost/${ path }`);

// Event system mock
const eventListeners: Record<string, Array<(event: unknown) => void>> = {};

export const listen = vi.fn(async (event: string, handler: (event: unknown) => void) => {
    if (!eventListeners[event]) {
        eventListeners[event] = [];
    }
    eventListeners[event].push(handler);

    // Return unlisten function
    return () => {
        const idx = eventListeners[event].indexOf(handler);
        if (idx > -1) {
            eventListeners[event].splice(idx, 1);
        }
    };
});

export const emit = vi.fn(async (event: string, payload?: unknown) => {
    if (eventListeners[event]) {
        eventListeners[event].forEach((handler) => {
            handler({
                event,
                payload,
            });
        });
    }
});

export const once = vi.fn(async (event: string, handler: (event: unknown) => void) => {
    const unlisten = await listen(event, (e) => {
        handler(e);
        unlisten();
    });
    return unlisten;
});

/**
 * Trigger an event manually for testing
 */
export function triggerEvent(event: string, payload?: unknown): void {
    if (eventListeners[event]) {
        eventListeners[event].forEach((handler) => {
            handler({
                event,
                payload,
            });
        });
    }
}

/**
 * Reset all event listeners
 */
export function resetEventListeners(): void {
    Object.keys(eventListeners).forEach((key) => {
        delete eventListeners[key];
    });
}
