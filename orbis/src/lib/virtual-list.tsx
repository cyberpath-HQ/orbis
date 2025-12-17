/**
 * Virtualized list and table components for performance optimization
 * Uses @tanstack/react-virtual for efficient rendering of large lists
 */

import React, {
    useRef,
    useMemo,
    memo
} from 'react';
import {
    useVirtualizer,
    type VirtualItem
} from '@tanstack/react-virtual';

/**
 * Constants
 */
const DEFAULT_OVERSCAN = 5;
const DEFAULT_ROW_HEIGHT = 40;
const DEFAULT_HEADER_HEIGHT = 48;

/**
 * Virtual list item render function type
 */
type RenderItemFunction<T> = (item: T, index: number, virtualItem: VirtualItem) => React.ReactNode;

/**
 * Virtual list props
 */
interface VirtualListProps<T> {
    /**
     * Array of items to render
     */
    items: Array<T>

    /**
     * Height of the list container in pixels
     */
    height: number

    /**
     * Estimated height of each item in pixels
     */
    item_height?: number

    /**
     * Number of items to render outside visible area (for smoother scrolling)
     */
    overscan?: number

    /**
     * Render function for each item
     */
    renderItem: RenderItemFunction<T>

    /**
     * Optional className for the container
     */
    className?: string

    /**
     * Optional key extractor for items
     */
    getItemKey?: (item: T, index: number) => string | number
}

/**
 * VirtualList - Efficiently renders large lists using virtualization
 */
function VirtualListInner<T>({
    items,
    height,
    item_height = DEFAULT_ROW_HEIGHT,
    overscan = DEFAULT_OVERSCAN,
    renderItem,
    className = ``,
    getItemKey,
}: VirtualListProps<T>): React.ReactElement {
    const parentRef = useRef<HTMLDivElement>(null);

    const virtualizer = useVirtualizer({
        count:            items.length,
        getScrollElement: () => parentRef.current,
        estimateSize:     () => item_height,
        overscan,
        getItemKey:       getItemKey
            ? (index) => getItemKey(items[index], index)
            : undefined,
    });

    const virtualItems = virtualizer.getVirtualItems();

    return (
        <div
            ref={parentRef}
            className={`overflow-auto ${ className }`}
            style={{
                height,
            }}
        >
            <div
                style={{
                    height:     `${ virtualizer.getTotalSize() }px`,
                    width:      `100%`,
                    position:   `relative`,
                }}
            >
                {virtualItems.map((virtualItem) => (
                    <div
                        key={virtualItem.key}
                        style={{
                            position:  `absolute`,
                            top:       0,
                            left:      0,
                            width:     `100%`,
                            height:    `${ virtualItem.size }px`,
                            transform: `translateY(${ virtualItem.start }px)`,
                        }}
                    >
                        {renderItem(items[virtualItem.index], virtualItem.index, virtualItem)}
                    </div>
                ))}
            </div>
        </div>
    );
}

export const VirtualList = memo(VirtualListInner) as typeof VirtualListInner;

/**
 * Virtual table column definition
 */
interface VirtualTableColumn<T> {
    id:        string
    header:    React.ReactNode
    width?:    number | string
    accessor?: keyof T | ((item: T) => React.ReactNode)
    render?:   (value: unknown, item: T, index: number) => React.ReactNode
}

/**
 * Virtual table props
 */
interface VirtualTableProps<T> {
    /**
     * Array of items to render
     */
    items: Array<T>

    /**
     * Column definitions
     */
    columns: Array<VirtualTableColumn<T>>

    /**
     * Height of the table container in pixels
     */
    height: number

    /**
     * Height of each row in pixels
     */
    row_height?: number

    /**
     * Height of the header row in pixels
     */
    header_height?: number

    /**
     * Number of rows to render outside visible area
     */
    overscan?: number

    /**
     * Key extractor for rows
     */
    getRowKey?: (item: T, index: number) => string | number

    /**
     * Optional className for the container
     */
    className?: string

    /**
     * Optional row click handler
     */
    onRowClick?: (item: T, index: number) => void
}

/**
 * VirtualTable - Efficiently renders large tables using virtualization
 */
