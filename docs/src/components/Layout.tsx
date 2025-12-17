import React from 'react';
import Head from '@docusaurus/Head';
import { cn } from '@site/src/lib/utils';

interface LayoutProps {
  children: React.ReactNode;
  title?: string;
  description?: string;
  noFooter?: boolean;
  className?: string;
}

/**
 * Custom Layout component replacing Docusaurus theme Layout
 * Uses pure shadcn/Tailwind styling with no Docusaurus theme dependencies
 */
export default function Layout({
  children,
  title,
  description,
  noFooter = false,
  className,
}: LayoutProps) {
  const pageTitle = title ? `${title} | Orbis` : 'Orbis - Build Powerful Desktop Apps';
  const pageDescription = description || 'A modern desktop application framework with plugin-driven architecture';

  return (
    <>
      <Head>
        <title>{pageTitle}</title>
        <meta name="description" content={pageDescription} />
        <meta property="og:title" content={pageTitle} />
        <meta property="og:description" content={pageDescription} />
        <meta name="twitter:card" content="summary_large_image" />
        <meta name="twitter:title" content={pageTitle} />
        <meta name="twitter:description" content={pageDescription} />
      </Head>

      <div className={cn('flex min-h-screen flex-col', className)}>
        {/* Main Content */}
        <main className="flex-1">
          {children}
        </main>

        {/* Footer */}
        {!noFooter && (
          <footer className="border-t bg-background py-12">
            <div className="mx-auto max-w-7xl px-6 lg:px-8">
              <div className="grid grid-cols-1 gap-8 md:grid-cols-4">
                <div className="md:col-span-2">
                  <h3 className="text-lg font-semibold">Orbis</h3>
                  <p className="mt-4 text-sm text-muted-foreground max-w-md">
                    Build extensible desktop applications with schema-driven plugins.
                    Modern, type-safe, and lightning fast.
                  </p>
                </div>
                
                <div>
                  <h4 className="font-semibold mb-4">Documentation</h4>
                  <ul className="space-y-2 text-sm">
                    <li>
                      <a href="/docs/" className="text-muted-foreground hover:text-primary transition-colors">
                        Getting Started
                      </a>
                    </li>
                    <li>
                      <a href="/docs/core-concepts/architecture" className="text-muted-foreground hover:text-primary transition-colors">
                        Core Concepts
                      </a>
                    </li>
                    <li>
                      <a href="/docs/api-reference/components" className="text-muted-foreground hover:text-primary transition-colors">
                        API Reference
                      </a>
                    </li>
                    <li>
                      <a href="/docs/plugin-development/getting-started" className="text-muted-foreground hover:text-primary transition-colors">
                        Plugin Development
                      </a>
                    </li>
                  </ul>
                </div>
                
                <div>
                  <h4 className="font-semibold mb-4">Community</h4>
                  <ul className="space-y-2 text-sm">
                    <li>
                      <a 
                        href="https://github.com/cyberpath-HQ/orbis" 
                        target="_blank" 
                        rel="noopener noreferrer"
                        className="text-muted-foreground hover:text-primary transition-colors"
                      >
                        GitHub
                      </a>
                    </li>
                    <li>
                      <a href="/docs/changelog" className="text-muted-foreground hover:text-primary transition-colors">
                        Changelog
                      </a>
                    </li>
                  </ul>
                </div>
              </div>
              
              <div className="mt-12 border-t pt-8 text-center text-sm text-muted-foreground">
                <p>© {new Date().getFullYear()} Orbis. Released under the MIT License.</p>
              </div>
            </div>
          </footer>
        )}
      </div>
    </>
  );
}
