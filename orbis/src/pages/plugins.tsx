/**
 * Plugins management page - Full featured management dashboard
 */

import React, {
    useState,
    useCallback
} from 'react';
import * as LucideIcons from 'lucide-react';
import { toast } from 'sonner';

import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
    CardFooter
} from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
    DialogTrigger
} from '@/components/ui/dialog';
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuSeparator,
    DropdownMenuTrigger
} from '@/components/ui/dropdown-menu';
import {
    AlertDialog,
    AlertDialogAction,
    AlertDialogCancel,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle
} from '@/components/ui/alert-dialog';
import { Skeleton } from '@/components/ui/skeleton';
import {
    Tooltip,
    TooltipContent,
    TooltipProvider,
    TooltipTrigger
} from '@/components/ui/tooltip';

import {
    usePluginManagement,
    usePluginWatcher,
    type PluginInfo,
    type PluginState,
    getPluginStateColor,
    getPluginStateText
} from '@/hooks/use-plugin-management';

/**
 * Plugin card component
 */
function PluginCard({
    plugin,
    onReload,
    onEnable,
    onDisable,
    onUninstall,
    onViewDetails,
    is_operating,
}: {
    plugin:        PluginInfo
    onReload:      () => void
    onEnable:      () => void
    onDisable:     () => void
    onUninstall:   () => void
    onViewDetails: () => void
    is_operating:  boolean
}): React.ReactElement {
    const getStateVariant = (state: PluginState): `default` | `secondary` | `destructive` | `outline` => {
        switch (state) {
            case `Running`:
                return `default`;
            case `Loaded`:
                return `secondary`;
            case `Error`:
                return `destructive`;
            case `Disabled`:
                return `outline`;
            default:
                return `secondary`;
        }
    };

    const is_running = plugin.state === `Running`;
    const is_disabled = plugin.state === `Disabled`;
    const has_error = plugin.state === `Error`;

    return (
        <Card className={`relative ${ has_error ? `border-destructive` : `` }`}>
            {/* Status indicator */}
            <div
                className={`absolute top-3 right-3 h-2 w-2 rounded-full ${ getPluginStateColor(plugin.state) }`}
                title={getPluginStateText(plugin.state)}
            />

            <CardHeader className="pb-3">
                <div className="flex items-start justify-between">
                    <div className="space-y-1">
                        <CardTitle className="text-lg font-semibold">
                            {plugin.name}
                        </CardTitle>
                        <div className="flex items-center gap-2">
                            <Badge
                                variant={getStateVariant(plugin.state)}
                                className="text-xs"
                            >
                                {getPluginStateText(plugin.state)}
                            </Badge>
                            <span className="text-xs text-muted-foreground">
                                v{plugin.version}
                            </span>
                        </div>
                    </div>
                </div>
                <CardDescription className="text-sm mt-2">
                    {plugin.description || `No description available`}
                </CardDescription>
            </CardHeader>

            <CardContent className="pb-3">
                <div className="space-y-2">
                    {plugin.author && (
                        <div className="flex items-center gap-2 text-xs text-muted-foreground">
                            <LucideIcons.User className="h-3 w-3" />
                            <span>{plugin.author}</span>
                        </div>
                    )}
                    {plugin.license && (
                        <div className="flex items-center gap-2 text-xs text-muted-foreground">
                            <LucideIcons.Scale className="h-3 w-3" />
                            <span>{plugin.license}</span>
                        </div>
                    )}
                </div>
            </CardContent>

            <CardFooter className="flex justify-between items-center pt-3 border-t">
                <div className="flex gap-1">
                    <TooltipProvider>
                        {/* Enable/Disable toggle */}
                        {is_running && (
                            <Tooltip>
                                <TooltipTrigger asChild>
                                    <Button
                                        variant="ghost"
                                        size="sm"
                                        onClick={onDisable}
                                        disabled={is_operating}
                                        aria-label="Disable plugin"
                                    >
                                        <LucideIcons.Pause className="h-4 w-4" />
                                    </Button>
                                </TooltipTrigger>
                                <TooltipContent>Disable Plugin</TooltipContent>
                            </Tooltip>
                        )}
                        {is_disabled && (
                            <Tooltip>
                                <TooltipTrigger asChild>
                                    <Button
                                        variant="ghost"
                                        size="sm"
                                        onClick={onEnable}
                                        disabled={is_operating}
                                        aria-label="Enable plugin"
                                    >
                                        <LucideIcons.Play className="h-4 w-4" />
                                    </Button>
                                </TooltipTrigger>
                                <TooltipContent>Enable Plugin</TooltipContent>
                            </Tooltip>
                        )}

                        {/* Hot Reload */}
                        <Tooltip>
                            <TooltipTrigger asChild>
                                <Button
                                    variant="ghost"
                                    size="sm"
                                    onClick={onReload}
                                    disabled={is_operating}
                                    aria-label="Reload plugin"
                                >
                                    <LucideIcons.RefreshCw className={`h-4 w-4 ${ is_operating ? `animate-spin` : `` }`} />
                                </Button>
                            </TooltipTrigger>
                            <TooltipContent>Hot Reload</TooltipContent>
                        </Tooltip>

                        {/* View Details */}
                        <Tooltip>
                            <TooltipTrigger asChild>
                                <Button
                                    variant="ghost"
                                    size="sm"
                                    onClick={onViewDetails}
                                    aria-label="View plugin details"
                                >
                                    <LucideIcons.Info className="h-4 w-4" />
                                </Button>
                            </TooltipTrigger>
                            <TooltipContent>View Details</TooltipContent>
                        </Tooltip>
                    </TooltipProvider>
                </div>

                {/* Actions menu */}
                <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                        <Button variant="ghost" size="sm">
                            <LucideIcons.MoreVertical className="h-4 w-4" />
                        </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end">
                        <DropdownMenuItem onClick={onReload}>
                            <LucideIcons.RefreshCw className="mr-2 h-4 w-4" />
                            Reload
                        </DropdownMenuItem>
                        <DropdownMenuItem onClick={onViewDetails}>
                            <LucideIcons.Info className="mr-2 h-4 w-4" />
                            Details
                        </DropdownMenuItem>
                        <DropdownMenuSeparator />
                        {is_running
? (
                            <DropdownMenuItem onClick={onDisable}>
                                <LucideIcons.Pause className="mr-2 h-4 w-4" />
                                Disable
                            </DropdownMenuItem>
                        )
: (
                            <DropdownMenuItem onClick={onEnable}>
                                <LucideIcons.Play className="mr-2 h-4 w-4" />
                                Enable
                            </DropdownMenuItem>
                        )}
                        <DropdownMenuSeparator />
                        <DropdownMenuItem
                            onClick={onUninstall}
                            className="text-destructive focus:text-destructive"
                        >
                            <LucideIcons.Trash2 className="mr-2 h-4 w-4" />
                            Uninstall
                        </DropdownMenuItem>
                    </DropdownMenuContent>
                </DropdownMenu>
            </CardFooter>
        </Card>
    );
}

