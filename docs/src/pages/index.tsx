import type {ReactNode} from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import Heading from '@theme/Heading';

import styles from './index.module.css';

type FeatureItem = {
  title: string;
  description: ReactNode;
  icon: string;
};

const FeatureList: FeatureItem[] = [
  {
    title: 'Schema-Driven UI',
    icon: '🎨',
    description: (
      <>
        Define your entire UI with JSON schemas. 35+ components ready to use,
        from buttons to data tables. No React code required in your plugins.
      </>
    ),
  },
  {
    title: 'Plugin Architecture',
    icon: '🧩',
    description: (
      <>
        Build modular plugins that extend Orbis functionality. WASM sandboxing
        ensures security while enabling powerful customizations.
      </>
    ),
  },
  {
    title: 'Dual Deployment',
    icon: '🚀',
    description: (
      <>
        Deploy as a standalone desktop app with SQLite or as a multi-user
        server with PostgreSQL. Same codebase, flexible deployment.
      </>
    ),
  },
];

function Feature({title, icon, description}: FeatureItem) {
  return (
    <div className={clsx('col col--4')}>
      <div className="text--center padding-horiz--md">
        <div style={{fontSize: '3rem', marginBottom: '1rem'}}>{icon}</div>
        <Heading as="h3">{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

function HomepageHeader() {
  const {siteConfig} = useDocusaurusContext();
  return (
    <header className={clsx('hero', styles.heroBanner)}>
      <div className="container">
        <Heading as="h1" className="hero__title">
          {siteConfig.title}
        </Heading>
        <p className="hero__subtitle">{siteConfig.tagline}</p>
        <div className={styles.buttons}>
          <Link
            className="button button--primary button--lg"
            to="/docs/">
            Get Started →
          </Link>
        </div>
      </div>
    </header>
  );
}

function HomepageFeatures(): ReactNode {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}

export default function Home(): ReactNode {
  const {siteConfig} = useDocusaurusContext();
  return (
    <Layout
      title={siteConfig.title}
      description="Build extensible applications with schema-driven plugins">
      <HomepageHeader />
      <main>
        <HomepageFeatures />
        <div className="container margin-vert--xl">
          <div className="row">
            <div className="col col--6 col--offset-3">
              <div className="text--center">
                <Heading as="h2">Why Orbis?</Heading>
                <p>
                  Orbis is a modern desktop application framework that lets you build 
                  powerful, extensible applications using a plugin-based architecture.
                  Define your UI with simple JSON schemas and let Orbis handle the rest.
                </p>
              </div>
            </div>
          </div>
        </div>
      </main>
    </Layout>
  );
}
