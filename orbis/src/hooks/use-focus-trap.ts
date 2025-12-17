/**
 * Focus trap hook for modal accessibility
 * Keeps focus within a container element
 */

import {
    useEffect,
    useRef,
    useCallback
} from 'react';

/**
 * Selector for focusable elements
 */
const FOCUSABLE_SELECTOR = [
    `a[href]`,
    `button:not([disabled])`,
    `textarea:not([disabled])`,
    `input:not([disabled]):not([type="hidden"])`,
    `select:not([disabled])`,
    `[tabindex]:not([tabindex="-1"])`,
    `[contenteditable="true"]`,
].join(`, `);

/**
 * Constants
 */
const FIRST_INDEX = 0;
const LAST_OFFSET = 1;
const RESTORE_FOCUS_DELAY = 0;

interface UseFocusTrapOptions {
    /**
     * Whether the trap is active
     */
    is_active?: boolean

    /**
     * Auto-focus the first focusable element when trap activates
     */
    should_auto_focus?: boolean

    /**
     * Return focus to the previously focused element when trap deactivates
     */
    should_restore_focus?: boolean

    /**
     * Initial element to focus (by selector or ref)
     */
    initial_focus?: string | React.RefObject<HTMLElement | null>
}

interface UseFocusTrapReturn {
    /**
     * Ref to attach to the container element
     */
    container_ref: React.RefObject<HTMLElement | null>

    /**
     * Manually focus the first focusable element
     */
    focus_first: () => void

    /**
     * Manually focus the last focusable element
     */
    focus_last: () => void
}

/**
 * Get all focusable elements within a container
 */
function getFocusableElements(container: HTMLElement): Array<HTMLElement> {
    const elements = container.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR);
    return Array.from(elements).filter((el) => {
        // Additional checks for visibility
        const style = window.getComputedStyle(el);
        return style.display !== `none` && style.visibility !== `hidden` && !el.hasAttribute(`inert`);
    });
}

/**
 * Focus trap hook
 */
export function useFocusTrap(options: UseFocusTrapOptions = {}): UseFocusTrapReturn {
    const {
        is_active = true,
        should_auto_focus = true,
        should_restore_focus = true,
        initial_focus,
    } = options;

    const container_ref = useRef<HTMLElement>(null);
    const previous_focus_ref = useRef<Element | null>(null);

    // Focus first focusable element
    const focus_first = useCallback(() => {
        const container = container_ref.current;
        if (!container) {
            return;
        }

        const elements = getFocusableElements(container);
        if (elements.length > FIRST_INDEX) {
            elements[FIRST_INDEX].focus();
        }
    }, []);

    // Focus last focusable element
    const focus_last = useCallback(() => {
        const container = container_ref.current;
        if (!container) {
            return;
        }

        const elements = getFocusableElements(container);
        if (elements.length > FIRST_INDEX) {
            elements[elements.length - LAST_OFFSET].focus();
        }
    }, []);

    // Handle keydown for tab trapping
    const handleKeyDown = useCallback((event: KeyboardEvent) => {
        if (event.key !== `Tab` || !container_ref.current) {
            return;
        }

        const elements = getFocusableElements(container_ref.current);
        if (elements.length === FIRST_INDEX) {
            event.preventDefault();
            return;
        }

        const [ first_element ] = elements;
        const last_element = elements[elements.length - LAST_OFFSET];
        const active_element = document.activeElement;

        if (event.shiftKey) {
            // Shift + Tab: going backwards
            if (active_element === first_element || !container_ref.current.contains(active_element)) {
                event.preventDefault();
                last_element.focus();
            }
        }
        else if (active_element === last_element || !container_ref.current.contains(active_element)) {
            // Tab: going forwards
            event.preventDefault();
            first_element.focus();
        }
    }, []);

    // Setup effect
    useEffect(() => {
        if (!is_active || !container_ref.current) {
            return;
        }

        // Store previously focused element
        if (should_restore_focus) {
            previous_focus_ref.current = document.activeElement;
        }

        // Auto-focus initial element or first focusable
        if (should_auto_focus) {
            if (initial_focus) {
                // Focus specified element
                if (typeof initial_focus === `string`) {
                    const element = container_ref.current.querySelector<HTMLElement>(initial_focus);
                    if (element) {
                        element.focus();
                    }
                    else {
                        focus_first();
                    }
                }
                else if (initial_focus.current) {
                    initial_focus.current.focus();
                }
                else {
                    focus_first();
                }
            }
            else {
                focus_first();
            }
        }

        // Add keyboard listener
        document.addEventListener(`keydown`, handleKeyDown);

        // Cleanup
        return (): void => {
            document.removeEventListener(`keydown`, handleKeyDown);

            // Restore focus
            if (should_restore_focus && previous_focus_ref.current) {
                const element_to_focus = previous_focus_ref.current;

                // Use setTimeout to ensure focus restoration happens after modal unmounts
                setTimeout(() => {
                    if (element_to_focus instanceof HTMLElement && document.body.contains(element_to_focus)) {
                        element_to_focus.focus();
                    }
                }, RESTORE_FOCUS_DELAY);
            }
        };
    }, [
        is_active,
        should_auto_focus,
        should_restore_focus,
        initial_focus,
        focus_first,
        handleKeyDown,
    ]);

    return {
        container_ref,
        focus_first,
        focus_last,
    };
}

/**
 * Simpler hook for just returning focus on unmount
 */
export function useReturnFocus(): void {
    const previous_focus_ref = useRef<Element | null>(null);

    useEffect(() => {
        previous_focus_ref.current = document.activeElement;

        return (): void => {
            const element = previous_focus_ref.current;
            if (element instanceof HTMLElement && document.body.contains(element)) {
                element.focus();
            }
        };
    }, []);
}

/**
 * Hook to focus an element on mount
 */
export function useFocusOnMount(ref: React.RefObject<HTMLElement | null>): void {
    useEffect(() => {
        if (ref.current) {
            ref.current.focus();
        }
    }, [ ref ]);
}

export default useFocusTrap;
