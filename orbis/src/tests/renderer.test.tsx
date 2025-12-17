/**
 * Schema renderer tests
 * Tests for ComponentRenderer and individual component renderers
 */

import React from 'react';
import {
    describe,
    it,
    expect,
    vi,
    beforeEach
} from 'vitest';
import {
    render,
    screen,
    fireEvent,
    waitFor
} from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';

import { SchemaRenderer } from '@/lib/renderer';
import { createPageStateStore } from '@/lib/state';
import type { ComponentSchema, TextSchema, HeadingSchema, ButtonSchema, ContainerSchema, ConditionalSchema, LoopSchema, BadgeSchema, AlertSchema, ProgressSchema, CardSchema, GridSchema, FlexSchema, DividerSchema, SkeletonSchema, EmptyStateSchema, FragmentSchema } from '@/types/schema';
import type { StateDefinition, StateFieldType } from '@/types/schema/base';
import type { ApiClient } from '@/lib/actions';

// Mock API client
const mockApiClient: ApiClient = {
    call: vi.fn().mockResolvedValue({
        data: {
            success: true,
        },
    }),
};

// Helper to create a store for testing with correct type inference
function createTestStore(initialState: Record<string, { type: StateFieldType; default: unknown }>): ReturnType<typeof createPageStateStore> {
    return createPageStateStore(initialState as StateDefinition);
}

// Simple helper for common types
function state<T>(type: StateFieldType, defaultValue: T): { type: StateFieldType; default: T } {
    return {
        type,
        default: defaultValue,
    };
}

// Test wrapper component that properly uses the zustand hook
function TestRenderer({
    schema,
    storeHook,
}: {
    schema:    ComponentSchema
    storeHook: ReturnType<typeof createTestStore>
}): React.ReactElement {
    // Call the store hook inside a component context
    const storeState = storeHook();

    return (
        <SchemaRenderer
            schema={schema}
            state={storeState}
            apiClient={mockApiClient}
        />
    );
}

// Wrapper with router for testing
function renderWithRouter(
    schema: ComponentSchema,
    storeHook: ReturnType<typeof createTestStore>
): ReturnType<typeof render> {
    return render(
        <MemoryRouter>
            <TestRenderer schema={schema} storeHook={storeHook} />
        </MemoryRouter>
    );
}

