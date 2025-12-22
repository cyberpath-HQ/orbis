/**
 * Main Orbis Application
 */

import {
    useState,
    useEffect,
    useMemo,
    useCallback
} from 'react';
import {
    Routes,
    Route,
    useNavigate
} from 'react-router-dom';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Toaster } from 'sonner';

import { AppLayout } from '@/lib/layout';
import { RouteGuard } from '@/lib/router';
import { SchemaRenderer } from '@/lib/renderer';
import { createPageStateStore } from '@/lib/state';
import { executeActions } from '@/lib/actions';
import {
    PluginErrorBoundary, PageErrorBoundary, LoadingIndicator
} from '@/components';
import { usePluginWatcher } from '@/hooks/use-plugin-management';
import type { ApiClient } from '@/lib/actions';
import type {
    PluginInfo, PluginPage, AppModeInfo
} from '@/types/plugin';
import type { NavigationConfig } from '@/types/schema';

// Core system pages
import {
    DashboardPage,
    PluginsPage,
    SettingsPage,
    LoginPage,
    NotFoundPage,
    UnauthorizedPage
} from '@/pages';

function App(): React.ReactElement {
    const [
        status,
        setStatus,
    ] = useState<`loading` | `connected` | `error`>(`loading`);
    const [
        mode,
        setMode,
    ] = useState<AppModeInfo | null>(null);
    const [
        plugins,
        setPlugins,
    ] = useState<Array<PluginInfo>>([]);
    const [
        pluginPages,
        setPluginPages,
    ] = useState<Array<PluginPage>>([]);
    const [
        error,
        setError,
    ] = useState<string | null>(null);

    // Refresh plugin pages
    const refreshPluginPages = useCallback(async() => {
        try {
            const {
                pages,
            } = await invoke<{ pages: Array<PluginPage> }>(`get_plugin_pages`);
            setPluginPages(pages);
        }
        catch (err) {
            console.error(`Failed to refresh plugin pages:`, err);
        }
    }, []);

    // Initialize application
    useEffect(() => {
        async function init(): Promise<void> {
            try {
                // Check health
                await invoke(`health_check`);
                setStatus(`connected`);

                // Get mode info
                const modeInfo = await invoke<AppModeInfo>(`get_mode`);
                setMode(modeInfo);

                // Get plugins
                const {
                    plugins: loadedPlugins,
                } = await invoke<{ plugins: Array<PluginInfo> }>(`get_plugins`);
                setPlugins(loadedPlugins);

                // Get plugin pages
                await refreshPluginPages();
            }
            catch (err) {
                setStatus(`error`);
                setError(err instanceof Error ? err.message : String(err));
            }
        }

        void init();
    }, [ refreshPluginPages ]);

    // Listen for plugin changes and refresh pages
    usePluginWatcher(
        useCallback(() => {
            // Refresh plugin pages when plugins change
            void refreshPluginPages();
        }, [ refreshPluginPages ])
    );

    // Listen for plugin state changes (enable/disable) and refresh pages
    useEffect(() => {
        let unlisten: (() => void) | undefined;

        const setupListener = async() => {
            unlisten = await listen<{ plugin: string
                state:                        string }>(`plugin-state-changed`, () => {
                // Refresh plugin pages when any plugin state changes
                void refreshPluginPages();
            });
        };

        void setupListener();

        return () => {
            if (unlisten) {
                unlisten();
            }
        };
    }, [ refreshPluginPages ]);

    // Navigation configuration
    const navigation = useMemo<NavigationConfig>(() => ({
        primary: [
            {
                id:    `dashboard`,
                label: `Dashboard`,
                icon:  `LayoutDashboard`,
                href:  `/`,
            },
            {
                id:    `plugins`,
                label: `Plugins`,
                icon:  `Puzzle`,
                href:  `/plugins`,
            },
        ],
        secondary: [
            {
                id:    `settings`,
                label: `Settings`,
                icon:  `Settings`,
                href:  `/settings`,
            },
        ],
    }), []);

    // Create a plugin-aware API client factory
    const createPluginApiClient = useCallback((pluginName?: string): ApiClient => ({
        call: async(api: string, method: string, args?: Record<string, unknown>) => {
            // Parse API path: "plugin.handler_name" or "core.command_name" or "plugin.plugin_name.handler_name"
            const parts = api.split(`.`);
            const [ namespace ] = parts;

            if (namespace === `plugin`) {
                // Determine the command format
                let command: string;

                if (parts.length === 2) {
                    // Format: "plugin.handler_name" - use current plugin context
                    if (!pluginName) {
                        throw new Error(`Plugin context required for API call: ${ api }`);
                    }
                    command = `${ pluginName }.${ parts[1] }`;
                }
                else if (parts.length === 3) {
                    // Format: "plugin.plugin_name.handler_name" - explicit plugin reference
                    command = `${ parts[1] }.${ parts[2] }`;
                }
                else {
                    throw new Error(
                        `Invalid plugin API format: ${ api }. ` +
                        `Expected "plugin.handler_name" or "plugin.plugin_name.handler_name"`
                    );
                }

                // Call plugin API with properly formatted command
                return invoke(`call_plugin_api`, {
                    command,
                    method,
                    args,
                });
            }

            // Call core API (format: "core.command_name")
            const command = parts.slice(1).join(`.`);
            return invoke(command, args);
        },
    }), []);

    // Loading state
    if (status === `loading`) {
        return <LoadingIndicator loading={true} message="Loading Orbis..." />;
    }

    // Error state
    if (status === `error`) {
        return (
            <div className="flex min-h-screen items-center justify-center">
                <div className="text-center space-y-4 p-8 max-w-md">
                    <div className="text-destructive text-6xl">⚠</div>
                    <h2 className="text-xl font-semibold">Failed to connect</h2>
                    <p className="text-muted-foreground">{error}</p>
                    <button
                        className="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90"
                        onClick={() => window.location.reload()}
                    >
                        Retry
                    </button>
                </div>
            </div>
        );
    }

    return (
        <AppLayout
            navigation={navigation}
            pluginPages={pluginPages}
            mode={mode ?? undefined}
        >
            <Routes>
                {/* Public routes - only show login in client-server mode */}
                {mode?.mode === `client` || mode?.mode === `server`
? (
                    <Route path="/login" element={<LoginPage />} />
                )
: null}
                <Route path="/unauthorized" element={<UnauthorizedPage />} />

                {/* Protected routes */}
                <Route element={<RouteGuard requiresAuth={false} />}>
                    <Route path="/" element={<DashboardPage mode={mode} plugins={plugins} />} />
                    <Route path="/plugins" element={<PluginsPage />} />
                    <Route path="/settings" element={<SettingsPage />} />
                    <Route path="/settings/*" element={<SettingsPage />} />
                </Route>

                {/* Plugin pages */}
                {pluginPages.map((page) => (
                    <Route
                        key={`${ page.plugin }-${ page.route }`}
                        path={`/plugins/${ page.plugin }${ page.route }`}
                        element={
                            <PluginPageRenderer
                                page={page}
                                apiClient={createPluginApiClient}
                            />
                        }
                    />
                ))}

                {/* Catch-all */}
                <Route path="*" element={<NotFoundPage />} />
            </Routes>
        </AppLayout>
    );
}

