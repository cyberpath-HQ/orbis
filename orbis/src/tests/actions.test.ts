/**
 * Action executor tests
 * Tests for executeAction, resolveValue, and action handlers
 */

import {
    describe,
    it,
    expect,
    vi,
    beforeEach
} from 'vitest';

import {
    executeAction,
    executeActions,
    type ActionContext,
    type ApiClient
} from '@/lib/actions';
import { createPageStateStore } from '@/lib/state';
import type { NavigateFunction } from 'react-router-dom';

// Mock navigator
const mockNavigate: NavigateFunction = vi.fn() as unknown as NavigateFunction;

// Mock API client
const mockApiClient: ApiClient = {
    call: vi.fn().mockResolvedValue({
        data: {
            success: true,
        },
    }),
};

// Create a fresh store for each test
function createTestStore() {
    return createPageStateStore({
        count: {
            type:    `number`,
            default: 0,
        },
        message: {
            type:    `string`,
            default: ``,
        },
        user: {
            type:    `object`,
            default: {
                name:  `John`,
                email: `john@example.com`,
            },
        },
    });
}

function createContext(
    store: ReturnType<typeof createTestStore>,
    overrides: Partial<ActionContext> = {}
): ActionContext {
    return {
        state:     store.getState(),
        navigate:  mockNavigate,
        apiClient: mockApiClient,
        ...overrides,
    };
}

describe(`executeAction`, () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe(`updateState action`, () => {
        it(`should update simple state value`, async() => {
            const store = createTestStore();
            const context = createContext(store);

            await executeAction({
                type:  `updateState`,
                path:  `count`,
                value: 42,
            }, context);

            expect(store.getState().getValue(`count`)).toBe(42);
        });

        it(`should update nested state value`, async() => {
            const store = createTestStore();
            const context = createContext(store);

            await executeAction({
                type:  `updateState`,
                path:  `user.name`,
                value: `Jane`,
            }, context);

            expect(store.getState().getValue(`user.name`)).toBe(`Jane`);
        });

        it(`should resolve from property for special variables`, async() => {
            const store = createTestStore();
            const context = createContext(store, {
                event: {
                    value: `event-value`,
                },
            });

            await executeAction({
                type: `updateState`,
                path: `message`,
                from: `$event.value`,
            }, context);

            expect(store.getState().getValue(`message`)).toBe(`event-value`);
        });

        it(`should use value directly when no from is provided`, async() => {
            const store = createTestStore();
            const context = createContext(store);

            await executeAction({
                type:  `updateState`,
                path:  `message`,
                value: `literal value`,
            }, context);

            expect(store.getState().getValue(`message`)).toBe(`literal value`);
        });
    });

    describe(`navigate action`, () => {
        it(`should navigate to specified path`, async() => {
            const store = createTestStore();
            const context = createContext(store);

            await executeAction({
                type: `navigate`,
                to:   `/dashboard`,
            }, context);

            expect(mockNavigate).toHaveBeenCalledWith(`/dashboard`);
        });

        it(`should interpolate path expressions`, async() => {
            const store = createTestStore();
            const context = createContext(store);
            store.getState().setState(`user`, {
                id: 123,
            });

            await executeAction({
                type: `navigate`,
                to:   `/users/{{user.id}}`,
            }, context);

            expect(mockNavigate).toHaveBeenCalledWith(`/users/123`);
        });
    });

    describe(`setLoading action`, () => {
        it(`should set loading state for target`, async() => {
            const store = createTestStore();
            const context = createContext(store);

            await executeAction({
                type:    `setLoading`,
                target:  `fetch`,
                loading: true,
            }, context);

            expect(store.getState().loading.fetch).toBe(true);
        });

        it(`should default to global loading`, async() => {
            const store = createTestStore();
            const context = createContext(store);

            await executeAction({
                type:    `setLoading`,
                loading: true,
            }, context);

            expect(store.getState().loading.global).toBe(true);
        });
    });

    describe(`conditional action`, () => {
        it(`should execute then actions when condition is true`, async() => {
            const store = createTestStore();
            const context = createContext(store);
            store.getState().setState(`count`, 10);

            await executeAction({
                type:      `conditional`,
                condition: `{{count}} > 5`,
                then:      [
                    {
                        type:  `updateState`,
                        path:  `message`,
                        value: `count is greater than 5`,
                    },
                ],
                else: [
                    {
                        type:  `updateState`,
                        path:  `message`,
                        value: `count is less than or equal to 5`,
                    },
                ],
            }, context);

            expect(store.getState().getValue(`message`)).toBe(`count is greater than 5`);
        });

        it(`should execute else actions when condition is false`, async() => {
            const store = createTestStore();
            const context = createContext(store);
            store.getState().setState(`count`, 3);

            await executeAction({
                type:      `conditional`,
                condition: `{{count}} > 5`,
                then:      [
                    {
                        type:  `updateState`,
                        path:  `message`,
                        value: `count is greater than 5`,
                    },
                ],
                else: [
                    {
                        type:  `updateState`,
                        path:  `message`,
                        value: `count is less than or equal to 5`,
                    },
                ],
            }, context);

            expect(store.getState().getValue(`message`)).toBe(`count is less than or equal to 5`);
        });
    });

    describe(`sequence action`, () => {
        it(`should execute actions in order`, async() => {
            const store = createTestStore();
            const context = createContext(store);

            await executeAction({
                type:    `sequence`,
                actions: [
                    {
                        type:  `updateState`,
                        path:  `count`,
                        value: 1,
                    },
                    {
                        type:  `updateState`,
                        path:  `count`,
                        value: 2,
                    },
                    {
                        type:  `updateState`,
                        path:  `count`,
                        value: 3,
                    },
                ],
            }, context);

            expect(store.getState().getValue(`count`)).toBe(3);
        });
    });

    describe(`showDialog action`, () => {
        it(`should open dialog with data`, async() => {
            const store = createTestStore();
            const context = createContext(store);

            await executeAction({
                type:     `showDialog`,
                dialogId: `confirm`,
                data:     {
                    title:   `Confirm`,
                    message: `Are you sure?`,
                },
            }, context);

            const dialogs = store.getState().getValue(`__dialogs.confirm`) as {
                open: boolean
                data: Record<string, unknown>
            };
            expect(dialogs.open).toBe(true);
            expect(dialogs.data.title).toBe(`Confirm`);
        });
    });

    describe(`closeDialog action`, () => {
        it(`should close dialog`, async() => {
            const store = createTestStore();
            const context = createContext(store);

            store.getState().setState(`__dialogs.confirm`, {
                open: true,
            });

            await executeAction({
                type:     `closeDialog`,
                dialogId: `confirm`,
            }, context);

            const dialogs = store.getState().getValue(`__dialogs.confirm`) as {
                open: boolean
            };
            expect(dialogs.open).toBe(false);
        });
    });
});

