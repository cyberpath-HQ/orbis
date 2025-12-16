import React from 'react';
import type { ComponentSchema } from '../types/plugin';

interface PluginRendererProps {
    schema:    ComponentSchema
    data?:     Record<string, unknown>
    handlers?: Record<string, (...args: Array<unknown>) => void>
}

/**
 * Renders a plugin UI from a JSON component schema.
 * This is the main entry point for rendering plugin pages.
 */
export function PluginRenderer({
    schema, data = {}, handlers = {},
}: PluginRendererProps) {
    return <ComponentRenderer schema={schema} data={data} handlers={handlers} />;
}

interface ComponentRendererProps {
    schema:   ComponentSchema
    data:     Record<string, unknown>
    handlers: Record<string, (...args: Array<unknown>) => void>
}

/**
 * Recursive component renderer that handles all component types.
 */
function ComponentRenderer({
    schema, data, handlers,
}: ComponentRendererProps): React.ReactElement | null {
    // Check visibility
    if (schema.visible === false) {
        return null;
    }
    if (typeof schema.visible === `string`) {
        const isVisible = evaluateExpression(schema.visible, data);
        if (!isVisible) {
            return null;
        }
    }

    const baseProps = {
        id:        schema.id,
        className: schema.className,
        style:     schema.style,
    };

    switch (schema.type) {
        case `Container`:
            return (
                <div {...baseProps}>
                    {schema.children.map((child, index) => (
                        <ComponentRenderer key={child.id || index} schema={child} data={data} handlers={handlers} />
                    ))}
                </div>
            );

        case `Text`:
            return (
                <span {...baseProps} className={`orbis-text orbis-text--${ schema.variant || `body` } ${ schema.className || `` }`}>
                    {interpolateText(schema.content, data)}
                </span>
            );

        case `Heading`: {
            const HeadingTag = `h${ schema.level }` as const;
            type HeadingElement = `h1` | `h2` | `h3` | `h4` | `h5` | `h6`;
            const Tag = HeadingTag as HeadingElement;
            return React.createElement(
                Tag,
                {
                    ...baseProps,
                    className: `orbis-heading orbis-heading--${ schema.level } ${ schema.className || `` }`,
                },
                interpolateText(schema.text, data)
            );
        }

        case `Button`:
            return (
                <button
                    {...baseProps}
                    className={`orbis-button orbis-button--${ schema.variant || `primary` } orbis-button--${ schema.size || `md` } ${ schema.className || `` }`}
                    disabled={evaluateBoolean(schema.disabled, data)}
                    onClick={() => schema.onClick && handlers[schema.onClick]?.()}
                >
                    {schema.label}
                </button>
            );

        case `Input`:
            return <InputRenderer schema={schema} data={data} handlers={handlers} />;

        case `Form`:
            return (
                <form
                    {...baseProps}
                    className={`orbis-form ${ schema.className || `` }`}
                    onSubmit={(e) => {
                        e.preventDefault();
                        const formData = new FormData(e.currentTarget);
                        handlers[schema.onSubmit]?.(Object.fromEntries(formData));
                    }}
                >
                    {schema.fields.map((field, index) => (
                        <InputRenderer key={field.name || index} schema={field} data={data} handlers={handlers} />
                    ))}
                    <button type="submit" className="orbis-button orbis-button--primary">
                        {schema.submitLabel || `Submit`}
                    </button>
                </form>
            );

        case `Card`:
            return (
                <div {...baseProps} className={`orbis-card ${ schema.hoverable ? `orbis-card--hoverable` : `` } ${ schema.className || `` }`}>
                    {(schema.title || schema.subtitle || schema.header) && (
                        <div className="orbis-card__header">
                            {schema.header
? (
                <ComponentRenderer schema={schema.header} data={data} handlers={handlers} />
              )
: (
                <>
                    {schema.title && <h3 className="orbis-card__title">{interpolateText(schema.title, data)}</h3>}
                    {schema.subtitle && <p className="orbis-card__subtitle">{interpolateText(schema.subtitle, data)}</p>}
                </>
              )}
                        </div>
                    )}
                    <div className="orbis-card__content">
                        <ComponentRenderer schema={schema.content} data={data} handlers={handlers} />
                    </div>
                    {schema.footer && (
                        <div className="orbis-card__footer">
                            <ComponentRenderer schema={schema.footer} data={data} handlers={handlers} />
                        </div>
                    )}
                </div>
            );

        case `Table`:
            const tableData = getNestedValue(data, schema.dataSource) as Array<Record<string, unknown>> || [];
            return (
                <table {...baseProps} className={`orbis-table ${ schema.className || `` }`}>
                    <thead>
                        <tr>
                            {schema.columns.map((col) => (
                                <th key={col.key} style={{
                                    width: col.width,
                                }}>
                                    {col.header}
                                </th>
                            ))}
                        </tr>
                    </thead>
                    <tbody>
                        {tableData.map((row, rowIndex) => (
                            <tr key={schema.rowKey ? String(row[schema.rowKey]) : rowIndex}>
                                {schema.columns.map((col) => (
                                    <td key={col.key}>
                                        {col.render
? (
                      <ComponentRenderer schema={col.render} data={{
                          ...data,
                          row,
                          rowIndex,
                      }} handlers={handlers} />
                    )
: (
                      String(row[col.key] ?? ``)
                    )}
                                    </td>
                                ))}
                            </tr>
                        ))}
                    </tbody>
                </table>
            );

        case `List`:
            const listData = getNestedValue(data, schema.dataSource) as Array<unknown> || [];
            if (listData.length === 0 && schema.emptyText) {
                return <p className="orbis-list--empty">{schema.emptyText}</p>;
            }
            return (
                <ul {...baseProps} className={`orbis-list ${ schema.className || `` }`}>
                    {listData.map((item, index) => (
                        <li key={index} className="orbis-list__item">
                            <ComponentRenderer schema={schema.itemTemplate} data={{
                                ...data,
                                item,
                                index,
                            }} handlers={handlers} />
                        </li>
                    ))}
                </ul>
            );

        case `Image`:
            return (
                <img
                    {...baseProps}
                    src={interpolateText(schema.src, data)}
                    alt={schema.alt || ``}
                    style={{
                        ...schema.style,
                        width:     schema.width,
                        height:    schema.height,
                        objectFit: schema.fit,
                    }}
                    className={`orbis-image ${ schema.className || `` }`}
                />
            );

        case `Icon`:
            return (
                <span {...baseProps} className={`orbis-icon orbis-icon--${ schema.size || `md` } ${ schema.className || `` }`} style={{
                    color: schema.color,
                    ...schema.style,
                }}>
                    {/* Icon implementation would depend on icon library */}
                    {schema.name}
                </span>
            );

        case `Link`:
            return (
                <a
                    {...baseProps}
                    href={interpolateText(schema.href, data)}
                    target={schema.external ? `_blank` : undefined}
                    rel={schema.external ? `noopener noreferrer` : undefined}
                    className={`orbis-link ${ schema.className || `` }`}
                >
                    {interpolateText(schema.text, data)}
                </a>
            );

        case `Badge`:
            return (
                <span {...baseProps} className={`orbis-badge orbis-badge--${ schema.variant || `default` } ${ schema.className || `` }`}>
                    {interpolateText(schema.text, data)}
                </span>
            );

        case `Alert`:
            return (
                <div {...baseProps} className={`orbis-alert orbis-alert--${ schema.variant } ${ schema.className || `` }`} role="alert">
                    {schema.title && <strong className="orbis-alert__title">{schema.title}</strong>}
                    <span className="orbis-alert__message">{interpolateText(schema.message, data)}</span>
                </div>
            );

        case `Progress`:
            const value = typeof schema.value === `string` ? evaluateExpression(schema.value, data) as number : schema.value;
            const max = schema.max || 100;
            const percentage = (value / max) * 100;
            return (
                <div {...baseProps} className={`orbis-progress ${ schema.className || `` }`}>
                    <div className="orbis-progress__bar" style={{
                        width: `${ percentage }%`,
                    }} />
                    {schema.showLabel && <span className="orbis-progress__label">{percentage.toFixed(0)}%</span>}
                </div>
            );

        case `Tabs`:
            return <TabsRenderer schema={schema} data={data} handlers={handlers} />;

        case `Accordion`:
            return <AccordionRenderer schema={schema} data={data} handlers={handlers} />;

        case `Grid`:
            return (
                <div
                    {...baseProps}
                    className={`orbis-grid ${ schema.className || `` }`}
                    style={{
                        display:             `grid`,
                        gridTemplateColumns: `repeat(${ schema.columns }, 1fr)`,
                        gap:                 schema.gap || `1rem`,
                        ...schema.style,
                    }}
                >
                    {schema.children.map((child, index) => (
                        <ComponentRenderer key={child.id || index} schema={child} data={data} handlers={handlers} />
                    ))}
                </div>
            );

        case `Flex`:
            return (
                <div
                    {...baseProps}
                    className={`orbis-flex ${ schema.className || `` }`}
                    style={{
                        display:        `flex`,
                        flexDirection:  schema.direction || `row`,
                        justifyContent: mapJustify(schema.justify),
                        alignItems:     mapAlign(schema.align),
                        gap:            schema.gap || `0.5rem`,
                        flexWrap:       schema.wrap ? `wrap` : `nowrap`,
                        ...schema.style,
                    }}
                >
                    {schema.children.map((child, index) => (
                        <ComponentRenderer key={child.id || index} schema={child} data={data} handlers={handlers} />
                    ))}
                </div>
            );

        case `Spacer`:
            const spacerSizes = {
                xs: `0.25rem`,
                sm: `0.5rem`,
                md: `1rem`,
                lg: `1.5rem`,
                xl: `2rem`,
            };
            return <div style={{
                height: spacerSizes[schema.size],
                width:  `100%`,
            }} />;

        case `Divider`:
            return (
                <hr
                    {...baseProps}
                    className={`orbis-divider orbis-divider--${ schema.orientation || `horizontal` } ${ schema.className || `` }`}
                />
            );

        case `Custom`:
            // Custom components would be registered and looked up here
            return (
                <div {...baseProps} className="orbis-custom">
                    Custom component: {schema.component}
                </div>
            );

        default:
            console.warn(`Unknown component type:`, (schema as { type: string }).type);
            return null;
    }
}

