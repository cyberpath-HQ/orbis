/**
 * Login page
 */

import React, { useState } from 'react';
import {
    useNavigate, useLocation
} from 'react-router-dom';
import * as LucideIcons from 'lucide-react';

import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle
} from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { useAuth } from '@/lib/router';

export function LoginPage(): React.ReactElement {
    const navigate = useNavigate();
    const location = useLocation();
    const auth = useAuth();
    const [
        email,
        setEmail,
    ] = useState(``);
    const [
        password,
        setPassword,
    ] = useState(``);
    const [
        isLoading,
        setIsLoading,
    ] = useState(false);

    const from = (location.state as { from?: { pathname: string } })?.from?.pathname ?? `/`;

    const handleSubmit = async(e: React.FormEvent): Promise<void> => {
        e.preventDefault();
        setIsLoading(true);

        try {
            await auth.login(email, password);
            navigate(from, {
                replace: true,
            });
        }
        catch {
            setIsLoading(false);
        }
    };

    return (
        <div className="flex min-h-screen items-center justify-center">
            <Card className="w-full max-w-md">
                <CardHeader className="text-center">
                    <div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-lg bg-primary">
                        <LucideIcons.Orbit className="h-6 w-6 text-primary-foreground" />
                    </div>
                    <CardTitle className="text-2xl">Welcome to Orbis</CardTitle>
                    <CardDescription>
                        Sign in to your account to continue.
                    </CardDescription>
                </CardHeader>
                <CardContent>
                    <form onSubmit={(e) => void handleSubmit(e)} className="space-y-4">
                        <div className="space-y-2">
                            <Label htmlFor="email">Email</Label>
                            <Input
                                id="email"
                                type="email"
                                placeholder="you@example.com"
                                value={email}
                                onChange={(e) => setEmail(e.target.value)}
                                required
                            />
                        </div>
                        <div className="space-y-2">
                            <Label htmlFor="password">Password</Label>
                            <Input
                                id="password"
                                type="password"
                                placeholder="Enter your password"
                                value={password}
                                onChange={(e) => setPassword(e.target.value)}
                                required
                            />
                        </div>
                        <Button type="submit" className="w-full" disabled={isLoading}>
                            {isLoading && (
                                <LucideIcons.Loader2 className="mr-2 h-4 w-4 animate-spin" />
                            )}
                            Sign In
                        </Button>
                    </form>
                </CardContent>
            </Card>
        </div>
    );
}

export default LoginPage;
