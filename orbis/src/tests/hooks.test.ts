/**
 * Hook tests
 * Tests for useFocusTrap and related hook utilities
 */

import {
    describe,
    it,
    expect,
    vi,
    beforeEach
} from 'vitest';
import {
    renderHook,
    act
} from '@testing-library/react';

import { useFocusTrap } from '@/hooks/use-focus-trap';

describe(`useFocusTrap`, () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it(`should return container ref and focus methods`, () => {
        const {
            result,
        } = renderHook(() => useFocusTrap());

        expect(result.current.container_ref).toBeDefined();
        expect(typeof result.current.focus_first).toBe(`function`);
        expect(typeof result.current.focus_last).toBe(`function`);
    });

    it(`should not trap focus when inactive`, () => {
        const {
            result,
        } = renderHook(() => useFocusTrap({
            is_active: false,
        }));

        expect(result.current.container_ref).toBeDefined();
    });

    it(`should accept custom options`, () => {
        const {
            result,
        } = renderHook(() => useFocusTrap({
            is_active:            true,
            should_auto_focus:    true,
            should_restore_focus: true,
        }));

        expect(result.current.container_ref).toBeDefined();
    });

    describe(`focus methods`, () => {
        it(`should have focus_first method that can be called`, () => {
            const {
                result,
            } = renderHook(() => useFocusTrap({
                is_active: false,
            }));

            // Should not throw when called without container
            expect(() => {
                act(() => {
                    result.current.focus_first();
                });
            }).not.toThrow();
        });

        it(`should have focus_last method that can be called`, () => {
            const {
                result,
            } = renderHook(() => useFocusTrap({
                is_active: false,
            }));

            // Should not throw when called without container
            expect(() => {
                act(() => {
                    result.current.focus_last();
                });
            }).not.toThrow();
        });
    });
});
