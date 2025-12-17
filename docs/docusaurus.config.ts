import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

const config: Config = {
  title: 'Orbis Documentation',
  tagline: 'Build powerful desktop applications with plugin-driven architecture',
  favicon: 'img/favicon.ico',

  // Future flags, see https://docusaurus.io/docs/api/docusaurus-config#future
  future: {
    v4: true, // Improve compatibility with the upcoming Docusaurus v4
  },

  // Production URL
  url: 'https://docs.orbis.cyberpath-hq.com',
  baseUrl: '/',

  // GitHub deployment config
  organizationName: 'cyberpath-HQ',
  projectName: 'orbis',

  onBrokenLinks: 'throw',

  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  plugins: ["./src/tailwind-config.js"],

  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          editUrl: 'https://github.com/cyberpath-HQ/orbis/tree/main/docs/',
          // Enable versioning
          lastVersion: 'current',
          versions: {
            current: {
              label: '1.x',
              badge: true,
            },
          },
        },
        blog: false,
        theme: {
          customCss: './src/css/custom.css',
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    image: 'img/orbis-social-card.png',
    
    // Announcement bar for important notices
    announcementBar: {
      id: 'v1_release',
      content: '🎉 Orbis v1.0 is now available! <a href="/docs/">Get started</a>',
      backgroundColor: 'var(--ifm-color-primary)',
      textColor: 'var(--ifm-color-primary-contrast-foreground)',
      isCloseable: true,
    },

    colorMode: {
      defaultMode: 'light',
      disableSwitch: false,
      respectPrefersColorScheme: true,
    },

    navbar: {
      title: 'Orbis',
      logo: {
        alt: 'Orbis Logo',
        src: 'img/logo.svg',
      },
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'docsSidebar',
          position: 'left',
          label: 'Docs',
        },
        {
          to: '/docs/api-reference/overview',
          position: 'left',
          label: 'API Reference',
        },
        {
          type: 'docsVersionDropdown',
          position: 'right',
          dropdownActiveClassDisabled: true,
        },
        {
          href: 'https://github.com/cyberpath-HQ/orbis',
          label: 'GitHub',
          position: 'right',
        },
      ],
    },

    footer: {
      style: 'dark',
      links: [
        {
          title: 'Learn',
          items: [
            {
              label: 'Getting Started',
              to: '/docs/',
            },
            {
              label: 'Core Concepts',
              to: '/docs/core-concepts/architecture',
            },
            {
              label: 'Plugin Development',
              to: '/docs/plugin-development/overview',
            },
          ],
        },
        {
          title: 'Reference',
          items: [
            {
              label: 'Components',
              to: '/docs/components/overview',
            },
            {
              label: 'Actions',
              to: '/docs/actions/overview',
            },
            {
              label: 'API',
              to: '/docs/api-reference/state-management',
            },
          ],
        },
        {
          title: 'Community',
          items: [
            {
              label: 'GitHub',
              href: 'https://github.com/cyberpath-HQ/orbis',
            },
            {
              label: 'Discord',
              href: 'https://discord.gg/orbis',
            },
          ],
        },
        {
          title: 'More',
          items: [
            {
              label: 'Changelog',
              to: '/docs/changelog',
            },
            {
              label: 'Contributing',
              href: 'https://github.com/cyberpath-HQ/orbis/blob/main/CONTRIBUTING.md',
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} Orbis. Built with Docusaurus.`,
    },

    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ['rust', 'json', 'bash', 'typescript'],
    },

    // Table of contents
    tableOfContents: {
      minHeadingLevel: 2,
      maxHeadingLevel: 4,
    },

    // Docs features
    docs: {
      sidebar: {
        hideable: true,
        autoCollapseCategories: true,
      },
    },
  } satisfies Preset.ThemeConfig,

  // Markdown features
  markdown: {
    mermaid: true,
    hooks: {
      onBrokenMarkdownLinks: `throw`,
    }
  },

  // Additional themes for diagrams
  themes: ['@docusaurus/theme-mermaid'],
};

export default config;
