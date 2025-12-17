/**
 * Mock router wrapper for testing components that use react-router
 */

import React from 'react';
import { MemoryRouter, Routes, Route } from 'react-router-dom';

interface TestRouterProps {
    children: React.ReactNode;
    initialEntries?: string[];
}

/**
 * Wrapper component that provides router context for testing
 */
export function TestRouter({ 
    children, 
    initialEntries = ['/'] 
}: TestRouterProps): React.ReactElement {
    return (
        <MemoryRouter initialEntries={initialEntries}>
            <Routes>
                <Route path="*" element={children} />
            </Routes>
        </MemoryRouter>
    );
}

/**
 * Create a test router with specific routes
 */
export function createTestRouter(routes: Array<{ path: string; element: React.ReactNode }>) {
    return function RouterWrapper({ 
        initialEntries = ['/'] 
    }: { initialEntries?: string[] }): React.ReactElement {
        return (
            <MemoryRouter initialEntries={initialEntries}>
                <Routes>
                    {routes.map(({ path, element }) => (
                        <Route key={path} path={path} element={element} />
                    ))}
                </Routes>
            </MemoryRouter>
        );
    };
}
