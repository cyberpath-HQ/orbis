/**
 * State management tests
 * Tests for createPageStateStore, getNestedValue, setNestedValue,
 * interpolateExpression, and evaluateBooleanExpression
 */

import {
    describe,
    it,
    expect
} from 'vitest';

import {
    createPageStateStore,
    getNestedValue,
    setNestedValue,
    interpolateExpression,
    evaluateBooleanExpression
} from '@/lib/state';

describe(`getNestedValue`, () => {
    const testObject = {
        user: {
            name:    `John`,
            profile: {
                email: `john@example.com`,
                age:   30,
            },
            tags: [
                `admin`,
                `user`,
            ],
        },
        count:    42,
        isActive: true,
        items:    [
            {
                id:   1,
                name: `Item 1`,
            },
            {
                id:   2,
                name: `Item 2`,
            },
        ],
    };

    it(`should get top-level value`, () => {
        expect(getNestedValue(testObject, `count`)).toBe(42);
        expect(getNestedValue(testObject, `isActive`)).toBe(true);
    });

    it(`should get nested object value`, () => {
        expect(getNestedValue(testObject, `user.name`)).toBe(`John`);
        expect(getNestedValue(testObject, `user.profile.email`)).toBe(`john@example.com`);
        expect(getNestedValue(testObject, `user.profile.age`)).toBe(30);
    });

    it(`should get array elements by index`, () => {
        expect(getNestedValue(testObject, `user.tags.0`)).toBe(`admin`);
        expect(getNestedValue(testObject, `user.tags.1`)).toBe(`user`);
        expect(getNestedValue(testObject, `items.0.name`)).toBe(`Item 1`);
        expect(getNestedValue(testObject, `items.1.id`)).toBe(2);
    });

    it(`should return undefined for non-existent paths`, () => {
        expect(getNestedValue(testObject, `nonexistent`)).toBeUndefined();
        expect(getNestedValue(testObject, `user.nonexistent`)).toBeUndefined();
        expect(getNestedValue(testObject, `user.profile.nonexistent.deep`)).toBeUndefined();
    });

    it(`should return undefined for null/undefined base`, () => {
        expect(getNestedValue(null as unknown as Record<string, unknown>, `any.path`)).toBeUndefined();
        expect(getNestedValue(undefined as unknown as Record<string, unknown>, `any.path`)).toBeUndefined();
    });
});

describe(`setNestedValue`, () => {
    it(`should set top-level value`, () => {
        const obj = {
            count: 0,
        };
        const result = setNestedValue(obj, `count`, 42);
        expect(result.count).toBe(42);
    });

    it(`should set nested value`, () => {
        const obj = {
            user: {
                name: `John`,
            },
        };
        const result = setNestedValue(obj, `user.name`, `Jane`);
        expect(result.user.name).toBe(`Jane`);
    });

    it(`should create intermediate objects`, () => {
        const obj: Record<string, unknown> = {};
        const result = setNestedValue(obj, `user.profile.email`, `test@example.com`) as { user: { profile: { email: string } } };
        expect(result.user.profile.email).toBe(`test@example.com`);
    });

    it(`should set array elements`, () => {
        const obj = {
            items: [
                `a`,
                `b`,
                `c`,
            ],
        };
        const result = setNestedValue(obj, `items.1`, `modified`);
        expect(result.items[1]).toBe(`modified`);
    });

    it(`should not mutate original object`, () => {
        const original = {
            user: {
                name: `John`,
            },
        };
        const result = setNestedValue(original, `user.name`, `Jane`);
        expect(original.user.name).toBe(`John`);
        expect(result.user.name).toBe(`Jane`);
    });

    it(`should prevent prototype pollution with __proto__`, () => {
        const obj = {
            user: {
                name: `John`,
            },
        };
        const result = setNestedValue(obj, `__proto__.polluted`, true);
        expect(result).toEqual(obj);
        expect(({} as Record<string, unknown>).polluted).toBeUndefined();
    });

    it(`should prevent prototype pollution with constructor`, () => {
        const obj = {
            user: {
                name: `John`,
            },
        };
        const result = setNestedValue(obj, `constructor.polluted`, true);
        expect(result).toEqual(obj);
    });

    it(`should prevent prototype pollution with prototype`, () => {
        const obj = {
            user: {
                name: `John`,
            },
        };
        const result = setNestedValue(obj, `prototype.polluted`, true);
        expect(result).toEqual(obj);
    });

    it(`should prevent prototype pollution in nested paths`, () => {
        const obj = {
            user: {
                name: `John`,
            },
        };
        const result = setNestedValue(obj, `user.__proto__.polluted`, true);
        expect(result).toEqual(obj);
    });
});

