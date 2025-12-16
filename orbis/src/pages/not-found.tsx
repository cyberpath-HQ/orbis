/**
 * Not Found page
 */

import React from 'react';
import { useNavigate } from 'react-router-dom';
import * as LucideIcons from 'lucide-react';

import { Button } from '@/components/ui/button';

export function NotFoundPage(): React.ReactElement {
    const navigate = useNavigate();

    return (
        <div className="flex min-h-[60vh] flex-col items-center justify-center text-center">
            <LucideIcons.FileQuestion className="h-16 w-16 text-muted-foreground mb-4" />
            <h1 className="text-4xl font-bold mb-2">404</h1>
            <p className="text-xl text-muted-foreground mb-6">Page not found</p>
            <p className="text-muted-foreground mb-8 max-w-md">
                The page you are looking for does not exist or has been moved.
            </p>
            <Button onClick={async() => navigate(`/`)}>
                <LucideIcons.Home className="mr-2 h-4 w-4" />
                Go Home
            </Button>
        </div>
    );
}

export default NotFoundPage;