// Helper components

function InputRenderer({
    schema, data, handlers: _handlers,
}: { schema:  ComponentSchema & { type: `Input` }
    data:     Record<string, unknown>
    handlers: Record<string, (...args: Array<unknown>) => void> }) {
    const isDisabled = evaluateBoolean(schema.disabled, data);
    const inputType = schema.inputType || `text`;

    const inputProps = {
        id:           schema.id || schema.name,
        name:         schema.name,
        required:     schema.required,
        disabled:     isDisabled,
        placeholder:  schema.placeholder,
        defaultValue: schema.defaultValue,
        className:    `orbis-input orbis-input--${ inputType } ${ schema.className || `` }`,
    };

    let input: React.ReactElement;

    switch (inputType) {
        case `textarea`:
            input = <textarea {...inputProps} />;
            break;

        case `checkbox`:
            input = <input {...inputProps} type="checkbox" defaultChecked={schema.defaultValue === `true`} />;
            break;

        case `radio`:
            input = (
                <div className="orbis-radio-group">
                    {schema.options?.map((opt) => (
                        <label key={opt.value} className="orbis-radio">
                            <input type="radio" name={schema.name} value={opt.value} />
                            {opt.label}
                        </label>
                    ))}
                </div>
            );
            break;

        case `select`:
            input = (
                <select {...inputProps}>
                    <option value="">{schema.placeholder || `Select...`}</option>
                    {schema.options?.map((opt) => (
                        <option key={opt.value} value={opt.value}>
                            {opt.label}
                        </option>
                    ))}
                </select>
            );
            break;

        default:
            input = <input {...inputProps} type={inputType} />;
    }

    return (
        <div className="orbis-field">
            {schema.label && (
                <label htmlFor={inputProps.id} className="orbis-field__label">
                    {schema.label}
                    {schema.required && <span className="orbis-field__required">*</span>}
                </label>
            )}
            {input}
        </div>
    );
}

