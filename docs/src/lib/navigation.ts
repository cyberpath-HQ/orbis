export interface NavItem {
    title:     string
    href?:     string
    items?:    Array<NavItem>
    badge?:    string
    external?: boolean
}

export interface NavSection {
    title: string
    items: Array<NavItem>
}

export const docsNav: Array<NavSection> = [
    {
        title: `Getting Started`,
        items: [
            {
                title: `Introduction`,
                href:  `/docs/`,
            },
            {
                title: `Installation`,
                href:  `/docs/getting-started/installation/`,
            },
            {
                title: `Quick Start`,
                href:  `/docs/getting-started/quickstart/`,
            },
            {
                title: `Project Structure`,
                href:  `/docs/getting-started/project-structure/`,
            },
        ],
    },
    {
        title: `Core Concepts`,
        items: [
            {
                title: `Architecture`,
                href:  `/docs/core-concepts/architecture/`,
            },
            {
                title: `Schema System`,
                href:  `/docs/core-concepts/schema-system/`,
            },
            {
                title: `Plugin System`,
                href:  `/docs/core-concepts/plugin-system/`,
            },
            {
                title: `State Management`,
                href:  `/docs/core-concepts/state-management/`,
            },
            {
                title: `Event Handling`,
                href:  `/docs/core-concepts/event-handling/`,
            },
            {
                title: `Expressions`,
                href:  `/docs/core-concepts/expressions/`,
            },
        ],
    },
    {
        title: `Components`,
        items: [
            {
                title: `Overview`,
                href:  `/docs/components/overview/`,
            },
            {
                title: `Layout`,
                href:  `/docs/components/layout/`,
            },
            {
                title: `Typography`,
                href:  `/docs/components/typography/`,
            },
            {
                title: `Forms`,
                items: [
                    {
                        title: `Input`,
                        href:  `/docs/components/forms/input/`,
                    },
                    {
                        title: `Button`,
                        href:  `/docs/components/forms/button/`,
                    },
                    {
                        title: `Select`,
                        href:  `/docs/components/forms/select/`,
                    },
                    {
                        title: `Checkbox`,
                        href:  `/docs/components/forms/checkbox/`,
                    },
                ],
            },
            {
                title: `Data Display`,
                href:  `/docs/components/data-display/`,
            },
            {
                title: `Feedback`,
                href:  `/docs/components/feedback/`,
            },
            {
                title: `Navigation`,
                items: [
                    {
                        title: `Tabs`,
                        href:  `/docs/components/navigation/tabs/`,
                    },
                    {
                        title: `Breadcrumb`,
                        href:  `/docs/components/navigation/breadcrumb/`,
                    },
                ],
            },
            {
                title: `Overlays`,
                items: [
                    {
                        title: `Dialog`,
                        href:  `/docs/components/overlays/dialog/`,
                    },
                    {
                        title: `Tooltip`,
                        href:  `/docs/components/overlays/tooltip/`,
                    },
                ],
            },
        ],
    },
    {
        title: `Actions`,
        items: [
            {
                title: `Overview`,
                href:  `/docs/actions/overview/`,
            },
            {
                title: `Update State`,
                href:  `/docs/actions/update-state/`,
            },
            {
                title: `Navigate`,
                href:  `/docs/actions/navigate/`,
            },
            {
                title: `Call API`,
                href:  `/docs/actions/call-api/`,
            },
            {
                title: `Show Toast`,
                href:  `/docs/actions/show-toast/`,
            },
            {
                title: `Dialogs`,
                href:  `/docs/actions/dialogs/`,
            },
            {
                title: `Form Actions`,
                href:  `/docs/actions/form-actions/`,
            },
            {
                title: `Flow Control`,
                href:  `/docs/actions/flow-control/`,
            },
            {
                title: `Utility Actions`,
                href:  `/docs/actions/utility-actions/`,
            },
        ],
    },
    {
        title: `API Reference`,
        items: [
            {
                title: `Overview`,
                href:  `/docs/api-reference/overview/`,
            },
            {
                title: `Types Reference`,
                href:  `/docs/api-reference/types-reference/`,
            },
            {
                title: `Expressions`,
                href:  `/docs/api-reference/expressions/`,
            },
            {
                title: `State Management`,
                href:  `/docs/api-reference/state-management/`,
            },
            {
                title: `Event Handlers`,
                href:  `/docs/api-reference/event-handlers/`,
            },
            {
                title: `Data Sources`,
                href:  `/docs/api-reference/data-sources/`,
            },
            {
                title: `Validation`,
                href:  `/docs/api-reference/validation/`,
            },
            {
                title: `Special Values`,
                href:  `/docs/api-reference/special-values/`,
            },
        ],
    },
    {
        title: `Plugin Development`,
        items: [
            {
                title: `Overview`,
                href:  `/docs/plugin-development/overview/`,
            },
            {
                title: `Building Plugins`,
                href:  `/docs/plugin-development/building-plugins/`,
            },
            {
                title: `Plugin Manifest`,
                href:  `/docs/plugin-development/manifest/`,
            },
            {
                title: `Page Definitions`,
                href:  `/docs/plugin-development/page-definitions/`,
            },
            {
                title: `WASM Plugins`,
                href:  `/docs/plugin-development/wasm-plugins/`,
            },
            {
                title: `Testing Plugins`,
                href:  `/docs/plugin-development/testing-plugins/`,
            },
            {
                title: `Best Practices`,
                href:  `/docs/plugin-development/best-practices/`,
            },
        ],
    },
    {
        title: `Configuration`,
        items: [
            {
                title: `Overview`,
                href:  `/docs/configuration/overview/`,
            },
            {
                title: `Database`,
                href:  `/docs/configuration/database/`,
            },
            {
                title: `Authentication`,
                href:  `/docs/configuration/authentication/`,
            },
            {
                title: `Server`,
                href:  `/docs/configuration/server/`,
            },
            {
                title: `TLS Security`,
                href:  `/docs/configuration/tls-security/`,
            },
        ],
    },
    {
        title: `Deployment`,
        items: [
            {
                title: `Overview`,
                href:  `/docs/deployment/overview/`,
            },
            {
                title: `Standalone Mode`,
                href:  `/docs/deployment/standalone/`,
            },
            {
                title: `Client-Server Mode`,
                href:  `/docs/deployment/client-server/`,
            },
            {
                title: `Docker`,
                href:  `/docs/deployment/docker/`,
            },
        ],
    },
    {
        title: `Resources`,
        items: [
            {
                title: `Roadmap`,
                href:  `/docs/roadmap/`,
            },
            {
                title: `Changelog`,
                href:  `/docs/changelog/`,
            },
        ],
    },
];

export const mainNav: Array<NavItem> = [
    {
        title: `Documentation`,
        href:  `/docs/`,
    },
    {
        title: `Components`,
        href:  `/docs/components/overview/`,
    },
    {
        title: `API Reference`,
        href:  `/docs/api-reference/overview/`,
    },
    {
        title:    `GitHub`,
        href:     `https://github.com/cyberpath-HQ/orbis`,
        external: true,
    },
];
