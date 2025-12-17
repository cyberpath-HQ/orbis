/**
 * Accessibility utilities for extracting and applying ARIA props from schema
 */

import type { AriaProps } from '@/types/schema/base';

/**
 * Resolve an expression value (simple version - full resolver is in renderer)
 */
function resolveExpression(
    value: boolean | string | undefined,
    context?: Record<string, unknown>
): string | boolean | undefined {
    if (typeof value === `string` && value.startsWith(`\${`) && value.endsWith(`}`) && context) {
        const path = value.slice(2, -1);
        const parts = path.split(`.`);
        let result: unknown = context;
        for (const part of parts) {
            if (result && typeof result === `object`) {
                result = (result as Record<string, unknown>)[part];
            }
            else {
                return undefined;
            }
        }
        return result as boolean | string;
    }
    return value;
}

/**
 * Convert schema ARIA props to DOM aria-* attributes
 */
export function extractAriaProps(
    schema: AriaProps,
    context?: Record<string, unknown>
): Record<string, string | boolean | number | undefined> {
    const ariaAttrs: Record<string, string | boolean | number | undefined> = {};

    // Basic ARIA
    if (schema.role) {
        ariaAttrs.role = schema.role;
    }
    if (schema.ariaLabel !== undefined) {
        const resolved = resolveExpression(schema.ariaLabel, context);
        if (resolved !== undefined) {
            ariaAttrs[`aria-label`] = String(resolved);
        }
    }
    if (schema.ariaLabelledBy) {
        ariaAttrs[`aria-labelledby`] = schema.ariaLabelledBy;
    }
    if (schema.ariaDescribedBy) {
        ariaAttrs[`aria-describedby`] = schema.ariaDescribedBy;
    }
    if (schema.ariaHidden !== undefined) {
        ariaAttrs[`aria-hidden`] = resolveExpression(schema.ariaHidden, context);
    }

    // Interactive states
    if (schema.ariaDisabled !== undefined) {
        ariaAttrs[`aria-disabled`] = resolveExpression(schema.ariaDisabled, context);
    }
    if (schema.ariaExpanded !== undefined) {
        ariaAttrs[`aria-expanded`] = resolveExpression(schema.ariaExpanded, context);
    }
    if (schema.ariaPressed !== undefined) {
        const value = schema.ariaPressed === `mixed` ? `mixed` : resolveExpression(schema.ariaPressed, context);
        ariaAttrs[`aria-pressed`] = value;
    }
    if (schema.ariaSelected !== undefined) {
        ariaAttrs[`aria-selected`] = resolveExpression(schema.ariaSelected, context);
    }
    if (schema.ariaChecked !== undefined) {
        const value = schema.ariaChecked === `mixed` ? `mixed` : resolveExpression(schema.ariaChecked, context);
        ariaAttrs[`aria-checked`] = value;
    }

    // Form/input
    if (schema.ariaRequired !== undefined) {
        ariaAttrs[`aria-required`] = resolveExpression(schema.ariaRequired, context);
    }
    if (schema.ariaInvalid !== undefined) {
        ariaAttrs[`aria-invalid`] = resolveExpression(schema.ariaInvalid, context);
    }
    if (schema.ariaErrorMessage) {
        ariaAttrs[`aria-errormessage`] = schema.ariaErrorMessage;
    }
    if (schema.ariaPlaceholder !== undefined) {
        const resolved = resolveExpression(schema.ariaPlaceholder, context);
        if (resolved !== undefined) {
            ariaAttrs[`aria-placeholder`] = String(resolved);
        }
    }

    // Live regions
    if (schema.ariaLive) {
        ariaAttrs[`aria-live`] = schema.ariaLive;
    }
    if (schema.ariaAtomic !== undefined) {
        ariaAttrs[`aria-atomic`] = resolveExpression(schema.ariaAtomic, context);
    }
    if (schema.ariaBusy !== undefined) {
        ariaAttrs[`aria-busy`] = resolveExpression(schema.ariaBusy, context);
    }
    if (schema.ariaRelevant) {
        ariaAttrs[`aria-relevant`] = schema.ariaRelevant;
    }

    // Relationships
    if (schema.ariaControls) {
        ariaAttrs[`aria-controls`] = schema.ariaControls;
    }
    if (schema.ariaOwns) {
        ariaAttrs[`aria-owns`] = schema.ariaOwns;
    }
    if (schema.ariaFlowTo) {
        ariaAttrs[`aria-flowto`] = schema.ariaFlowTo;
    }

    // Current state
    if (schema.ariaCurrent !== undefined) {
        ariaAttrs[`aria-current`] = schema.ariaCurrent;
    }

    // Keyboard
    if (schema.tabIndex !== undefined) {
        ariaAttrs.tabIndex = schema.tabIndex;
    }

    return ariaAttrs;
}

/**
 * Generate aria-label for icon-only buttons
 */
export function generateIconLabel(iconName: string): string {
    // Convert icon name from PascalCase to readable text
    // e.g., "ChevronLeft" -> "Chevron Left"
    return iconName
        .replace(/([A-Z])/g, ` $1`)
        .trim()
        .toLowerCase()
        .replace(/^\w/, (c) => c.toUpperCase());
}

/**
 * Announce text to screen readers via aria-live region
 */
export function announceToScreenReader(
    message: string,
    priority: `polite` | `assertive` = `polite`
): void {
    // Find or create the announcer element
    let announcer = document.getElementById(`sr-announcer`);
    if (!announcer) {
        announcer = document.createElement(`div`);
        announcer.id = `sr-announcer`;
        announcer.setAttribute(`role`, `status`);
        announcer.setAttribute(`aria-live`, priority);
        announcer.setAttribute(`aria-atomic`, `true`);
        announcer.className = `sr-only`;
        announcer.style.cssText = `
            position: absolute;
            width: 1px;
            height: 1px;
            padding: 0;
            margin: -1px;
            overflow: hidden;
            clip: rect(0, 0, 0, 0);
            white-space: nowrap;
            border: 0;
        `;
        document.body.appendChild(announcer);
    }

    // Update priority if needed
    announcer.setAttribute(`aria-live`, priority);

    // Clear and set message (forces announcement)
    announcer.textContent = ``;
    requestAnimationFrame(() => {
        if (announcer) {
            announcer.textContent = message;
        }
    });
}

interface AccessibleNameSchema {
    ariaLabel?:   string
    text?:        string
    title?:       string
    label?:       string
    placeholder?: string
}

/**
 * Get accessible name for an element based on schema
 */
export function getAccessibleName(schema: AccessibleNameSchema): string | undefined {
    return schema.ariaLabel ?? schema.text ?? schema.title ?? schema.label ?? schema.placeholder;
}