function TabsRenderer({
    schema, data, handlers,
}: { schema:  ComponentSchema & { type: `Tabs` }
    data:     Record<string, unknown>
    handlers: Record<string, (...args: Array<unknown>) => void> }) {
    const [
        activeTab,
        setActiveTab,
    ] = React.useState(schema.defaultTab || schema.items[0]?.key);

    return (
        <div className={`orbis-tabs ${ schema.className || `` }`}>
            <div className="orbis-tabs__list" role="tablist">
                {schema.items.map((item) => (
                    <button
                        key={item.key}
                        role="tab"
                        aria-selected={activeTab === item.key}
                        disabled={item.disabled}
                        className={`orbis-tabs__tab ${ activeTab === item.key ? `orbis-tabs__tab--active` : `` }`}
                        onClick={() => setActiveTab(item.key)}
                    >
                        {item.label}
                    </button>
                ))}
            </div>
            <div className="orbis-tabs__content">
                {schema.items.map((item) => (
                    <div
                        key={item.key}
                        role="tabpanel"
                        hidden={activeTab !== item.key}
                        className="orbis-tabs__panel"
                    >
                        {activeTab === item.key && (
                            <ComponentRenderer schema={item.content} data={data} handlers={handlers} />
                        )}
                    </div>
                ))}
            </div>
        </div>
    );
}

