/* eslint-disable */
/**
 * Test utilities and custom render functions
 */

import React from 'react';
import { render, type RenderOptions, type RenderResult } from '@testing-library/react';
import { TestRouter } from './mocks/router';

interface CustomRenderOptions extends Omit<RenderOptions, 'wrapper'> {
    initialEntries?: string[];
    wrapper?: React.ComponentType<{ children: React.ReactNode }>;
}

/**
 * Custom render function that includes router context
 */
export function renderWithRouter(
    ui: React.ReactElement,
    options: CustomRenderOptions = {}
): RenderResult {
    const { initialEntries = ['/'], wrapper: CustomWrapper, ...renderOptions } = options;

    const Wrapper = ({ children }: { children: React.ReactNode }): React.ReactElement => {
        const content = CustomWrapper ? <CustomWrapper>{children}</CustomWrapper> : children;
        return <TestRouter initialEntries={initialEntries}>{content}</TestRouter>;
    };

    return render(ui, { wrapper: Wrapper, ...renderOptions });
}

/**
 * Wait for a condition to be true
 */
export async function waitForCondition(
    condition: () => boolean,
    timeout = 5000,
    interval = 100
): Promise<void> {
    const startTime = Date.now();
    
    while (!condition()) {
        if (Date.now() - startTime > timeout) {
            throw new Error('Condition not met within timeout');
        }
        await new Promise(resolve => setTimeout(resolve, interval));
    }
}

/**
 * Create a deferred promise for async testing
 */
export function createDeferred<T>(): {
    promise: Promise<T>;
    resolve: (value: T) => void;
    reject: (reason?: unknown) => void;
} {
    let resolve!: (value: T) => void;
    let reject!: (reason?: unknown) => void;
    
    const promise = new Promise<T>((res, rej) => {
        resolve = res;
        reject = rej;
    });
    
    return { promise, resolve, reject };
}

/**
 * Flush all pending promises and timers
 */
export async function flushPromises(): Promise<void> {
    await new Promise(resolve => setTimeout(resolve, 0));
}

// Re-export testing-library utilities
export * from '@testing-library/react';
export { default as userEvent } from '@testing-library/user-event';
