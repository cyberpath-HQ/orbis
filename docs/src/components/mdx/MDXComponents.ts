import type { MDXComponents } from 'mdx/types';
import CodeBlock from './CodeBlock.astro';
import Tabs from './Tabs.astro';
import Tab from './Tab.astro';
import Card from './Card.astro';
import Callout from './Callout.astro';
import Mermaid from './Mermaid.astro';

export const components: MDXComponents = {
    // Override pre element to use our CodeBlock component
    pre: CodeBlock,
    // Custom components available in MDX
    Tabs,
    Tab,
    Card,
    Callout,
    Mermaid,
};
