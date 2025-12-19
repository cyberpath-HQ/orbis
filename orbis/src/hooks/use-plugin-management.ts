/**
 * Plugin management hooks and utilities
 */

import {
    useCallback,
    useEffect,
    useState
} from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
    listen, type UnlistenFn
} from '@tauri-apps/api/event';

/**
 * Plugin state enum
 */
export type PluginState = `Loaded` | `Running` | `Disabled` | `Error` | `Unloading`;

/**
 * Plugin information from the backend
 */
export interface PluginInfo {
    id:            string
    name:          string
    version:       string
    description:   string | null
    author?:       string
    license?:      string
    state:         PluginState
    loaded_at?:    string
    permissions?:  Array<string>
    routes_count?: number
    pages_count?:  number
}

/**
 * Plugin list response
 */
export interface PluginsResponse {
    plugins: Array<PluginInfo>
    count:   number
}

/**
 * Plugin operation result
 */
export interface PluginOperationResult {
    success: boolean
    message: string
    plugin?: PluginInfo
}

/**
 * Plugin change event (from file watcher)
 */
export interface PluginChangeEvent {
    kind:       `Added` | `Modified` | `Removed`
    path:       string
    plugin_id?: string
}

/**
 * Hook for managing plugins
 */
export interface UsePluginManagementReturn {
    plugins:         Array<PluginInfo>
    isLoading:       boolean
    error:           string | null
    refresh:         () => Promise<void>
    reloadPlugin:    (name: string) => Promise<PluginOperationResult>
    enablePlugin:    (name: string) => Promise<PluginOperationResult>
    disablePlugin:   (name: string) => Promise<PluginOperationResult>
    installPlugin:   (path: string) => Promise<PluginOperationResult>
    uninstallPlugin: (name: string) => Promise<PluginOperationResult>
    getPluginInfo:   (name: string) => Promise<PluginInfo | null>
}

/**
 * Hook for plugin management
 */
export function usePluginManagement(): UsePluginManagementReturn {
    const [
        plugins,
        setPlugins,
    ] = useState<Array<PluginInfo>>([]);
    const [
        is_loading,
        setIsLoading,
    ] = useState(true);
    const [
        error,
        setError,
    ] = useState<string | null>(null);

    // Fetch plugins from backend
    const refresh = useCallback(async(): Promise<void> => {
        setIsLoading(true);
        setError(null);

        try {
            const response = await invoke<PluginsResponse>(`get_plugins`);
            setPlugins(response.plugins);
        }
        catch (err) {
            const message = err instanceof Error ? err.message : String(err);
            setError(message);
            console.error(`Failed to fetch plugins:`, err);
        }
        finally {
            setIsLoading(false);
        }
    }, []);

    // Load plugins on mount
    useEffect(() => {
        void refresh();
    }, [ refresh ]);

    // Reload a specific plugin
    const reloadPlugin = useCallback(async(name: string): Promise<PluginOperationResult> => {
        try {
            const result = await invoke<PluginOperationResult>(`reload_plugin`, {
                name,
            });

            // Refresh the list after reload
            await refresh();
            return result;
        }
        catch (err) {
            const message = err instanceof Error ? err.message : String(err);
            return {
                success: false,
                message,
            };
        }
    }, [ refresh ]);

    // Enable a plugin
    const enablePlugin = useCallback(async(name: string): Promise<PluginOperationResult> => {
        try {
            const result = await invoke<PluginOperationResult>(`enable_plugin`, {
                name,
            });
            await refresh();
            return result;
        }
        catch (err) {
            const message = err instanceof Error ? err.message : String(err);
            return {
                success: false,
                message,
            };
        }
    }, [ refresh ]);

    // Disable a plugin
    const disablePlugin = useCallback(async(name: string): Promise<PluginOperationResult> => {
        try {
            const result = await invoke<PluginOperationResult>(`disable_plugin`, {
                name,
            });
            await refresh();
            return result;
        }
        catch (err) {
            const message = err instanceof Error ? err.message : String(err);
            return {
                success: false,
                message,
            };
        }
    }, [ refresh ]);

    // Install a plugin from path
    const installPlugin = useCallback(async(path: string): Promise<PluginOperationResult> => {
        try {
            const result = await invoke<PluginOperationResult>(`install_plugin`, {
                path,
            });
            await refresh();
            return result;
        }
        catch (err) {
            const message = err instanceof Error ? err.message : String(err);
            return {
                success: false,
                message,
            };
        }
    }, [ refresh ]);

    // Uninstall a plugin
    const uninstallPlugin = useCallback(async(name: string): Promise<PluginOperationResult> => {
        try {
            const result = await invoke<PluginOperationResult>(`uninstall_plugin`, {
                name,
            });
            await refresh();
            return result;
        }
        catch (err) {
            const message = err instanceof Error ? err.message : String(err);
            return {
                success: false,
                message,
            };
        }
    }, [ refresh ]);

    // Get detailed plugin info
    const getPluginInfo = useCallback(async(name: string): Promise<PluginInfo | null> => {
        try {
            return await invoke<PluginInfo>(`get_plugin_info`, {
                name,
            });
        }
        catch (err) {
            console.error(`Failed to get plugin info:`, err);
            return null;
        }
    }, []);

    return {
        plugins,
        isLoading: is_loading,
        error,
        refresh,
        reloadPlugin,
        enablePlugin,
        disablePlugin,
        installPlugin,
        uninstallPlugin,
        getPluginInfo,
    };
}

/**
 * Hook for listening to plugin change events
 */
export function usePluginWatcher(
    onPluginChange?: (event: PluginChangeEvent) => void
): { isWatching: boolean } {
    const [
        is_watching,
        setIsWatching,
    ] = useState(false);

    useEffect(() => {
        let unlisten: UnlistenFn | null = null;
        let debounceTimer: NodeJS.Timeout | null = null;

        const startListening = async(): Promise<void> => {
            try {
                unlisten = await listen<PluginChangeEvent>(`plugin-changed`, (event) => {
                    console.log(`Plugin changed:`, event.payload);
                    
                    // Debounce to prevent rapid-fire updates
                    if (debounceTimer) {
                        clearTimeout(debounceTimer);
                    }
                    
                    debounceTimer = setTimeout(() => {
                        onPluginChange?.(event.payload);
                        debounceTimer = null;
                    }, 300); // 300ms debounce
                });
                setIsWatching(true);
            }
            catch (err) {
                console.error(`Failed to start plugin watcher listener:`, err);
            }
        };

        void startListening();

        return (): void => {
            if (debounceTimer) {
                clearTimeout(debounceTimer);
            }
            if (unlisten) {
                unlisten();
            }
        };
    }, [ onPluginChange ]);

    return {
        isWatching: is_watching,
    };
}

/**
 * Plugin state badge color mapping
 */
export function getPluginStateColor(state: PluginState): string {
    switch (state) {
        case `Running`:
            return `bg-green-500`;
        case `Loaded`:
            return `bg-blue-500`;
        case `Disabled`:
            return `bg-gray-500`;
        case `Error`:
            return `bg-red-500`;
        case `Unloading`:
            return `bg-yellow-500`;
        default:
            return `bg-gray-400`;
    }
}

/**
 * Plugin state display text
 */
export function getPluginStateText(state: PluginState): string {
    switch (state) {
        case `Running`:
            return `Active`;
        case `Loaded`:
            return `Loaded`;
        case `Disabled`:
            return `Disabled`;
        case `Error`:
            return `Error`;
        case `Unloading`:
            return `Unloading`;
        default:
            return `Unknown`;
    }
}
