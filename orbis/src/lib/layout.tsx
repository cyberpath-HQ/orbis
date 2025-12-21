/**
 * Main application layout with sidebar navigation
 */

import React from 'react';
import {
    Link, useLocation, useNavigate
} from 'react-router-dom';
import * as LucideIcons from 'lucide-react';
import { Toaster } from 'sonner';

import {
    Sidebar,
    SidebarContent,
    SidebarFooter,
    SidebarGroup,
    SidebarGroupContent,
    SidebarGroupLabel,
    SidebarHeader,
    SidebarInset,
    SidebarMenu,
    SidebarMenuButton,
    SidebarMenuItem,
    SidebarProvider,
    SidebarTrigger
} from '@/components/ui/sidebar';
import { Separator } from '@/components/ui/separator';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Button } from '@/components/ui/button';
import {
    Avatar, AvatarFallback, AvatarImage
} from '@/components/ui/avatar';
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuSeparator,
    DropdownMenuTrigger
} from '@/components/ui/dropdown-menu';
import { Badge } from '@/components/ui/badge';
import { SkipLink } from '@/components';

import { useAuth } from './router';
import type {
    NavigationItem, NavigationConfig
} from '../types/schema';
import type {
    PluginPage, AppModeInfo
} from '../types';

interface AppLayoutProps {
    children:     React.ReactNode
    navigation?:  NavigationConfig
    pluginPages?: Array<PluginPage>
    mode?:        AppModeInfo
    appName?:     string
}

