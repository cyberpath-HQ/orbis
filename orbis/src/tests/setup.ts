/* eslint-disable */
/**
 * Global test setup for Vitest
 */

import '@testing-library/jest-dom/vitest';
import { cleanup } from '@testing-library/react';
import {
    afterEach,
    beforeAll,
    vi
} from 'vitest';

// Cleanup after each test
afterEach(() => {
    cleanup();
    vi.clearAllMocks();
});

// Mock window.matchMedia
beforeAll(() => {
    Object.defineProperty(window, `matchMedia`, {
        writable: true,
        value:    vi.fn().mockImplementation((query: string) => ({
            matches:             false,
            media:               query,
            onchange:            null,
            addListener:         vi.fn(),
            removeListener:      vi.fn(),
            addEventListener:    vi.fn(),
            removeEventListener: vi.fn(),
            dispatchEvent:       vi.fn(),
        })),
    });
});

// Mock ResizeObserver
beforeAll(() => {
    global.ResizeObserver = vi.fn().mockImplementation(() => ({
        observe:    vi.fn(),
        unobserve:  vi.fn(),
        disconnect: vi.fn(),
    }));
});

// Mock IntersectionObserver
beforeAll(() => {
    global.IntersectionObserver = vi.fn().mockImplementation(() => ({
        root:        null,
        rootMargin:  ``,
        thresholds:  [],
        observe:     vi.fn(),
        unobserve:   vi.fn(),
        disconnect:  vi.fn(),
        takeRecords: vi.fn().mockReturnValue([]),
    }));
});

// Suppress console errors during tests unless explicitly testing error handling
const OriginalConsoleError = console.error;
beforeAll(() => {
    console.error = (...args: Array<unknown>) => {
        // Filter out React act() warnings
        const message = args[0];
        if (
            typeof message === `string` &&
            (message.includes(`act(...)`) || message.includes(`not wrapped in act`))
        ) {
            return;
        }
        OriginalConsoleError.apply(console, args);
    };
});
