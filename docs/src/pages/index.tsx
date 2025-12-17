import type {ReactNode} from 'react';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@site/src/components/Layout';
import { Button } from '@site/src/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@site/src/components/ui/card';
import { 
  Code2, 
  Puzzle, 
  Rocket, 
  Shield, 
  Zap, 
  Database,
  ArrowRight,
  Github,
  BookOpen,
  Terminal
} from 'lucide-react';

const features = [
  {
    title: 'Schema-Driven UI',
    description: 'Define your entire UI with JSON schemas. 37+ components ready to use, from buttons to data tables.',
    icon: Code2,
    gradient: 'from-blue-500 to-cyan-500',
  },
  {
    title: 'Plugin Architecture',
    description: 'Build modular plugins that extend Orbis functionality. WASM sandboxing ensures security.',
    icon: Puzzle,
    gradient: 'from-purple-500 to-pink-500',
  },
  {
    title: 'Dual Deployment',
    description: 'Deploy as a standalone desktop app with SQLite or as a multi-user server with PostgreSQL.',
    icon: Rocket,
    gradient: 'from-orange-500 to-red-500',
  },
  {
    title: 'Type-Safe',
    description: 'Rust backend with TypeScript frontend. Full type safety from database to UI.',
    icon: Shield,
    gradient: 'from-green-500 to-emerald-500',
  },
  {
    title: 'Lightning Fast',
    description: 'Built on Tauri for native performance. React 18 with modern optimization.',
    icon: Zap,
    gradient: 'from-yellow-500 to-amber-500',
  },
  {
    title: 'Flexible Data',
    description: 'Switch between SQLite and PostgreSQL without code changes. Migrations handled automatically.',
    icon: Database,
    gradient: 'from-indigo-500 to-blue-500',
  },
];

const stats = [
  { label: 'Components', value: '37+' },
  { label: 'Action Types', value: '16' },
  { label: 'Type-Safe', value: '100%' },
  { label: 'Open Source', value: 'MIT' },
];

function FeatureCard({title, description, icon: Icon, gradient}: typeof features[0]) {
  return (
    <Card className="group relative overflow-hidden transition-all hover:shadow-lg">
      <div className={`absolute inset-0 bg-gradient-to-br ${gradient} opacity-0 group-hover:opacity-5 transition-opacity`} />
      <CardHeader>
        <div className={`mb-4 inline-flex h-12 w-12 items-center justify-center rounded-lg bg-gradient-to-br ${gradient}`}>
          <Icon className="h-6 w-6 text-white" />
        </div>
        <CardTitle className="text-xl">{title}</CardTitle>
        <CardDescription className="text-base">{description}</CardDescription>
      </CardHeader>
    </Card>
  );
}

function StatItem({label, value}: typeof stats[0]) {
  return (
    <div className="text-center">
      <div className="text-4xl font-bold bg-gradient-to-r from-primary to-primary/60 bg-clip-text text-transparent">
        {value}
      </div>
      <div className="mt-2 text-sm text-muted-foreground">{label}</div>
    </div>
  );
}

