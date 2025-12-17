/**
 * Global Loading Indicator Component
 * Displays a loading overlay with spinner when app is in loading state
 */

import React from 'react';
import { Loader2 } from 'lucide-react';

interface LoadingIndicatorProps {
    loading: boolean;
    message?: string;
}

export function LoadingIndicator({ loading, message }: LoadingIndicatorProps): React.ReactElement | null {
    if (!loading) {
        return null;
    }

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-sm">
            <div className="flex flex-col items-center gap-4 rounded-lg bg-card p-8 shadow-lg border">
                <Loader2 className="h-8 w-8 animate-spin text-primary" />
                {message && (
                    <p className="text-sm text-muted-foreground">{message}</p>
                )}
            </div>
        </div>
    );
}

/**
 * Inline Loading Spinner
 * Small spinner for inline loading states
 */
export function InlineSpinner({ className = '' }: { className?: string }): React.ReactElement {
    return (
        <Loader2 className={`h-4 w-4 animate-spin ${className}`} />
    );
}

/**
 * Button Loading State
 * Spinner specifically sized for buttons
 */
export function ButtonSpinner(): React.ReactElement {
    return (
        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
    );
}

/**
 * Page Loading Skeleton
 * Skeleton loader for page content
 */
export function PageSkeleton(): React.ReactElement {
    return (
        <div className="space-y-6 animate-pulse">
            <div className="space-y-2">
                <div className="h-8 w-1/3 bg-muted rounded" />
                <div className="h-4 w-1/2 bg-muted rounded" />
            </div>
            <div className="space-y-3">
                <div className="h-4 w-full bg-muted rounded" />
                <div className="h-4 w-5/6 bg-muted rounded" />
                <div className="h-4 w-4/6 bg-muted rounded" />
            </div>
            <div className="grid grid-cols-3 gap-4">
                <div className="h-32 bg-muted rounded" />
                <div className="h-32 bg-muted rounded" />
                <div className="h-32 bg-muted rounded" />
            </div>
        </div>
    );
}

/**
 * Table Loading Skeleton
 * Skeleton for table loading state
 */
export function TableSkeleton({ rows = 5 }: { rows?: number }): React.ReactElement {
    return (
        <div className="space-y-2 animate-pulse">
            <div className="h-10 bg-muted rounded" />
            {Array.from({ length: rows }).map((_, i) => (
                <div key={i} className="h-16 bg-muted/50 rounded" />
            ))}
        </div>
    );
}