function VirtualTableInner<T>({
    items,
    columns,
    height,
    row_height = DEFAULT_ROW_HEIGHT,
    header_height = DEFAULT_HEADER_HEIGHT,
    overscan = DEFAULT_OVERSCAN,
    getRowKey,
    className = ``,
    onRowClick,
}: VirtualTableProps<T>): React.ReactElement {
    const parentRef = useRef<HTMLDivElement>(null);

    const virtualizer = useVirtualizer({
        count:            items.length,
        getScrollElement: () => parentRef.current,
        estimateSize:     () => row_height,
        overscan,
        getItemKey:       getRowKey
            ? (index) => getRowKey(items[index], index)
            : undefined,
    });

    const virtualItems = virtualizer.getVirtualItems();

    const getCellValue = useMemo(() => (column: VirtualTableColumn<T>, item: T): unknown => {
        if (column.accessor) {
            if (typeof column.accessor === `function`) {
                return column.accessor(item);
            }
            return item[column.accessor];
        }
        return null;
    }, []);

    return (
        <div className={`border rounded-md ${ className }`}>
            {/* Table header */}
            <div
                className="flex border-b bg-muted/50"
                style={{
                    height: header_height,
                }}
            >
                {columns.map((column) => (
                    <div
                        key={column.id}
                        className="flex items-center px-4 font-medium text-muted-foreground"
                        style={{
                            width:    column.width ?? `auto`,
                            flexGrow: column.width ? 0 : 1,
                        }}
                    >
                        {column.header}
                    </div>
                ))}
            </div>

            {/* Virtualized rows */}
            <div
                ref={parentRef}
                className="overflow-auto"
                style={{
                    height: height - header_height,
                }}
            >
                <div
                    style={{
                        height:   `${ virtualizer.getTotalSize() }px`,
                        width:    `100%`,
                        position: `relative`,
                    }}
                >
                    {virtualItems.map((virtualItem) => {
                        const item = items[virtualItem.index];
                        return (
                            <div
                                key={virtualItem.key}
                                className={`flex border-b hover:bg-muted/50 ${ onRowClick ? `cursor-pointer` : `` }`}
                                style={{
                                    position:  `absolute`,
                                    top:       0,
                                    left:      0,
                                    width:     `100%`,
                                    height:    `${ virtualItem.size }px`,
                                    transform: `translateY(${ virtualItem.start }px)`,
                                }}
                                onClick={onRowClick ? () => onRowClick(item, virtualItem.index) : undefined}
                            >
                                {columns.map((column) => {
                                    const value = getCellValue(column, item);
                                    return (
                                        <div
                                            key={column.id}
                                            className="flex items-center px-4"
                                            style={{
                                                width:    column.width ?? `auto`,
                                                flexGrow: column.width ? 0 : 1,
                                            }}
                                        >
                                            {column.render
                                                ? column.render(value, item, virtualItem.index)
                                                : String(value ?? ``)}
                                        </div>
                                    );
                                })}
                            </div>
                        );
                    })}
                </div>
            </div>
        </div>
    );
}

export const VirtualTable = memo(VirtualTableInner) as typeof VirtualTableInner;

/**
 * useVirtualScroll - Hook for custom virtualization implementations
 */
interface UseVirtualScrollOptions {
    item_count:    number
    item_height:   number
    container_ref: React.RefObject<HTMLElement | null>
    overscan?:     number
}

interface UseVirtualScrollReturn {
    virtualItems:  Array<VirtualItem>
    totalHeight:   number
    scrollToIndex: (index: number) => void
}

export function useVirtualScroll({
    item_count,
    item_height,
    container_ref,
    overscan = DEFAULT_OVERSCAN,
}: UseVirtualScrollOptions): UseVirtualScrollReturn {
    const virtualizer = useVirtualizer({
        count:            item_count,
        getScrollElement: () => container_ref.current,
        estimateSize:     () => item_height,
        overscan,
    });

    return {
        virtualItems:  virtualizer.getVirtualItems(),
        totalHeight:   virtualizer.getTotalSize(),
        scrollToIndex: (index: number) => virtualizer.scrollToIndex(index),
    };
}