export function AppLayout({
    children,
    navigation,
    pluginPages = [],
    mode,
    appName = `Orbis`,
}: AppLayoutProps): React.ReactElement {
    const location = useLocation();
    const navigate = useNavigate();
    const auth = useAuth();

    const getIcon = (name?: string): React.ComponentType<{ className?: string }> | null => {
        if (!name) {
            return null;
        }
        const IconComponent = (LucideIcons as Record<string, unknown>)[
            name.charAt(0).toUpperCase() + name.slice(1)
        ] as React.ComponentType<{ className?: string }> | undefined;
        return IconComponent ?? null;
    };

    const renderNavItem = (item: NavigationItem): React.ReactNode => {
        const Icon = getIcon(item.icon);
        const isActive = location.pathname === item.href ||
            location.pathname.startsWith(item.href + `/`);

        return (
            <SidebarMenuItem key={item.id}>
                <SidebarMenuButton
                    asChild={Boolean(item.href)}
                    isActive={isActive}
                    disabled={item.disabled === true}
                >
                    {item.href
? (
                        <Link to={item.href}>
                            {Icon && <Icon className="h-4 w-4" />}
                            <span>{item.label}</span>
                            {item.badge && (
                                <Badge variant={item.badgeVariant ?? `default`} className="ml-auto">
                                    {item.badge}
                                </Badge>
                            )}
                        </Link>
                    )
: (
                        <span className="flex items-center gap-2">
                            {Icon && <Icon className="h-4 w-4" />}
                            <span>{item.label}</span>
                        </span>
                    )}
                </SidebarMenuButton>
            </SidebarMenuItem>
        );
    };

    const pluginNavItems: Array<NavigationItem> = pluginPages
        .filter((p) => p.show_in_menu)
        .sort((a, b) => (a.menu_order ?? 0) - (b.menu_order ?? 0))
        .map((page) => ({
            id:    `plugin-${ page.plugin }-${ page.route }`,
            label: page.title,
            icon:  page.icon,
            href:  `/plugins/${ page.plugin }${ page.route }`,
        }));

    return (
        <SidebarProvider>
            {/* Skip link for keyboard navigation accessibility */}
            <SkipLink targetId="main-content" />

            <div className="flex min-h-screen w-full">
                <Sidebar>
                    <SidebarHeader className="border-b px-4 py-3">
                        <Link to="/" className="flex items-center gap-2">
                            <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary text-primary-foreground">
                                <LucideIcons.Orbit className="h-5 w-5" />
                            </div>
                            <span className="font-bold text-lg">{appName}</span>
                            {mode && (
                                <Badge variant="secondary" className="ml-auto text-xs">
                                    {mode.mode}
                                </Badge>
                            )}
                        </Link>
                    </SidebarHeader>

                    <SidebarContent>
                        <ScrollArea className="flex-1">
                            {navigation?.primary && navigation.primary.length > 0 && (
                                <SidebarGroup>
                                    <SidebarGroupLabel>Navigation</SidebarGroupLabel>
                                    <SidebarGroupContent>
                                        <SidebarMenu>
                                            {navigation.primary.map(renderNavItem)}
                                        </SidebarMenu>
                                    </SidebarGroupContent>
                                </SidebarGroup>
                            )}

                            {pluginNavItems.length > 0 && (
                                <SidebarGroup>
                                    <SidebarGroupLabel>Plugins</SidebarGroupLabel>
                                    <SidebarGroupContent>
                                        <SidebarMenu>
                                            {pluginNavItems.map(renderNavItem)}
                                        </SidebarMenu>
                                    </SidebarGroupContent>
                                </SidebarGroup>
                            )}

                            {navigation?.secondary && navigation.secondary.length > 0 && (
                                <SidebarGroup>
                                    <SidebarGroupLabel>More</SidebarGroupLabel>
                                    <SidebarGroupContent>
                                        <SidebarMenu>
                                            {navigation.secondary.map(renderNavItem)}
                                        </SidebarMenu>
                                    </SidebarGroupContent>
                                </SidebarGroup>
                            )}
                        </ScrollArea>
                    </SidebarContent>

                    <SidebarFooter className="border-t p-4">
                        {auth.isAuthenticated && auth.user
? (
                            <DropdownMenu>
                                <DropdownMenuTrigger asChild>
                                    <Button variant="ghost" className="w-full justify-start gap-2">
                                        <Avatar className="h-6 w-6">
                                            <AvatarImage src={auth.user.avatar} alt={auth.user.name} />
                                            <AvatarFallback>{auth.user.name.charAt(0)}</AvatarFallback>
                                        </Avatar>
                                        <span className="truncate">{auth.user.name}</span>
                                        <LucideIcons.ChevronUp className="ml-auto h-4 w-4" />
                                    </Button>
                                </DropdownMenuTrigger>
                                <DropdownMenuContent align="start" className="w-56">
                                    <DropdownMenuItem onClick={async() => navigate(`/settings/profile`)}>
                                        <LucideIcons.User className="mr-2 h-4 w-4" />
                                        Profile
                                    </DropdownMenuItem>
                                    <DropdownMenuItem onClick={async() => navigate(`/settings`)}>
                                        <LucideIcons.Settings className="mr-2 h-4 w-4" />
                                        Settings
                                    </DropdownMenuItem>
                                    <DropdownMenuSeparator />
                                    <DropdownMenuItem onClick={() => void auth.logout()}>
                                        <LucideIcons.LogOut className="mr-2 h-4 w-4" />
                                        Logout
                                    </DropdownMenuItem>
                                </DropdownMenuContent>
                            </DropdownMenu>
                        )
: mode?.mode === 'client' || mode?.mode === 'server' ? (
                            <Button className="w-full" onClick={async() => navigate(`/login`)}>
                                <LucideIcons.LogIn className="mr-2 h-4 w-4" />
                                Sign In
                            </Button>
                        ) : null}
                    </SidebarFooter>
                </Sidebar>

                <SidebarInset className="flex flex-col flex-1">
                    <header className="sticky top-0 z-10 flex h-14 items-center gap-4 border-b bg-background px-4">
                        <SidebarTrigger />
                        <Separator orientation="vertical" className="h-6" />
                        <div className="flex-1" />
                    </header>

                    <main id="main-content" className="flex-1 p-6" role="main" aria-label="Main content">
                        {children}
                    </main>
                </SidebarInset>
            </div>
            <Toaster richColors position="top-right" />
        </SidebarProvider>
    );
}

export default AppLayout;