export default function Home(): ReactNode {
  const {siteConfig} = useDocusaurusContext();
  
  return (
    <Layout
      title={siteConfig.title}
      description="Build extensible applications with schema-driven plugins">
      
      {/* Hero Section */}
      <section className="relative overflow-hidden border-b bg-background">
        <div className="absolute inset-0 bg-grid-slate-100 [mask-image:linear-gradient(0deg,white,rgba(255,255,255,0.6))] dark:bg-grid-slate-700/25 dark:[mask-image:linear-gradient(0deg,rgba(255,255,255,0.1),rgba(255,255,255,0.5))]" />
        <div className="relative mx-auto max-w-7xl px-6 py-24 sm:py-32 lg:px-8">
          <div className="mx-auto max-w-2xl text-center">
            <h1 className="text-5xl font-bold tracking-tight sm:text-7xl bg-gradient-to-r from-primary via-primary/80 to-primary/60 bg-clip-text text-transparent">
              Build Powerful Apps with Orbis
            </h1>
            <p className="mt-6 text-lg leading-8 text-muted-foreground">
              A modern desktop application framework with plugin-driven architecture. 
              Define your UI with JSON schemas and let Orbis handle the rest.
            </p>
            <div className="mt-10 flex items-center justify-center gap-4">
              <Button asChild size="lg" className="gap-2">
                <a href="/docs/">
                  Get Started
                  <ArrowRight className="h-4 w-4" />
                </a>
              </Button>
              <Button asChild variant="outline" size="lg" className="gap-2">
                <a href="https://github.com/cyberpath-HQ/orbis" target="_blank" rel="noopener noreferrer">
                  <Github className="h-4 w-4" />
                  GitHub
                </a>
              </Button>
            </div>
          </div>
        </div>
      </section>

      {/* Stats Section */}
      <section className="border-b bg-muted/50 py-12">
        <div className="mx-auto max-w-7xl px-6 lg:px-8">
          <div className="grid grid-cols-2 gap-8 md:grid-cols-4">
            {stats.map((stat) => (
              <StatItem key={stat.label} {...stat} />
            ))}
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section className="py-24">
        <div className="mx-auto max-w-7xl px-6 lg:px-8">
          <div className="mx-auto max-w-2xl text-center mb-16">
            <h2 className="text-3xl font-bold tracking-tight sm:text-4xl">
              Everything you need to build amazing apps
            </h2>
            <p className="mt-4 text-lg text-muted-foreground">
              Orbis provides a complete toolkit for building modern desktop applications
            </p>
          </div>
          
          <div className="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
            {features.map((feature) => (
              <FeatureCard key={feature.title} {...feature} />
            ))}
          </div>
        </div>
      </section>

      {/* Code Example Section */}
      <section className="border-y bg-muted/30 py-24">
        <div className="mx-auto max-w-7xl px-6 lg:px-8">
          <div className="grid grid-cols-1 gap-12 lg:grid-cols-2 lg:gap-8 items-center">
            <div>
              <h2 className="text-3xl font-bold tracking-tight">
                Schema-driven development
              </h2>
              <p className="mt-4 text-lg text-muted-foreground">
                Define your entire UI with simple JSON schemas. No React code required in your plugins.
              </p>
              <div className="mt-8 space-y-4">
                <div className="flex items-start gap-3">
                  <div className="flex h-8 w-8 items-center justify-center rounded-full bg-primary/10">
                    <Code2 className="h-4 w-4 text-primary" />
                  </div>
                  <div>
                    <h3 className="font-semibold">Type-Safe Schemas</h3>
                    <p className="text-sm text-muted-foreground">Rust types generate TypeScript definitions automatically</p>
                  </div>
                </div>
                <div className="flex items-start gap-3">
                  <div className="flex h-8 w-8 items-center justify-center rounded-full bg-primary/10">
                    <Zap className="h-4 w-4 text-primary" />
                  </div>
                  <div>
                    <h3 className="font-semibold">Hot Reload</h3>
                    <p className="text-sm text-muted-foreground">Changes reflect instantly during development</p>
                  </div>
                </div>
                <div className="flex items-start gap-3">
                  <div className="flex h-8 w-8 items-center justify-center rounded-full bg-primary/10">
                    <Shield className="h-4 w-4 text-primary" />
                  </div>
                  <div>
                    <h3 className="font-semibold">Sandboxed Execution</h3>
                    <p className="text-sm text-muted-foreground">WASM plugins run in secure isolation</p>
                  </div>
                </div>
              </div>
            </div>
            
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Terminal className="h-5 w-5" />
                  page-definition.json
                </CardTitle>
              </CardHeader>
              <CardContent>
                <pre className="text-sm overflow-x-auto">
                  <code className="language-json">{`{
  "type": "Container",
  "children": [
    {
      "type": "Heading",
      "text": "Welcome to Orbis",
      "level": 1
    },
    {
      "type": "Button",
      "label": "Get Started",
      "onClick": [
        {
          "type": "navigate",
          "to": "/dashboard"
        }
      ]
    }
  ]
}`}</code>
                </pre>
              </CardContent>
            </Card>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="py-24">
        <div className="mx-auto max-w-7xl px-6 lg:px-8">
          <Card className="overflow-hidden">
            <div className="bg-gradient-to-r from-primary to-primary/80 px-6 py-12 sm:px-12">
              <div className="mx-auto max-w-2xl text-center">
                <h2 className="text-3xl font-bold tracking-tight text-primary-foreground sm:text-4xl">
                  Ready to start building?
                </h2>
                <p className="mx-auto mt-4 max-w-xl text-lg text-primary-foreground/90">
                  Get started with Orbis today and build your next desktop application in record time.
                </p>
                <div className="mt-8 flex items-center justify-center gap-4">
                  <Button asChild size="lg" variant="secondary" className="gap-2">
                    <a href="/docs/">
                      <BookOpen className="h-4 w-4" />
                      Read Documentation
                    </a>
                  </Button>
                  <Button asChild size="lg" variant="outline" className="gap-2 bg-white/10 border-white/20 text-white hover:bg-white/20">
                    <a href="/docs/getting-started/quickstart">
                      <Rocket className="h-4 w-4" />
                      Quick Start
                    </a>
                  </Button>
                </div>
              </div>
            </div>
          </Card>
        </div>
      </section>
    </Layout>
  );
}