function AccordionRenderer({
    schema, data, handlers,
}: { schema:  ComponentSchema & { type: `Accordion` }
    data:     Record<string, unknown>
    handlers: Record<string, (...args: Array<unknown>) => void> }) {
    const [
        openItems,
        setOpenItems,
    ] = React.useState<Set<string>>(new Set());

    const toggleItem = (key: string) => {
        setOpenItems((prev) => {
            const next = new Set(prev);
            if (next.has(key)) {
                next.delete(key);
            }
            else {
                if (!schema.allowMultiple) {
                    next.clear();
                }
                next.add(key);
            }
            return next;
        });
    };

    return (
        <div className={`orbis-accordion ${ schema.className || `` }`}>
            {schema.items.map((item) => (
                <div key={item.key} className={`orbis-accordion__item ${ openItems.has(item.key) ? `orbis-accordion__item--open` : `` }`}>
                    <button
                        className="orbis-accordion__trigger"
                        onClick={() => toggleItem(item.key)}
                        aria-expanded={openItems.has(item.key)}
                    >
                        {item.title}
                    </button>
                    {openItems.has(item.key) && (
                        <div className="orbis-accordion__content">
                            <ComponentRenderer schema={item.content} data={data} handlers={handlers} />
                        </div>
                    )}
                </div>
            ))}
        </div>
    );
}

// Utility functions

function interpolateText(text: string, data: Record<string, unknown>): string {
    return text.replace(/\{\{(\w+(?:\.\w+)*)\}\}/g, (_, path) => {
        const value = getNestedValue(data, path);
        return value !== undefined ? String(value) : ``;
    });
}

function getNestedValue(obj: Record<string, unknown>, path: string): unknown {
    return path.split(`.`).reduce((acc, part) => {
        if (acc && typeof acc === `object` && part in acc) {
            return (acc as Record<string, unknown>)[part];
        }
        return undefined;
    }, obj as unknown);
}

function evaluateExpression(expr: string, data: Record<string, unknown>): unknown {
    // Simple expression evaluation - in production, use a safe expression evaluator
    try {
        const interpolated = interpolateText(expr, data);

        // Handle simple boolean expressions
        if (interpolated === `true`) {
            return true;
        }
        if (interpolated === `false`) {
            return false;
        }

        // Try to parse as number
        const num = Number(interpolated);
        if (!isNaN(num)) {
            return num;
        }
        return interpolated;
    }
    catch {
        return expr;
    }
}

function evaluateBoolean(value: boolean | string | undefined, data: Record<string, unknown>): boolean {
    if (value === undefined) {
        return false;
    }
    if (typeof value === `boolean`) {
        return value;
    }
    return Boolean(evaluateExpression(value, data));
}

function mapJustify(value?: string): string {
    const map: Record<string, string> = {
        start:   `flex-start`,
        end:     `flex-end`,
        center:  `center`,
        between: `space-between`,
        around:  `space-around`,
    };
    return map[value || `start`] || `flex-start`;
}

function mapAlign(value?: string): string {
    const map: Record<string, string> = {
        start:   `flex-start`,
        end:     `flex-end`,
        center:  `center`,
        stretch: `stretch`,
    };
    return map[value || `stretch`] || `stretch`;
}

export default PluginRenderer;