describe(`interpolateExpression`, () => {
    const state = {
        user: {
            name:  `John`,
            email: `john@example.com`,
        },
        count:   42,
        message: `Hello`,
    };

    it(`should return non-expression strings as-is`, () => {
        expect(interpolateExpression(`plain text`, state)).toBe(`plain text`);
        expect(interpolateExpression(``, state)).toBe(``);
    });

    it(`should interpolate single expression`, () => {
        expect(interpolateExpression(`{{count}}`, state)).toBe(`42`);
        expect(interpolateExpression(`{{user.name}}`, state)).toBe(`John`);
    });

    it(`should interpolate expression in template string`, () => {
        expect(interpolateExpression(`Hello, {{user.name}}!`, state)).toBe(`Hello, John!`);
        expect(interpolateExpression(`Count: {{count}}`, state)).toBe(`Count: 42`);
    });

    it(`should handle multiple expressions`, () => {
        const result = interpolateExpression(`{{message}}, {{user.name}}!`, state);
        expect(result).toBe(`Hello, John!`);
    });

    it(`should return empty string for undefined values in templates`, () => {
        const result = interpolateExpression(`Value: {{nonexistent}}`, state);
        expect(result).toBe(`Value: `);
    });

    it(`should handle props and context`, () => {
        const stateWithProps = {
            ...state,
            props: {
                title: `My Title`,
            },
            context: {
                theme: `dark`,
            },
        };
        expect(interpolateExpression(`{{props.title}}`, stateWithProps)).toBe(`My Title`);
        expect(interpolateExpression(`{{context.theme}}`, stateWithProps)).toBe(`dark`);
    });
});

describe(`evaluateBooleanExpression`, () => {
    const state = {
        count:    5,
        isActive: true,
        user:     {
            name: `John`,
            role: `admin`,
        },
        items:   [],
        nullVal: null,
    };

    it(`should return boolean values as-is`, () => {
        expect(evaluateBooleanExpression(true, state)).toBe(true);
        expect(evaluateBooleanExpression(false, state)).toBe(false);
    });

    it(`should evaluate simple state references`, () => {
        expect(evaluateBooleanExpression(`{{isActive}}`, state)).toBe(true);
    });

    it(`should evaluate comparison expressions`, () => {
        expect(evaluateBooleanExpression(`{{count}} > 3`, state)).toBe(true);
        expect(evaluateBooleanExpression(`{{count}} < 3`, state)).toBe(false);
        expect(evaluateBooleanExpression(`{{count}} === 5`, state)).toBe(true);
        expect(evaluateBooleanExpression(`{{count}} !== 5`, state)).toBe(false);
    });

    it(`should evaluate equality expressions`, () => {
        expect(evaluateBooleanExpression(`{{user.role}} === admin`, state)).toBe(true);
        expect(evaluateBooleanExpression(`{{user.name}} === Jane`, state)).toBe(false);
    });

    it(`should evaluate logical expressions`, () => {
        expect(evaluateBooleanExpression(`{{isActive}} && {{count}} > 0`, state)).toBe(true);
        expect(evaluateBooleanExpression(`{{isActive}} || {{count}} < 0`, state)).toBe(true);
        expect(evaluateBooleanExpression(`!{{isActive}}`, state)).toBe(false);
    });

    it(`should handle undefined state references`, () => {
        expect(evaluateBooleanExpression(`{{nonexistent}}`, state)).toBe(false);
    });
});

