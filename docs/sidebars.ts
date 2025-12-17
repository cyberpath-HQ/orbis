import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

/**
 * Orbis Documentation Sidebar Configuration
 * 
 * This defines the structure of the documentation navigation.
 * The sidebar is organized into logical sections for easy navigation.
 */
const sidebars: SidebarsConfig = {
  // Main documentation sidebar
  docsSidebar: [
    // Introduction
    'intro',
    
    // Getting Started
    {
      type: 'category',
      label: 'Getting Started',
      collapsed: false,
      items: [
        'getting-started/installation',
        'getting-started/quickstart',
        'getting-started/project-structure',
      ],
    },
    
    // Core Concepts
    {
      type: 'category',
      label: 'Core Concepts',
      collapsed: false,
      items: [
        'core-concepts/architecture',
        'core-concepts/plugin-system',
        'core-concepts/schema-system',
        'core-concepts/state-management',
        'core-concepts/expressions',
        'core-concepts/event-handling',
      ],
    },
    
    // Plugin Development
    {
      type: 'category',
      label: 'Plugin Development',
      collapsed: true,
      items: [
        'plugin-development/overview',
        'plugin-development/manifest',
        'plugin-development/wasm-plugins',
        'plugin-development/page-definitions',
        'plugin-development/building-plugins',
        'plugin-development/testing-plugins',
        'plugin-development/best-practices',
      ],
    },
    
    // UI Components Reference
    {
      type: 'category',
      label: 'Components',
      collapsed: true,
      items: [
        'components/overview',
        {
          type: 'category',
          label: 'Layout',
          items: ['components/layout/container'],
        },
        {
          type: 'category',
          label: 'Typography',
          items: ['components/typography/text'],
        },
        {
          type: 'category',
          label: 'Forms',
          items: ['components/forms/form'],
        },
        {
          type: 'category',
          label: 'Data Display',
          items: ['components/data-display/table'],
        },
        {
          type: 'category',
          label: 'Feedback',
          items: ['components/feedback/alert'],
        },
        {
          type: 'category',
          label: 'Navigation',
          items: ['components/navigation/button'],
        },
        {
          type: 'category',
          label: 'Overlays',
          items: ['components/overlays/modal'],
        },
        {
          type: 'category',
          label: 'Advanced',
          items: ['components/advanced/conditional'],
        },
      ],
    },
    
    // Actions Reference
    {
      type: 'category',
      label: 'Actions',
      collapsed: true,
      items: [
        'actions/overview',
        'actions/update-state',
        'actions/call-api',
        'actions/navigate',
        'actions/show-toast',
        'actions/dialogs',
        'actions/form-actions',
        'actions/utility-actions',
        'actions/flow-control',
      ],
    },
    
    // API Reference
    {
      type: 'category',
      label: 'API Reference',
      collapsed: true,
      items: [
        'api-reference/overview',
        'api-reference/state-management',
        'api-reference/expressions',
        'api-reference/event-handlers',
        'api-reference/special-values',
        'api-reference/validation',
        'api-reference/data-sources',
        'api-reference/types-reference',
      ],
    },
    
    // Configuration
    {
      type: 'category',
      label: 'Configuration',
      collapsed: true,
      items: [
        'configuration/overview',
        'configuration/server',
        'configuration/database',
        'configuration/authentication',
        'configuration/tls-security',
      ],
    },
    
    // Deployment
    {
      type: 'category',
      label: 'Deployment',
      collapsed: true,
      items: [
        'deployment/overview',
        'deployment/standalone',
        'deployment/client-server',
        'deployment/docker',
      ],
    },
    
    // Changelog
    'changelog',
  ],
};

export default sidebars;