/**
 * Plugin details dialog
 */
function PluginDetailsDialog({
    plugin,
    is_open,
    onClose,
}: {
    plugin:  PluginInfo | null
    is_open: boolean
    onClose: () => void
}): React.ReactElement | null {
    if (!plugin) {
        return null;
    }

    return (
        <Dialog
            open={is_open}
            onOpenChange={(open) => {
                if (!open) {
                    onClose();
                }
            }}
        >
            <DialogContent className="sm:max-w-md">
                <DialogHeader>
                    <DialogTitle className="flex items-center gap-2">
                        <LucideIcons.Puzzle className="h-5 w-5" />
                        {plugin.name}
                    </DialogTitle>
                    <DialogDescription>
                        {plugin.description || `No description available`}
                    </DialogDescription>
                </DialogHeader>
                <div className="space-y-4 py-4">
                    <div className="grid grid-cols-2 gap-4 text-sm">
                        <div>
                            <Label className="text-muted-foreground">Version</Label>
                            <p className="font-medium">{plugin.version}</p>
                        </div>
                        <div>
                            <Label className="text-muted-foreground">Status</Label>
                            <p className="font-medium">{getPluginStateText(plugin.state)}</p>
                        </div>
                        {plugin.author && (
                            <div>
                                <Label className="text-muted-foreground">Author</Label>
                                <p className="font-medium">{plugin.author}</p>
                            </div>
                        )}
                        {plugin.license && (
                            <div>
                                <Label className="text-muted-foreground">License</Label>
                                <p className="font-medium">{plugin.license}</p>
                            </div>
                        )}
                        {plugin.routes_count !== undefined && (
                            <div>
                                <Label className="text-muted-foreground">API Routes</Label>
                                <p className="font-medium">{plugin.routes_count}</p>
                            </div>
                        )}
                        {plugin.pages_count !== undefined && (
                            <div>
                                <Label className="text-muted-foreground">UI Pages</Label>
                                <p className="font-medium">{plugin.pages_count}</p>
                            </div>
                        )}
                    </div>

                    {plugin.permissions && plugin.permissions.length > 0 && (
                        <div>
                            <Label className="text-muted-foreground">Permissions</Label>
                            <div className="flex flex-wrap gap-1 mt-1">
                                {plugin.permissions.map((perm) => (
                                    <Badge key={perm} variant="outline" className="text-xs">
                                        {perm}
                                    </Badge>
                                ))}
                            </div>
                        </div>
                    )}

                    {plugin.loaded_at && (
                        <div>
                            <Label className="text-muted-foreground">Loaded At</Label>
                            <p className="text-sm">{new Date(plugin.loaded_at).toLocaleString()}</p>
                        </div>
                    )}
                </div>
                <DialogFooter>
                    <Button variant="outline" onClick={onClose}>
                        Close
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}

/**
 * Install plugin dialog
 */
function InstallPluginDialog({
    onInstall,
}: {
    onInstall: (path: string) => Promise<void>
}): React.ReactElement {
    const [
        is_open,
        setIsOpen,
    ] = useState(false);
    const [
        path,
        setPath,
    ] = useState(``);
    const [
        is_installing,
        setIsInstalling,
    ] = useState(false);

    const handleInstall = async(): Promise<void> => {
        if (!path.trim()) {
            toast.error(`Please enter a plugin path`);
            return;
        }

        setIsInstalling(true);
        try {
            await onInstall(path.trim());
            setPath(``);
            setIsOpen(false);
        }
        finally {
            setIsInstalling(false);
        }
    };

    return (
        <Dialog open={is_open} onOpenChange={setIsOpen}>
            <DialogTrigger asChild>
                <Button>
                    <LucideIcons.Plus className="mr-2 h-4 w-4" />
                    Install Plugin
                </Button>
            </DialogTrigger>
            <DialogContent className="sm:max-w-md">
                <DialogHeader>
                    <DialogTitle>Install Plugin</DialogTitle>
                    <DialogDescription>
                        Enter the path to a plugin file (.wasm or .zip) or directory.
                    </DialogDescription>
                </DialogHeader>
                <div className="space-y-4 py-4">
                    <div className="space-y-2">
                        <Label htmlFor="plugin-path">Plugin Path</Label>
                        <Input
                            id="plugin-path"
                            placeholder="/path/to/plugin.wasm"
                            value={path}
                            onChange={(e) => setPath(e.target.value)}
                            disabled={is_installing}
                        />
                        <p className="text-xs text-muted-foreground">
                            Supports .wasm, .zip files or plugin directories
                        </p>
                    </div>
                </div>
                <DialogFooter>
                    <Button
                        variant="outline"
                        onClick={() => setIsOpen(false)}
                        disabled={is_installing}
                    >
                        Cancel
                    </Button>
                    <Button onClick={handleInstall} disabled={is_installing}>
                        {is_installing
? (
                            <>
                                <LucideIcons.Loader2 className="mr-2 h-4 w-4 animate-spin" />
                                Installing...
                            </>
                        )
: (
                            <>
                                <LucideIcons.Download className="mr-2 h-4 w-4" />
                                Install
                            </>
                        )}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}

/**
 * Loading skeleton for plugin cards
 */
function PluginCardSkeleton(): React.ReactElement {
    return (
        <Card>
            <CardHeader className="pb-3">
                <div className="space-y-2">
                    <Skeleton className="h-5 w-32" />
                    <div className="flex gap-2">
                        <Skeleton className="h-4 w-16" />
                        <Skeleton className="h-4 w-12" />
                    </div>
                </div>
                <Skeleton className="h-4 w-full mt-2" />
            </CardHeader>
            <CardContent className="pb-3">
                <Skeleton className="h-3 w-24" />
            </CardContent>
            <CardFooter className="pt-3 border-t">
                <div className="flex gap-1">
                    <Skeleton className="h-8 w-8" />
                    <Skeleton className="h-8 w-8" />
                    <Skeleton className="h-8 w-8" />
                </div>
            </CardFooter>
        </Card>
    );
}

/**
 * Main plugins page component
 */
export function PluginsPage(): React.ReactElement {
    const {
        plugins,
        isLoading,
        error,
        refresh,
        reloadPlugin,
        enablePlugin,
        disablePlugin,
        installPlugin,
        uninstallPlugin,
        getPluginInfo,
    } = usePluginManagement();

    const [
        operating_plugin,
        setOperatingPlugin,
    ] = useState<string | null>(null);
    const [
        selected_plugin,
        setSelectedPlugin,
    ] = useState<PluginInfo | null>(null);
    const [
        is_details_open,
        setIsDetailsOpen,
    ] = useState(false);
    const [
        uninstall_confirm,
        setUninstallConfirm,
    ] = useState<PluginInfo | null>(null);

    // Listen for plugin changes for auto-refresh
    usePluginWatcher(
        useCallback(() => {
            void refresh();
            toast.info(`Plugin changes detected`, {
                description: `Plugin list has been refreshed`,
            });
        }, [ refresh ])
    );

    // Plugin operation handlers
    const handleReload = useCallback(async(name: string): Promise<void> => {
        setOperatingPlugin(name);
        const result = await reloadPlugin(name);
        setOperatingPlugin(null);

        if (result.success) {
            toast.success(`Plugin reloaded`, {
                description: result.message,
            });
        }
        else {
            toast.error(`Reload failed`, {
                description: result.message,
            });
        }
    }, [ reloadPlugin ]);

    const handleEnable = useCallback(async(name: string): Promise<void> => {
        setOperatingPlugin(name);
        const result = await enablePlugin(name);
        setOperatingPlugin(null);

        if (result.success) {
            toast.success(`Plugin enabled`, {
                description: result.message,
            });
        }
        else {
            toast.error(`Enable failed`, {
                description: result.message,
            });
        }
    }, [ enablePlugin ]);

    const handleDisable = useCallback(async(name: string): Promise<void> => {
        setOperatingPlugin(name);
        const result = await disablePlugin(name);
        setOperatingPlugin(null);

        if (result.success) {
            toast.success(`Plugin disabled`, {
                description: result.message,
            });
        }
        else {
            toast.error(`Disable failed`, {
                description: result.message,
            });
        }
    }, [ disablePlugin ]);

    const handleInstall = useCallback(async(path: string): Promise<void> => {
        const result = await installPlugin(path);

        if (result.success) {
            toast.success(`Plugin installed`, {
                description: result.message,
            });
        }
        else {
            toast.error(`Install failed`, {
                description: result.message,
            });
        }
    }, [ installPlugin ]);

    const handleUninstall = useCallback(async(name: string): Promise<void> => {
        setOperatingPlugin(name);
        const result = await uninstallPlugin(name);
        setOperatingPlugin(null);
        setUninstallConfirm(null);

        if (result.success) {
            toast.success(`Plugin uninstalled`, {
                description: result.message,
            });
        }
        else {
            toast.error(`Uninstall failed`, {
                description: result.message,
            });
        }
    }, [ uninstallPlugin ]);

    const handleViewDetails = useCallback(async(name: string): Promise<void> => {
        const details = await getPluginInfo(name);
        if (details) {
            setSelectedPlugin(details);
            setIsDetailsOpen(true);
        }
    }, [ getPluginInfo ]);

    // Stats
    const running_count = plugins.filter((p) => p.state === `Running`).length;
    const total_count = plugins.length;

    return (
        <div className="space-y-6">
            {/* Header */}
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold tracking-tight">Plugins</h1>
                    <p className="text-muted-foreground">
                        Manage your installed plugins. {running_count} of {total_count} plugins active.
                    </p>
                </div>
                <div className="flex gap-2">
                    <Button variant="outline" onClick={refresh} disabled={isLoading}>
                        <LucideIcons.RefreshCw className={`mr-2 h-4 w-4 ${ isLoading ? `animate-spin` : `` }`} />
                        Refresh
                    </Button>
                    <InstallPluginDialog onInstall={handleInstall} />
                </div>
            </div>

            {/* Error state */}
            {error && (
                <Card className="border-destructive">
                    <CardContent className="flex items-center gap-3 py-4">
                        <LucideIcons.AlertCircle className="h-5 w-5 text-destructive" />
                        <div>
                            <p className="font-medium text-destructive">Failed to load plugins</p>
                            <p className="text-sm text-muted-foreground">{error}</p>
                        </div>
                        <Button variant="outline" size="sm" className="ml-auto" onClick={refresh}>
                            Retry
                        </Button>
                    </CardContent>
                </Card>
            )}

            {/* Loading state */}
            {isLoading && plugins.length === 0 && (
                <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                    <PluginCardSkeleton />
                    <PluginCardSkeleton />
                    <PluginCardSkeleton />
                </div>
            )}

            {/* Empty state */}
            {!isLoading && plugins.length === 0 && !error && (
                <Card>
                    <CardContent className="flex flex-col items-center justify-center py-12">
                        <LucideIcons.Puzzle className="h-12 w-12 text-muted-foreground mb-4" />
                        <h3 className="text-lg font-medium">No Plugins Installed</h3>
                        <p className="text-muted-foreground mb-4 text-center max-w-sm">
                            Get started by installing your first plugin. Plugins extend Orbis
                            with custom pages, API routes, and features.
                        </p>
                        <InstallPluginDialog onInstall={handleInstall} />
                    </CardContent>
                </Card>
            )}

            {/* Plugin grid */}
            {plugins.length > 0 && (
                <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                    {plugins.map((plugin) => (
                        <PluginCard
                            key={plugin.id}
                            plugin={plugin}
                            onReload={async() => handleReload(plugin.name)}
                            onEnable={async() => handleEnable(plugin.name)}
                            onDisable={async() => handleDisable(plugin.name)}
                            onUninstall={() => setUninstallConfirm(plugin)}
                            onViewDetails={async() => handleViewDetails(plugin.name)}
                            is_operating={operating_plugin === plugin.name}
                        />
                    ))}
                </div>
            )}

            {/* Plugin details dialog */}
            <PluginDetailsDialog
                plugin={selected_plugin}
                is_open={is_details_open}
                onClose={() => {
                    setIsDetailsOpen(false);
                    setSelectedPlugin(null);
                }}
            />

            {/* Uninstall confirmation dialog */}
            <AlertDialog
                open={uninstall_confirm !== null}
                onOpenChange={(open) => {
                    if (!open) {
                        setUninstallConfirm(null);
                    }
                }}
            >
                <AlertDialogContent>
                    <AlertDialogHeader>
                        <AlertDialogTitle>Uninstall Plugin</AlertDialogTitle>
                        <AlertDialogDescription>
                            Are you sure you want to uninstall &quot;{uninstall_confirm?.name}&quot;?
                            This action cannot be undone.
                        </AlertDialogDescription>
                    </AlertDialogHeader>
                    <AlertDialogFooter>
                        <AlertDialogCancel>Cancel</AlertDialogCancel>
                        <AlertDialogAction
                            onClick={() => {
                                if (uninstall_confirm) {
                                    void handleUninstall(uninstall_confirm.name);
                                }
                            }}
                            className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
                        >
                            Uninstall
                        </AlertDialogAction>
                    </AlertDialogFooter>
                </AlertDialogContent>
            </AlertDialog>
        </div>
    );
}

export default PluginsPage;
