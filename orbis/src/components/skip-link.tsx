/**
 * Skip link component for keyboard navigation accessibility
 * Allows users to skip navigation and jump directly to main content
 */

import React from 'react';

interface SkipLinkProps {
    /**
     * Target element ID to skip to (without #)
     */
    targetId?: string

    /**
     * Link text (screen reader friendly)
     */
    children?: React.ReactNode
}

/**
 * SkipLink - Invisible link that becomes visible on focus
 * Place at the very top of the page, before any navigation
 */
export function SkipLink({
    targetId = `main-content`,
    children = `Skip to main content`,
}: SkipLinkProps): React.ReactElement {
    const handleClick = (e: React.MouseEvent<HTMLAnchorElement>): void => {
        e.preventDefault();
        const target = document.getElementById(targetId);
        if (target) {
            // Set focus to the target element
            target.setAttribute(`tabindex`, `-1`);
            target.focus();

            // Scroll into view
            target.scrollIntoView({
                behavior: `smooth`,
                block:    `start`,
            });
        }
    };

    return (
        <a
            href={`#${ targetId }`}
            onClick={handleClick}
            className="
                sr-only focus:not-sr-only
                focus:absolute focus:top-4 focus:left-4
                focus:z-9999 focus:px-4 focus:py-2
                focus:bg-primary focus:text-primary-foreground
                focus:rounded-md focus:shadow-lg
                focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2
                transition-all duration-200
            "
        >
            {children}
        </a>
    );
}

/**
 * SkipLinks - Multiple skip links for complex pages
 */
interface SkipLinksProps {
    links: Array<{
        targetId: string
        label:    string
    }>
}

export function SkipLinks({
    links,
}: SkipLinksProps): React.ReactElement {
    const wrapperClasses = [
        `sr-only`,
        `focus-within:not-sr-only`,
        `focus-within:fixed`,
        `focus-within:top-0`,
        `focus-within:left-0`,
        `focus-within:z-9999`,
        `focus-within:p-4`,
        `focus-within:bg-background`,
        `focus-within:shadow-lg`,
    ].join(` `);

    return (
        <div className={wrapperClasses}>
            <ul className="flex gap-4">
                {links.map((link) => (
                    <li key={link.targetId}>
                        <SkipLink targetId={link.targetId}>{link.label}</SkipLink>
                    </li>
                ))}
            </ul>
        </div>
    );
}

export default SkipLink;