describe(`executeActions`, () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it(`should execute multiple actions in sequence`, async() => {
        const store = createTestStore();
        const context = createContext(store);

        await executeActions([
            {
                type:  `updateState`,
                path:  `count`,
                value: 10,
            },
            {
                type:  `updateState`,
                path:  `message`,
                value: `Updated`,
            },
        ], context);

        expect(store.getState().getValue(`count`)).toBe(10);
        expect(store.getState().getValue(`message`)).toBe(`Updated`);
    });

    it(`should handle empty actions array`, async() => {
        const store = createTestStore();
        const context = createContext(store);

        await executeActions([], context);

        expect(store.getState().getValue(`count`)).toBe(0);
    });

    it(`should throw on undefined actions`, async() => {
        const store = createTestStore();
        const context = createContext(store);

        await expect(async() => {
            await executeActions(undefined as unknown as Array<never>, context);
        }).rejects.toThrow();
    });
});

describe(`special variables resolution`, () => {
    it(`should resolve $row variable via from property`, async() => {
        const store = createTestStore();
        const context = createContext(store, {
            row: {
                id:   1,
                name: `Item 1`,
            },
        });

        await executeAction({
            type: `updateState`,
            path: `message`,
            from: `$row.name`,
        }, context);

        expect(store.getState().getValue(`message`)).toBe(`Item 1`);
    });

    it(`should resolve $item variable via from property`, async() => {
        const store = createTestStore();
        const context = createContext(store, {
            item: {
                value: `test-item`,
            },
        });

        await executeAction({
            type: `updateState`,
            path: `message`,
            from: `$item.value`,
        }, context);

        expect(store.getState().getValue(`message`)).toBe(`test-item`);
    });

    it(`should resolve $index variable via from property`, async() => {
        const store = createTestStore();
        const context = createContext(store, {
            index: 5,
        });

        await executeAction({
            type: `updateState`,
            path: `count`,
            from: `$index`,
        }, context);

        expect(store.getState().getValue(`count`)).toBe(5);
    });
});