// Plugin page renderer component
interface PluginPageRendererProps {
    page:      PluginPage
    apiClient: (pluginName?: string) => ApiClient
}

function PluginPageRenderer({
    page,
    apiClient: createApiClient,
}: PluginPageRendererProps): React.ReactElement {
    const navigate = useNavigate();

    // Create plugin-specific API client that knows the plugin context
    const apiClient = useMemo(
        () => createApiClient(page.plugin),
        [
            createApiClient,
            page.plugin,
        ]
    );

    // Create page state store with persistence
    const stateStore = useMemo(() => {
        // Create persistence key from plugin name and page route
        const persistenceKey = `${ page.plugin }:${ page.route }`;
        
        // Use page state definition if present
        if (page.state) {
            return createPageStateStore(
                Object.fromEntries(
                    Object.entries(page.state).map(([
                        key,
                        def,
                    ]) => [
                        key,
                        {
                            type:    def.type,
                            default: def.default,
                        },
                    ])
                ),
                persistenceKey // Enable persistence with plugin:route key
            );
        }
        return createPageStateStore(undefined, persistenceKey);
    }, [ page ]);

    // Note: stateStore is a Zustand hook - we pass it directly, not call it
    // Components will call it internally to subscribe to changes

    // Execute onMount hook when page mounts
    useEffect(() => {
        if (page.hooks?.onMount) {
            const actionContext = {
                state: stateStore,
                apiClient,
                navigate,
            };
            executeActions(page.hooks.onMount, actionContext).catch((error) => {
                console.error(`Error executing onMount hook:`, error);
            });
        }
    }, [ page, stateStore, apiClient, navigate ]);

    // Execute onUnmount hook when page unmounts
    useEffect(() => () => {
        if (page.hooks?.onUnmount) {
            const actionContext = {
                state: stateStore,
                apiClient,
                navigate,
            };
            executeActions(page.hooks.onUnmount, actionContext).catch((error) => {
                console.error(`Error executing onUnmount hook:`, error);
            });
        }
    // eslint-disable-next-line react-hooks/exhaustive-deps
    }, []);

    return (
        <PluginErrorBoundary pluginId={page.plugin}>
            <div className="space-y-6">
                <div className="flex items-center justify-between">
                    <div>
                        <h1 className="text-3xl font-bold tracking-tight">{page.title}</h1>
                        {page.description && (
                            <p className="text-muted-foreground">{page.description}</p>
                        )}
                    </div>
                </div>

                {page.sections.map((section, index) => (
                    <PageErrorBoundary key={index}>
                        <SchemaRenderer
                            schema={section}
                            state={stateStore}
                            apiClient={apiClient}
                        />
                    </PageErrorBoundary>
                ))}
            </div>
        </PluginErrorBoundary>
    );
}

export default App;

/**
 * Toaster Component for Global Toast Notifications
 * Positioned at top-right for non-intrusive notifications
 */
export function AppToaster(): React.ReactElement {
    return <Toaster position="top-right" richColors closeButton />;
}
