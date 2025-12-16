/**
 * Unauthorized page
 */

import React from 'react';
import { useNavigate } from 'react-router-dom';
import * as LucideIcons from 'lucide-react';

import { Button } from '@/components/ui/button';

export function UnauthorizedPage(): React.ReactElement {
    const navigate = useNavigate();

    return (
        <div className="flex min-h-[60vh] flex-col items-center justify-center text-center">
            <LucideIcons.ShieldX className="h-16 w-16 text-destructive mb-4" />
            <h1 className="text-4xl font-bold mb-2">Unauthorized</h1>
            <p className="text-xl text-muted-foreground mb-6">Access Denied</p>
            <p className="text-muted-foreground mb-8 max-w-md">
                You do not have permission to access this page.
                Please contact your administrator if you believe this is an error.
            </p>
            <div className="flex gap-4">
                <Button variant="outline" onClick={async() => navigate(-1)}>
                    <LucideIcons.ArrowLeft className="mr-2 h-4 w-4" />
                    Go Back
                </Button>
                <Button onClick={async() => navigate(`/`)}>
                    <LucideIcons.Home className="mr-2 h-4 w-4" />
                    Go Home
                </Button>
            </div>
        </div>
    );
}

export default UnauthorizedPage;
