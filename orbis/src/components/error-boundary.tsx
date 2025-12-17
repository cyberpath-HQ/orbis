/**
 * Error Boundary Components
 * Provides error handling and recovery for different parts of the application
 */

import React, { Component, type ReactNode } from 'react';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { AlertCircle, RefreshCw, Home } from 'lucide-react';

interface ErrorBoundaryProps {
    children: ReactNode;
    fallback?: ReactNode;
    onError?: (error: Error, errorInfo: React.ErrorInfo) => void;
    resetKeys?: unknown[];
    context?: string;
}

interface ErrorBoundaryState {
    hasError: boolean;
    error: Error | null;
    errorInfo: React.ErrorInfo | null;
}

/**
 * Generic Error Boundary
 * Catches React errors and displays a fallback UI
 */
export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
    constructor(props: ErrorBoundaryProps) {
        super(props);
        this.state = {
            hasError: false,
            error: null,
            errorInfo: null,
        };
    }

    static getDerivedStateFromError(error: Error): Partial<ErrorBoundaryState> {
        return { hasError: true, error };
    }

    componentDidCatch(error: Error, errorInfo: React.ErrorInfo): void {
        // Log error to console in development
        if (import.meta.env.DEV) {
            console.error('ErrorBoundary caught an error:', error, errorInfo);
        }

        // Call optional error handler
        this.props.onError?.(error, errorInfo);

        this.setState({ errorInfo });
    }

    componentDidUpdate(prevProps: ErrorBoundaryProps): void {
        // Reset error state when resetKeys change
        if (this.state.hasError && this.props.resetKeys) {
            const prevKeys = prevProps.resetKeys || [];
            const currentKeys = this.props.resetKeys;

            if (prevKeys.length !== currentKeys.length ||
                prevKeys.some((key, index) => key !== currentKeys[index])) {
                this.reset();
            }
        }
    }

    reset = (): void => {
        this.setState({
            hasError: false,
            error: null,
            errorInfo: null,
        });
    };

    render(): ReactNode {
        if (this.state.hasError) {
            if (this.props.fallback) {
                return this.props.fallback;
            }

            return (
                <DefaultErrorFallback
                    error={this.state.error}
                    errorInfo={this.state.errorInfo}
                    reset={this.reset}
                    context={this.props.context}
                />
            );
        }

        return this.props.children;
    }
}

interface ErrorFallbackProps {
    error: Error | null;
    errorInfo: React.ErrorInfo | null;
    reset: () => void;
    context?: string;
}

/**
 * Default Error Fallback UI
 */
function DefaultErrorFallback({ error, errorInfo, reset, context }: ErrorFallbackProps): React.ReactElement {
    const showDetails = import.meta.env.DEV;

    return (
        <div className="flex items-center justify-center min-h-[400px] p-4">
            <Card className="max-w-2xl w-full p-6">
                <Alert variant="destructive">
                    <AlertCircle className="h-4 w-4" />
                    <AlertTitle>
                        {context ? `Error in ${context}` : 'Something went wrong'}
                    </AlertTitle>
                    <AlertDescription>
                        {error?.message || 'An unexpected error occurred'}
                    </AlertDescription>
                </Alert>

                {showDetails && error && (
                    <div className="mt-4 space-y-2">
                        <details className="text-sm">
                            <summary className="cursor-pointer font-semibold text-muted-foreground">
                                Error Details
                            </summary>
                            <pre className="mt-2 p-4 bg-muted rounded-md overflow-auto text-xs">
                                {error.stack}
                            </pre>
                        </details>

                        {errorInfo && (
                            <details className="text-sm">
                                <summary className="cursor-pointer font-semibold text-muted-foreground">
                                    Component Stack
                                </summary>
                                <pre className="mt-2 p-4 bg-muted rounded-md overflow-auto text-xs">
                                    {errorInfo.componentStack}
                                </pre>
                            </details>
                        )}
                    </div>
                )}

                <div className="flex gap-2 mt-4">
                    <Button onClick={reset} variant="default">
                        <RefreshCw className="mr-2 h-4 w-4" />
                        Try Again
                    </Button>
                    <Button onClick={() => window.location.href = '/'} variant="outline">
                        <Home className="mr-2 h-4 w-4" />
                        Go Home
                    </Button>
                </div>
            </Card>
        </div>
    );
}

/**
 * Compact Error Fallback for Component-Level Errors
 */
export function CompactErrorFallback({ error, reset, context }: ErrorFallbackProps): React.ReactElement {
    return (
        <Alert variant="destructive" className="my-2">
            <AlertCircle className="h-4 w-4" />
            <AlertTitle className="text-sm">
                {context ? `Error in ${context}` : 'Component Error'}
            </AlertTitle>
            <AlertDescription className="text-xs">
                {error?.message || 'Failed to render component'}
                <Button onClick={reset} variant="ghost" size="sm" className="ml-2">
                    <RefreshCw className="h-3 w-3 mr-1" />
                    Retry
                </Button>
            </AlertDescription>
        </Alert>
    );
}

/**
 * Component Error Boundary
 * Used for individual components in the schema renderer
 */
export function ComponentErrorBoundary({ children }: { children: ReactNode }): React.ReactElement {
    return (
        <ErrorBoundary
            context="Component"
            fallback={
                <Alert variant="destructive" className="my-2">
                    <AlertCircle className="h-4 w-4" />
                    <AlertDescription className="text-sm">
                        Failed to render component
                    </AlertDescription>
                </Alert>
            }
        >
            {children}
        </ErrorBoundary>
    );
}

/**
 * Page Error Boundary
 * Used for entire page rendering
 */
export function PageErrorBoundary({ children }: { children: ReactNode }): React.ReactElement {
    return (
        <ErrorBoundary context="Page">
            {children}
        </ErrorBoundary>
    );
}

/**
 * Plugin Error Boundary
 * Used for plugin pages with plugin-specific error handling
 */
export function PluginErrorBoundary({
    children,
    pluginId,
}: {
    children: ReactNode;
    pluginId?: string;
}): React.ReactElement {
    const handleError = (error: Error, errorInfo: React.ErrorInfo): void => {
        // Log to console in development
        if (import.meta.env.DEV) {
            console.error(`Plugin ${pluginId || 'unknown'} error:`, error, errorInfo);
        }

        // In production, could send to error tracking service
        // Example: sendToErrorTracking({ pluginId, error, errorInfo });
    };

    return (
        <ErrorBoundary
            context={pluginId ? `Plugin: ${pluginId}` : 'Plugin'}
            onError={handleError}
        >
            {children}
        </ErrorBoundary>
    );
}
