/**
 * Dashboard page
 */

import React from 'react';
import * as LucideIcons from 'lucide-react';

import {
    Card, CardContent, CardDescription, CardHeader, CardTitle
} from '@/components/ui/card';
import type {
    AppModeInfo, PluginInfo
} from '@/types';

interface DashboardPageProps {
    mode:    AppModeInfo | null
    plugins: Array<PluginInfo>
}

export function DashboardPage({
    mode,
    plugins,
}: DashboardPageProps): React.ReactElement {
    const stats = [
        {
            title: `Mode`,
            value: mode?.mode ?? `Unknown`,
            icon:  LucideIcons.Monitor,
        },
        {
            title: `Plugins Loaded`,
            value: String(plugins.length),
            icon:  LucideIcons.Puzzle,
        },
        {
            title: `Active Plugins`,
            value: String(plugins.filter((p) => p.state === `Running`).length),
            icon:  LucideIcons.CheckCircle,
        },
        {
            title: `Status`,
            value: `Connected`,
            icon:  LucideIcons.Wifi,
        },
    ];

    return (
        <div className="space-y-6">
            <div>
                <h1 className="text-3xl font-bold tracking-tight">Dashboard</h1>
                <p className="text-muted-foreground">
                    Welcome to Orbis. Overview of your system status.
                </p>
            </div>

            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
                {stats.map((stat) => (
                    <Card key={stat.title}>
                        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                            <CardTitle className="text-sm font-medium">{stat.title}</CardTitle>
                            <stat.icon className="h-4 w-4 text-muted-foreground" />
                        </CardHeader>
                        <CardContent>
                            <div className="text-2xl font-bold">{stat.value}</div>
                        </CardContent>
                    </Card>
                ))}
            </div>

            {plugins.length > 0 && (
                <Card>
                    <CardHeader>
                        <CardTitle>Loaded Plugins</CardTitle>
                        <CardDescription>
                            Plugins currently available in your Orbis instance.
                        </CardDescription>
                    </CardHeader>
                    <CardContent>
                        <div className="space-y-4">
                            {plugins.map((plugin) => (
                                <div
                                    key={plugin.id}
                                    className="flex items-center justify-between p-4 rounded-lg border"
                                >
                                    <div className="space-y-1">
                                        <p className="font-medium">{plugin.name}</p>
                                        <p className="text-sm text-muted-foreground">
                                            {plugin.description}
                                        </p>
                                    </div>
                                    <div className="flex items-center gap-4">
                                        <span className="text-sm text-muted-foreground">
                                            v{plugin.version}
                                        </span>
                                        <span
                                            className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                                                plugin.state === `Running`
                                                    ? `bg-green-100 text-green-800`
                                                    : plugin.state === `Error`
                                                        ? `bg-red-100 text-red-800`
                                                        : `bg-gray-100 text-gray-800`
                                            }`}
                                        >
                                            {plugin.state}
                                        </span>
                                    </div>
                                </div>
                            ))}
                        </div>
                    </CardContent>
                </Card>
            )}

            {plugins.length === 0 && (
                <Card>
                    <CardContent className="flex flex-col items-center justify-center py-12">
                        <LucideIcons.Puzzle className="h-12 w-12 text-muted-foreground mb-4" />
                        <h3 className="text-lg font-medium">No Plugins Loaded</h3>
                        <p className="text-muted-foreground">
                            Install plugins to extend Orbis functionality.
                        </p>
                    </CardContent>
                </Card>
            )}
        </div>
    );
}

export default DashboardPage;
