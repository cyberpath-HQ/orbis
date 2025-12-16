/**
 * Plugins management page
 */

import React from 'react';
import * as LucideIcons from 'lucide-react';

import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle
} from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import type { PluginInfo } from '@/types';

interface PluginsPageProps {
    plugins: Array<PluginInfo>
}

export function PluginsPage({
    plugins,
}: PluginsPageProps): React.ReactElement {
    const getStateVariant = (state: PluginInfo[`state`]): `default` | `secondary` | `destructive` => {
        if (state === `Running`) {
            return `default`;
        }
        if (state === `Error`) {
            return `destructive`;
        }
        return `secondary`;
    };

    return (
        <div className="space-y-6">
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold tracking-tight">Plugins</h1>
                    <p className="text-muted-foreground">
                        Manage your installed plugins.
                    </p>
                </div>
                <Button>
                    <LucideIcons.Plus className="mr-2 h-4 w-4" />
                    Install Plugin
                </Button>
            </div>

            {plugins.length === 0
? (
                <Card>
                    <CardContent className="flex flex-col items-center justify-center py-12">
                        <LucideIcons.Puzzle className="h-12 w-12 text-muted-foreground mb-4" />
                        <h3 className="text-lg font-medium">No Plugins Installed</h3>
                        <p className="text-muted-foreground mb-4">
                            Get started by installing your first plugin.
                        </p>
                        <Button>
                            <LucideIcons.Plus className="mr-2 h-4 w-4" />
                            Browse Plugins
                        </Button>
                    </CardContent>
                </Card>
            )
: (
                <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                    {plugins.map((plugin) => (
                        <Card key={plugin.id}>
                            <CardHeader>
                                <div className="flex items-center justify-between">
                                    <CardTitle className="text-lg">{plugin.name}</CardTitle>
                                    <Badge variant={getStateVariant(plugin.state)}>
                                        {plugin.state}
                                    </Badge>
                                </div>
                                <CardDescription>{plugin.description}</CardDescription>
                            </CardHeader>
                            <CardContent>
                                <div className="flex items-center justify-between">
                                    <span className="text-sm text-muted-foreground">
                                        v{plugin.version}
                                    </span>
                                    <div className="flex gap-2">
                                        <Button variant="outline" size="sm">
                                            <LucideIcons.Settings className="h-4 w-4" />
                                        </Button>
                                        {plugin.state === `Running`
? (
                                            <Button variant="outline" size="sm">
                                                <LucideIcons.Pause className="h-4 w-4" />
                                            </Button>
                                        )
: (
                                            <Button variant="outline" size="sm">
                                                <LucideIcons.Play className="h-4 w-4" />
                                            </Button>
                                        )}
                                    </div>
                                </div>
                            </CardContent>
                        </Card>
                    ))}
                </div>
            )}
        </div>
    );
}

export default PluginsPage;