describe(`SchemaRenderer`, () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe(`Text component`, () => {
        it(`should render text content`, () => {
            const store = createTestStore({});
            const schema: TextSchema = {
                type:    `Text`,
                id:      `text-1`,
                content: `Hello, World!`,
            };

            renderWithRouter(schema, store);

            expect(screen.getByText(`Hello, World!`)).toBeInTheDocument();
        });

        it(`should interpolate state values`, () => {
            const store = createTestStore({
                name: state(`string`, `John`),
            });
            const schema: TextSchema = {
                type:    `Text`,
                id:      `text-1`,
                content: `Hello, {{name}}!`,
            };

            renderWithRouter(schema, store);

            expect(screen.getByText(`Hello, John!`)).toBeInTheDocument();
        });

        it(`should apply variant classes`, () => {
            const store = createTestStore({});
            const schema: TextSchema = {
                type:    `Text`,
                id:      `text-caption`,
                content: `Caption text`,
                variant: `caption`,
            };

            renderWithRouter(schema, store);

            const element = screen.getByText(`Caption text`);
            expect(element).toHaveClass(`text-sm`);
            expect(element).toHaveClass(`text-muted-foreground`);
        });
    });

    describe(`Heading component`, () => {
        it(`should render heading with correct level`, () => {
            const store = createTestStore({});
            const schema: HeadingSchema = {
                type:  `Heading`,
                id:    `heading-1`,
                text:  `Page Title`,
                level: 1,
            };

            renderWithRouter(schema, store);

            const heading = screen.getByRole(`heading`, {
                level: 1,
            });
            expect(heading).toBeInTheDocument();
            expect(heading).toHaveTextContent(`Page Title`);
        });

        it(`should default to h1 when no level specified`, () => {
            const store = createTestStore({});
            const schema: HeadingSchema = {
                type: `Heading`,
                id:   `heading-default`,
                text: `Default Heading`,
            };

            renderWithRouter(schema, store);

            expect(screen.getByRole(`heading`, {
                level: 1,
            })).toBeInTheDocument();
        });

        it(`should apply level-specific classes`, () => {
            const store = createTestStore({});
            const schema: HeadingSchema = {
                type:  `Heading`,
                id:    `heading-2`,
                text:  `Section Title`,
                level: 2,
            };

            renderWithRouter(schema, store);

            const heading = screen.getByRole(`heading`, {
                level: 2,
            });
            expect(heading).toHaveClass(`text-3xl`);
            expect(heading).toHaveClass(`font-semibold`);
        });
    });

    describe(`Button component`, () => {
        it(`should render button with label`, () => {
            const store = createTestStore({});
            const schema: ButtonSchema = {
                type:  `Button`,
                id:    `btn-1`,
                label: `Click Me`,
            };

            renderWithRouter(schema, store);

            expect(screen.getByRole(`button`, {
                name: `Click Me`,
            })).toBeInTheDocument();
        });

        it(`should execute onClick actions`, async() => {
            const store = createTestStore({
                clicked: state(`boolean`, false),
            });
            const schema: ButtonSchema = {
                type:   `Button`,
                id:     `btn-click`,
                label:  `Submit`,
                events: {
                    onClick: [
                        {
                            type:  `updateState`,
                            path:  `clicked`,
                            value: true,
                        },
                    ],
                },
            };

            renderWithRouter(schema, store);
            fireEvent.click(screen.getByRole(`button`));

            await waitFor(() => {
                expect(store.getState().getValue(`clicked`)).toBe(true);
            });
        });

        it(`should be disabled via disabled property as boolean`, () => {
            const store = createTestStore({});
            const schema: ButtonSchema = {
                type:     `Button`,
                id:       `btn-disabled`,
                label:    `Submit`,
                disabled: true,
            };

            renderWithRouter(schema, store);

            expect(screen.getByRole(`button`)).toBeDisabled();
        });

        it(`should apply variant styles`, () => {
            const store = createTestStore({});
            const schema: ButtonSchema = {
                type:    `Button`,
                id:      `btn-destructive`,
                label:   `Delete`,
                variant: `destructive`,
            };

            renderWithRouter(schema, store);

            // Button should exist with variant applied (shadcn applies classes)
            expect(screen.getByRole(`button`, {
                name: `Delete`,
            })).toBeInTheDocument();
        });
    });

    describe(`Container component`, () => {
        it(`should render children`, () => {
            const store = createTestStore({});
            const schema: ContainerSchema = {
                type:     `Container`,
                id:       `container-1`,
                children: [
                    {
                        type:    `Text`,
                        id:      `child-1`,
                        content: `Child 1`,
                    },
                    {
                        type:    `Text`,
                        id:      `child-2`,
                        content: `Child 2`,
                    },
                ],
            };

            renderWithRouter(schema, store);

            expect(screen.getByText(`Child 1`)).toBeInTheDocument();
            expect(screen.getByText(`Child 2`)).toBeInTheDocument();
        });

        it(`should apply className`, () => {
            const store = createTestStore({});
            const schema: ContainerSchema = {
                type:      `Container`,
                id:        `styled-container`,
                className: `flex gap-4 p-4`,
                children:  [
                    {
                        type:    `Text`,
                        id:      `child`,
                        content: `Content`,
                    },
                ],
            };

            renderWithRouter(schema, store);

            const container = screen.getByText(`Content`).parentElement;
            expect(container).toHaveClass(`flex`);
            expect(container).toHaveClass(`gap-4`);
            expect(container).toHaveClass(`p-4`);
        });
    });

    describe(`Conditional component`, () => {
        it(`should render then branch when condition is true`, () => {
            const store = createTestStore({
                showContent: state(`boolean`, true),
            });
            const schema: ConditionalSchema = {
                type:      `Conditional`,
                id:        `cond-1`,
                condition: `{{showContent}}`,
                then:      {
                    type:    `Text`,
                    id:      `cond-text`,
                    content: `Visible content`,
                },
            };

            renderWithRouter(schema, store);

            expect(screen.getByText(`Visible content`)).toBeInTheDocument();
        });

        it(`should not render then when condition is false`, () => {
            const store = createTestStore({
                showContent: state(`boolean`, false),
            });
            const schema: ConditionalSchema = {
                type:      `Conditional`,
                id:        `cond-2`,
                condition: `{{showContent}}`,
                then:      {
                    type:    `Text`,
                    id:      `cond-text`,
                    content: `Hidden content`,
                },
            };

            renderWithRouter(schema, store);

            expect(screen.queryByText(`Hidden content`)).not.toBeInTheDocument();
        });

        it(`should render else branch when condition is false`, () => {
            const store = createTestStore({
                showContent: state(`boolean`, false),
            });
            const schema: ConditionalSchema = {
                type:      `Conditional`,
                id:        `cond-fallback`,
                condition: `{{showContent}}`,
                then:      {
                    type:    `Text`,
                    id:      `main`,
                    content: `Main content`,
                },
                else: {
                    type:    `Text`,
                    id:      `fallback`,
                    content: `Fallback content`,
                },
            };

            renderWithRouter(schema, store);

            expect(screen.queryByText(`Main content`)).not.toBeInTheDocument();
            expect(screen.getByText(`Fallback content`)).toBeInTheDocument();
        });
    });

    describe(`visibility expressions`, () => {
        it(`should hide component when visible is false`, () => {
            const store = createTestStore({
                isVisible: state(`boolean`, false),
            });
            const schema: TextSchema = {
                type:    `Text`,
                id:      `hidden-text`,
                content: `Should not be visible`,
                visible: `{{isVisible}}`,
            };

            renderWithRouter(schema, store);

            expect(screen.queryByText(`Should not be visible`)).not.toBeInTheDocument();
        });

        it(`should show component when visible is true`, () => {
            const store = createTestStore({
                isVisible: state(`boolean`, true),
            });
            const schema: TextSchema = {
                type:    `Text`,
                id:      `visible-text`,
                content: `Should be visible`,
                visible: `{{isVisible}}`,
            };

            renderWithRouter(schema, store);

            expect(screen.getByText(`Should be visible`)).toBeInTheDocument();
        });

        it(`should support comparison expressions`, () => {
            const store = createTestStore({
                count: state(`number`, 5),
            });
            const schema: TextSchema = {
                type:    `Text`,
                id:      `comparison-text`,
                content: `Count is high`,
                visible: `{{count}} > 3`,
            };

            renderWithRouter(schema, store);

            expect(screen.getByText(`Count is high`)).toBeInTheDocument();
        });
    });

    describe(`Loop component`, () => {
        it(`should render items from array`, () => {
            const store = createTestStore({
                items: state(`array`, [
                    {
                        id:   1,
                        name: `Item 1`,
                    },
                    {
                        id:   2,
                        name: `Item 2`,
                    },
                    {
                        id:   3,
                        name: `Item 3`,
                    },
                ]),
            });
            const schema: LoopSchema = {
                type:       `Loop`,
                id:         `loop-1`,
                dataSource: `state:items`,
                template:   {
                    type:    `Text`,
                    id:      `loop-item`,
                    content: `{{$item.name}}`,
                },
            };

            renderWithRouter(schema, store);

            expect(screen.getByText(`Item 1`)).toBeInTheDocument();
            expect(screen.getByText(`Item 2`)).toBeInTheDocument();
            expect(screen.getByText(`Item 3`)).toBeInTheDocument();
        });

        it(`should render empty state when no items`, () => {
            const store = createTestStore({
                items: state(`array`, []),
            });
            const schema: LoopSchema = {
                type:          `Loop`,
                id:            `loop-empty`,
                dataSource:    `state:items`,
                template:      {
                    type:    `Text`,
                    id:      `item`,
                    content: `{{$item.name}}`,
                },
                emptyTemplate: {
                    type:    `Text`,
                    id:      `empty`,
                    content: `No items found`,
                },
            };

            renderWithRouter(schema, store);

            expect(screen.getByText(`No items found`)).toBeInTheDocument();
        });
    });

    describe(`Badge component`, () => {
        it(`should render badge with text`, () => {
            const store = createTestStore({});
            const schema: BadgeSchema = {
                type: `Badge`,
                id:   `badge-1`,
                text: `Active`,
            };

            renderWithRouter(schema, store);

            expect(screen.getByText(`Active`)).toBeInTheDocument();
        });

        it(`should apply variant`, () => {
            const store = createTestStore({});
            const schema: BadgeSchema = {
                type:    `Badge`,
                id:      `badge-destructive`,
                text:    `Error`,
                variant: `destructive`,
            };

            renderWithRouter(schema, store);

            expect(screen.getByText(`Error`)).toBeInTheDocument();
        });
    });

    describe(`Alert component`, () => {
        it(`should render alert with title and message`, () => {
            const store = createTestStore({});
            const schema: AlertSchema = {
                type:    `Alert`,
                id:      `alert-1`,
                variant: `default`,
                title:   `Warning`,
                message: `This is a warning message`,
            };

            renderWithRouter(schema, store);

            expect(screen.getByText(`Warning`)).toBeInTheDocument();
            expect(screen.getByText(`This is a warning message`)).toBeInTheDocument();
        });
    });

    describe(`Progress component`, () => {
        it(`should render progress bar`, () => {
            const store = createTestStore({});
            const schema: ProgressSchema = {
                type:  `Progress`,
                id:    `progress-1`,
                value: `50`,
            };

            renderWithRouter(schema, store);

            // Progress bar should exist
            const progressBar = screen.getByRole(`progressbar`);
            expect(progressBar).toBeInTheDocument();
        });

        it(`should interpolate value from state`, () => {
            const store = createTestStore({
                progressValue: state(`number`, 75),
            });
            const schema: ProgressSchema = {
                type:  `Progress`,
                id:    `progress-dynamic`,
                value: `{{progressValue}}`,
            };

            renderWithRouter(schema, store);

            expect(screen.getByRole(`progressbar`)).toBeInTheDocument();
        });
    });

    describe(`Card component`, () => {
        it(`should render card with title and content`, () => {
            const store = createTestStore({});
            const schema: CardSchema = {
                type:    `Card`,
                id:      `card-1`,
                title:   `Card Title`,
                content: {
                    type:    `Text`,
                    id:      `card-content`,
                    content: `Card body content`,
                },
            };

            renderWithRouter(schema, store);

            expect(screen.getByText(`Card Title`)).toBeInTheDocument();
            expect(screen.getByText(`Card body content`)).toBeInTheDocument();
        });

        it(`should render card with footer`, () => {
            const store = createTestStore({});
            const schema: CardSchema = {
                type:    `Card`,
                id:      `card-with-footer`,
                title:   `Title`,
                content: {
                    type:    `Text`,
                    id:      `content`,
                    content: `Body`,
                },
                footer: {
                    type:  `Button`,
                    id:    `card-btn`,
                    label: `Action`,
                },
            };

            renderWithRouter(schema, store);

            expect(screen.getByRole(`button`, {
                name: `Action`,
            })).toBeInTheDocument();
        });
    });

    describe(`Grid component`, () => {
        it(`should render grid with children`, () => {
            const store = createTestStore({});
            const schema: GridSchema = {
                type:     `Grid`,
                id:       `grid-1`,
                columns:  3,
                gap:      `1rem`,
                children: [
                    {
                        type:    `Text`,
                        id:      `grid-item-1`,
                        content: `Grid Item 1`,
                    },
                    {
                        type:    `Text`,
                        id:      `grid-item-2`,
                        content: `Grid Item 2`,
                    },
                ],
            };

            renderWithRouter(schema, store);

            expect(screen.getByText(`Grid Item 1`)).toBeInTheDocument();
            expect(screen.getByText(`Grid Item 2`)).toBeInTheDocument();
        });
    });

    describe(`Flex component`, () => {
        it(`should render flex container with children`, () => {
            const store = createTestStore({});
            const schema: FlexSchema = {
                type:      `Flex`,
                id:        `flex-1`,
                direction: `row`,
                gap:       `1rem`,
                children:  [
                    {
                        type:    `Text`,
                        id:      `flex-item-1`,
                        content: `Flex Item 1`,
                    },
                    {
                        type:    `Text`,
                        id:      `flex-item-2`,
                        content: `Flex Item 2`,
                    },
                ],
            };

            renderWithRouter(schema, store);

            expect(screen.getByText(`Flex Item 1`)).toBeInTheDocument();
            expect(screen.getByText(`Flex Item 2`)).toBeInTheDocument();
        });
    });

    describe(`Divider component`, () => {
        it(`should render divider`, () => {
            const store = createTestStore({});
            const schema: DividerSchema = {
                type:        `Divider`,
                id:          `divider-1`,
                orientation: `horizontal`,
            };

            const {
                container,
            } = renderWithRouter(schema, store);

            // Divider uses data-slot attribute
            expect(container.querySelector(`[data-slot="separator"]`)).toBeInTheDocument();
        });
    });

    describe(`Skeleton component`, () => {
        it(`should render skeleton loader`, () => {
            const store = createTestStore({});
            const schema: SkeletonSchema = {
                type:   `Skeleton`,
                id:     `skeleton-1`,
                width:  `100%`,
                height: `20px`,
            };

            const {
                container,
            } = renderWithRouter(schema, store);

            // Skeleton renders with data-slot
            expect(container.querySelector(`[data-slot="skeleton"]`)).toBeInTheDocument();
        });
    });

    describe(`EmptyState component`, () => {
        it(`should render empty state with title`, () => {
            const store = createTestStore({});
            const schema: EmptyStateSchema = {
                type:        `EmptyState`,
                id:          `empty-1`,
                title:       `No data`,
                description: `There is nothing to display`,
            };

            renderWithRouter(schema, store);

            expect(screen.getByText(`No data`)).toBeInTheDocument();
            expect(screen.getByText(`There is nothing to display`)).toBeInTheDocument();
        });

        it(`should render action button when provided`, () => {
            const store = createTestStore({});
            const schema: EmptyStateSchema = {
                type:   `EmptyState`,
                id:     `empty-action`,
                title:  `No items`,
                action: {
                    type:  `Button`,
                    id:    `add-btn`,
                    label: `Add Item`,
                },
            };

            renderWithRouter(schema, store);

            expect(screen.getByRole(`button`, {
                name: `Add Item`,
            })).toBeInTheDocument();
        });
    });

    describe(`Fragment component`, () => {
        it(`should render children without wrapper`, () => {
            const store = createTestStore({});
            const schema: FragmentSchema = {
                type:     `Fragment`,
                id:       `fragment-1`,
                children: [
                    {
                        type:    `Text`,
                        id:      `frag-1`,
                        content: `Fragment Child 1`,
                    },
                    {
                        type:    `Text`,
                        id:      `frag-2`,
                        content: `Fragment Child 2`,
                    },
                ],
            };

            renderWithRouter(schema, store);

            expect(screen.getByText(`Fragment Child 1`)).toBeInTheDocument();
            expect(screen.getByText(`Fragment Child 2`)).toBeInTheDocument();
        });
    });
});
