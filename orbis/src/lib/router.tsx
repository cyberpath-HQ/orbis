/**
 * Router configuration and route guards for the Orbis application
 */

import React, {
    createContext,
    useContext,
    useMemo,
    useEffect
} from 'react';
import {
    BrowserRouter,
    Routes,
    Route,
    Navigate,
    Outlet,
    useLocation
} from 'react-router-dom';
import { invoke } from '@tauri-apps/api/core';
import { toast } from 'sonner';
import type { PageDefinition } from '../types/schema';

// Auth context types
interface AuthState {
    isAuthenticated: boolean
    user:            UserInfo | null
    permissions:     Array<string>
    roles:           Array<string>
}

interface UserInfo {
    id:      string
    name:    string
    email:   string
    avatar?: string
}

interface AuthContextValue extends AuthState {
    login:             (email: string, password: string) => Promise<void>
    logout:            () => Promise<void>
    hasPermission:     (permission: string) => boolean
    hasAnyPermission:  (permissions: Array<string>) => boolean
    hasAllPermissions: (permissions: Array<string>) => boolean
    hasRole:           (role: string) => boolean
    hasAnyRole:        (roles: Array<string>) => boolean
}

const AuthContext = createContext<AuthContextValue | null>(null);

export function useAuth(): AuthContextValue {
    const context = useContext(AuthContext);
    if (!context) {
        throw new Error(`useAuth must be used within AuthProvider`);
    }
    return context;
}

interface AuthProviderProps {
    children: React.ReactNode
}

export function AuthProvider({
    children,
}: AuthProviderProps): React.ReactElement {
    const [
        state,
        setState,
    ] = React.useState<AuthState>({
        isAuthenticated: false,
        user:            null,
        permissions:     [],
        roles:           [],
    });

    // Check for existing session on mount
    useEffect(() => {
        const checkSession = async() => {
            try {
                const session = await invoke<{
                    user_id: string;
                    username: string;
                    email: string;
                    token: string;
                    refresh_token: string | null;
                    permissions: string[];
                    roles: string[];
                    is_admin: boolean;
                    created_at: string;
                    expires_at: string | null;
                } | null>('get_session');

                if (session) {
                    setState({
                        isAuthenticated: true,
                        user: {
                            id: session.user_id,
                            name: session.username,
                            email: session.email,
                        },
                        permissions: session.permissions,
                        roles: session.roles,
                    });
                }
            } catch (error) {
                console.error('Failed to check session:', error);
            }
        };

        checkSession();
    }, []);

    const value = useMemo<AuthContextValue>(() => ({
        ...state,

        login: async(email: string, password: string) => {
            try {
                const response = await invoke<{
                    success: boolean;
                    message: string;
                    session?: {
                        user_id: string;
                        username: string;
                        email: string;
                        token: string;
                        refresh_token: string | null;
                        permissions: string[];
                        roles: string[];
                        is_admin: boolean;
                        created_at: string;
                        expires_at: string | null;
                    };
                }>('login', { username: email, password });

                if (response.success && response.session) {
                    setState({
                        isAuthenticated: true,
                        user: {
                            id: response.session.user_id,
                            name: response.session.username,
                            email: response.session.email,
                        },
                        permissions: response.session.permissions,
                        roles: response.session.roles,
                    });
                    toast.success('Login successful');
                } else {
                    toast.error(response.message || 'Login failed');
                    throw new Error(response.message || 'Login failed');
                }
            } catch (error) {
                console.error('Login error:', error);
                toast.error('Login failed');
                throw error;
            }
        },

        logout: async() => {
            try {
                await invoke('logout');
                setState({
                    isAuthenticated: false,
                    user: null,
                    permissions: [],
                    roles: [],
                });
                toast.success('Logged out successfully');
            } catch (error) {
                console.error('Logout error:', error);
                toast.error('Logout failed');
            }
        },

        hasPermission: (permission: string) => state.permissions.includes(permission),

        hasAnyPermission: (permissions: Array<string>) => permissions.some((p) => state.permissions.includes(p)),

        hasAllPermissions: (permissions: Array<string>) => permissions.every((p) => state.permissions.includes(p)),

        hasRole: (role: string) => state.roles.includes(role),

        hasAnyRole: (roles: Array<string>) => roles.some((r) => state.roles.includes(r)),
    }), [ state ]);

    return (
        <AuthContext.Provider value={value}>
            {children}
        </AuthContext.Provider>
    );
}

// Route guard component
interface RouteGuardProps {
    requiresAuth?: boolean
    permissions?:  Array<string>
    roles?:        Array<string>
    redirectTo?:   string
    children?:     React.ReactNode
}

export function RouteGuard({
    requiresAuth = false,
    permissions = [],
    roles = [],
    redirectTo = `/login`,
    children,
}: RouteGuardProps): React.ReactElement {
    const auth = useAuth();
    const location = useLocation();

    // Check authentication
    if (requiresAuth && !auth.isAuthenticated) {
        return (
            <Navigate
                to={redirectTo}
                state={{
                    from: location,
                }}
                replace
            />
        );
    }

    // Check permissions
    if (permissions.length > 0 && !auth.hasAnyPermission(permissions)) {
        return (
            <Navigate
                to="/unauthorized"
                state={{
                    from: location,
                }}
                replace
            />
        );
    }

    // Check roles
    if (roles.length > 0 && !auth.hasAnyRole(roles)) {
        return (
            <Navigate
                to="/unauthorized"
                state={{
                    from: location,
                }}
                replace
            />
        );
    }

    return children ? <>{children}</> : <Outlet />;
}

// Page router component that generates routes from page definitions
interface PageRouterProps {
    pages:      Array<PageDefinition>
    renderPage: (page: PageDefinition) => React.ReactNode
    layout?:    React.ComponentType<{ children: React.ReactNode }>
}

export function PageRouter({
    pages,
    renderPage,
    layout: Layout,
}: PageRouterProps): React.ReactElement {
    // Group pages by parent route
    const rootPages = pages.filter((p) => !p.parentRoute);
    const childPages = pages.filter((p) => p.parentRoute);

    const renderRoutes = (pageList: Array<PageDefinition>): Array<React.ReactNode> => pageList.map((page) => {
        const children = childPages.filter((c) => c.parentRoute === page.route);

        return (
            <Route
                key={page.route}
                path={page.route}
                element={
                    <RouteGuard
                        requiresAuth={page.requiresAuth}
                        permissions={page.permissions}
                        roles={page.roles}
                    >
                        {renderPage(page)}
                    </RouteGuard>
                }
            >
                {children.length > 0 && renderRoutes(children)}
            </Route>
        );
    });

    const routes = renderRoutes(rootPages);

    if (Layout) {
        return (
            <Routes>
                <Route element={<Layout><Outlet /></Layout>}>
                    {routes}
                </Route>
            </Routes>
        );
    }

    return <Routes>{routes}</Routes>;
}

// App router wrapper
interface AppRouterProps {
    children: React.ReactNode
}

export function AppRouter({
    children,
}: AppRouterProps): React.ReactElement {
    return (
        <BrowserRouter>
            <AuthProvider>
                {children}
            </AuthProvider>
        </BrowserRouter>
    );
}