describe(`createPageStateStore`, () => {
    it(`should create store with initial state from definition`, () => {
        const store = createPageStateStore({
            count: {
                type:    `number`,
                default: 0,
            },
            message: {
                type:    `string`,
                default: `Hello`,
            },
        });

        const state = store.getState();
        expect(state.state.count).toBe(0);
        expect(state.state.message).toBe(`Hello`);
    });

    it(`should update state with setState`, () => {
        const store = createPageStateStore({
            count: {
                type:    `number`,
                default: 0,
            },
        });

        store.getState().setState(`count`, 42);
        expect(store.getState().state.count).toBe(42);
    });

    it(`should update nested state with setState`, () => {
        const store = createPageStateStore({
            user: {
                type:    `object`,
                default: {
                    name:    `John`,
                    profile: {
                        age: 25,
                    },
                },
            },
        });

        store.getState().setState(`user.profile.age`, 30);
        expect((store.getState().state.user as Record<string, unknown>).profile).toEqual({
            age: 30,
        });
    });

    it(`should get value with getValue method`, () => {
        const store = createPageStateStore({
            count: {
                type:    `number`,
                default: 42,
            },
            nested: {
                type:    `object`,
                default: {
                    value: `test`,
                },
            },
        });

        expect(store.getState().getValue(`count`)).toBe(42);
        expect(store.getState().getValue(`nested.value`)).toBe(`test`);
    });

    it(`should reset state to initial values`, () => {
        const definition = {
            count: {
                type:    `number` as const,
                default: 0,
            },
            message: {
                type:    `string` as const,
                default: `initial`,
            },
        };
        const store = createPageStateStore(definition);

        store.getState().setState(`count`, 100);
        store.getState().setState(`message`, `changed`);

        store.getState().resetState(definition);

        expect(store.getState().state.count).toBe(0);
        expect(store.getState().state.message).toBe(`initial`);
    });

    it(`should merge state values`, () => {
        const store = createPageStateStore({
            user: {
                type:    `object`,
                default: {
                    name: `John`,
                    age:  25,
                },
            },
        });

        store.getState().mergeState(`user`, {
            age:   30,
            email: `john@example.com`,
        });

        const user = store.getState().state.user as Record<string, unknown>;
        expect(user.name).toBe(`John`);
        expect(user.age).toBe(30);
        expect(user.email).toBe(`john@example.com`);
    });

    it(`should manage loading state`, () => {
        const store = createPageStateStore({});

        expect(store.getState().loading.fetch).toBeUndefined();

        store.getState().setLoading(`fetch`, true);
        expect(store.getState().loading.fetch).toBe(true);

        store.getState().setLoading(`fetch`, false);
        expect(store.getState().loading.fetch).toBe(false);
    });

    it(`should manage error state`, () => {
        const store = createPageStateStore({});

        store.getState().setError(`validation`, `Invalid input`);
        expect(store.getState().errors.validation).toBe(`Invalid input`);

        store.getState().setError(`validation`, null);
        expect(store.getState().errors.validation).toBeUndefined();
    });

    it(`should clear all errors`, () => {
        const store = createPageStateStore({});

        store.getState().setError(`error1`, `First error`);
        store.getState().setError(`error2`, `Second error`);

        store.getState().clearErrors();

        expect(store.getState().errors).toEqual({});
    });

    it(`should subscribe to state changes`, () => {
        const store = createPageStateStore({
            count: {
                type:    `number`,
                default: 0,
            },
        });

        let notifiedCount = 0;
        const unsubscribe = store.subscribe(() => {
            notifiedCount += 1;
        });

        store.getState().setState(`count`, 1);
        store.getState().setState(`count`, 2);

        expect(notifiedCount).toBe(2);

        unsubscribe();

        store.getState().setState(`count`, 3);
        expect(notifiedCount).toBe(2);
    });

    it(`should prevent prototype pollution in setState with __proto__`, () => {
        const store = createPageStateStore({
            user: {
                type:    `object`,
                default: {
                    name: `John`,
                },
            },
        });

        store.getState().setState(`__proto__.polluted`, true);
        expect(({} as Record<string, unknown>).polluted).toBeUndefined();
    });

    it(`should prevent prototype pollution in setState with constructor`, () => {
        const store = createPageStateStore({
            user: {
                type:    `object`,
                default: {
                    name: `John`,
                },
            },
        });

        store.getState().setState(`constructor.polluted`, true);
        expect(store.getState().state.constructor).toBe(Object);
    });

    it(`should prevent prototype pollution in setState with nested dangerous properties`, () => {
        const store = createPageStateStore({
            user: {
                type:    `object`,
                default: {
                    name: `John`,
                },
            },
        });

        store.getState().setState(`user.__proto__.polluted`, true);
        expect(({} as Record<string, unknown>).polluted).toBeUndefined();
    });

    it(`should prevent prototype pollution in mergeState with __proto__`, () => {
        const store = createPageStateStore({
            user: {
                type:    `object`,
                default: {
                    name: `John`,
                },
            },
        });

        store.getState().mergeState(`__proto__`, {
            polluted: true,
        });
        expect(({} as Record<string, unknown>).polluted).toBeUndefined();
    });

    it(`should prevent prototype pollution in mergeState with constructor`, () => {
        const store = createPageStateStore({
            user: {
                type:    `object`,
                default: {
                    name: `John`,
                },
            },
        });

        store.getState().mergeState(`constructor`, {
            polluted: true,
        });
        expect(store.getState().state.constructor).toBeUndefined();
    });

    it(`should prevent prototype pollution in mergeState with nested dangerous properties`, () => {
        const store = createPageStateStore({
            user: {
                type:    `object`,
                default: {
                    name: `John`,
                },
            },
        });

        store.getState().mergeState(`user.__proto__`, {
            polluted: true,
        });
        expect(({} as Record<string, unknown>).polluted).toBeUndefined();
    });
});
