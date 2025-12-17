import React, { useState } from 'react';
import { cn } from '@site/src/lib/utils';
import type { ComponentProps, ReactElement } from 'react';
import {
  Table as ShadcnTable,
  TableHeader as ShadcnTableHeader,
  TableBody as ShadcnTableBody,
  TableRow as ShadcnTableRow,
  TableHead as ShadcnTableHead,
  TableCell as ShadcnTableCell,
} from '@site/src/components/ui/table';
import { Alert, AlertTitle, AlertDescription } from '@site/src/components/ui/alert';
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@site/src/components/ui/card';
import { Separator } from '@site/src/components/ui/separator';
import { Badge } from '@site/src/components/ui/badge';
import { Button } from '@site/src/components/ui/button';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { oneDark, oneLight } from 'react-syntax-highlighter/dist/esm/styles/prism';
import { Check, Copy } from 'lucide-react';
import TabsComponent, { TabItem } from '@site/src/components/Tabs';

// ============================================================================
// HEADING COMPONENTS - Using shadcn typography patterns
// ============================================================================

const Heading1 = ({ children, className, ...props }: ComponentProps<'h1'>) => (
  <h1 
    className={cn(
      "scroll-m-20 text-4xl font-bold tracking-tight lg:text-5xl",
      "border-b pb-2 mb-6 mt-8 first:mt-0",
      className
    )} 
    {...props}
  >
    {children}
  </h1>
);

const Heading2 = ({ children, className, ...props }: ComponentProps<'h2'>) => (
  <h2 
    className={cn(
      "scroll-m-20 border-b pb-2 text-3xl font-semibold tracking-tight",
      "mt-10 mb-4 first:mt-0",
      className
    )} 
    {...props}
  >
    {children}
  </h2>
);

const Heading3 = ({ children, className, ...props }: ComponentProps<'h3'>) => (
  <h3 
    className={cn(
      "scroll-m-20 text-2xl font-semibold tracking-tight",
      "mt-8 mb-4 first:mt-0",
      className
    )} 
    {...props}
  >
    {children}
  </h3>
);

const Heading4 = ({ children, className, ...props }: ComponentProps<'h4'>) => (
  <h4 
    className={cn(
      "scroll-m-20 text-xl font-semibold tracking-tight",
      "mt-6 mb-3 first:mt-0",
      className
    )} 
    {...props}
  >
    {children}
  </h4>
);

const Heading5 = ({ children, className, ...props }: ComponentProps<'h5'>) => (
  <h5 
    className={cn(
      "scroll-m-20 text-lg font-semibold tracking-tight",
      "mt-4 mb-2 first:mt-0",
      className
    )} 
    {...props}
  >
    {children}
  </h5>
);

const Heading6 = ({ children, className, ...props }: ComponentProps<'h6'>) => (
  <h6 
    className={cn(
      "scroll-m-20 text-base font-semibold tracking-tight",
      "mt-4 mb-2 first:mt-0",
      className
    )} 
    {...props}
  >
    {children}
  </h6>
);

// ============================================================================
// TEXT COMPONENTS - Using shadcn typography patterns
// ============================================================================

const Paragraph = ({ children, className, ...props }: ComponentProps<'p'>) => (
  <p 
    className={cn(
      "leading-7 text-base",
      "not-first:mt-6",
      className
    )} 
    {...props}
  >
    {children}
  </p>
);

const Blockquote = ({ children, className, ...props }: ComponentProps<'blockquote'>) => (
  <Alert className={cn("mt-6 mb-4 border-l-4", className)}>
    <AlertDescription className="italic text-muted-foreground">
      {children}
    </AlertDescription>
  </Alert>
);

const Strong = ({ children, className, ...props }: ComponentProps<'strong'>) => (
  <strong className={cn("font-semibold text-foreground", className)} {...props}>
    {children}
  </strong>
);

const Em = ({ children, className, ...props }: ComponentProps<'em'>) => (
  <em className={cn("italic", className)} {...props}>
    {children}
  </em>
);

// ============================================================================
// LIST COMPONENTS - Enhanced with shadcn styling
// ============================================================================

const UnorderedList = ({ children, className, ...props }: ComponentProps<'ul'>) => (
  <ul 
    className={cn(
      "my-6 ml-6 list-disc space-y-2",
      "[&>li]:leading-7",
      "marker:text-muted-foreground",
      className
    )} 
    {...props}
  >
    {children}
  </ul>
);

const OrderedList = ({ children, className, ...props }: ComponentProps<'ol'>) => (
  <ol 
    className={cn(
      "my-6 ml-6 list-decimal space-y-2",
      "[&>li]:leading-7",
      "marker:text-muted-foreground marker:font-medium",
      className
    )} 
    {...props}
  >
    {children}
  </ol>
);

const ListItem = ({ children, className, ...props }: ComponentProps<'li'>) => (
  <li 
    className={cn(
      "leading-7",
      "[&>ul]:mt-2 [&>ol]:mt-2",
      className
    )} 
    {...props}
  >
    {children}
  </li>
);

// ============================================================================
// CODE COMPONENTS - Enhanced with syntax highlighting and copy button
// ============================================================================

/**
 * CopyButton - Floating button to copy code content
 */
const CopyButton = ({ text }: { text: string }) => {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    await navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <Button
      variant="ghost"
      size="icon"
      className={cn(
        "h-8 w-8 opacity-0 group-hover:opacity-100 transition-opacity",
        "hover:bg-muted/80"
      )}
      onClick={handleCopy}
      aria-label="Copy code"
    >
      {copied ? (
        <Check className="h-4 w-4 text-green-500" />
      ) : (
        <Copy className="h-4 w-4" />
      )}
    </Button>
  );
};

/**
 * Extract language from className (e.g., "language-typescript" -> "typescript")
 */
const getLanguage = (className?: string): string => {
  if (!className) return 'text';
  const match = className.match(/language-(\w+)/);
  return match ? match[1] : 'text';
};

/**
 * Get display name for language badge
 */
const getLanguageDisplay = (lang: string): string => {
  const displayNames: Record<string, string> = {
    js: 'JavaScript',
    jsx: 'JSX',
    ts: 'TypeScript',
    tsx: 'TSX',
    bash: 'Bash',
    sh: 'Shell',
    json: 'JSON',
    md: 'Markdown',
    yaml: 'YAML',
    yml: 'YAML',
    toml: 'TOML',
    rs: 'Rust',
    py: 'Python',
    go: 'Go',
    java: 'Java',
    cpp: 'C++',
    c: 'C',
    cs: 'C#',
    php: 'PHP',
    rb: 'Ruby',
    sql: 'SQL',
    html: 'HTML',
    css: 'CSS',
    scss: 'SCSS',
    graphql: 'GraphQL',
    dockerfile: 'Dockerfile',
    text: 'Plain Text',
  };
  return displayNames[lang.toLowerCase()] || lang.toUpperCase();
};

/**
 * Inline code - for `code` elements
 */
const InlineCode = ({ children, className, ...props }: ComponentProps<'code'>) => (
  <code 
    className={cn(
      "relative rounded bg-muted px-[0.4rem] py-[0.2rem]",
      "font-mono text-sm",
      "border border-border/50",
      className
    )} 
    {...props}
  >
    {children}
  </code>
);

/**
 * Enhanced Pre - Code block with syntax highlighting, language badge, and copy button
 */
const Pre = ({ children, className, ...props }: ComponentProps<'pre'>) => {
  // Check if this is a dark theme (you can customize this based on your theme system)
  const [isDark, setIsDark] = React.useState(false);

  React.useEffect(() => {
    // Check for dark mode
    const checkDarkMode = () => {
      const isDarkMode = 
        document.documentElement.classList.contains('dark') ||
        document.documentElement.getAttribute('data-theme') === 'dark';
      setIsDark(isDarkMode);
    };

    checkDarkMode();

    // Watch for theme changes
    const observer = new MutationObserver(checkDarkMode);
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ['class', 'data-theme'],
    });

    return () => observer.disconnect();
  }, []);

  // Extract code content and language from children
  const codeElement = React.Children.toArray(children).find(
    (child): child is ReactElement<{ children?: string; className?: string }> => 
      React.isValidElement(child) && child.type === 'code'
  );

  if (!codeElement) {
    // Fallback for pre without code element
    return (
      <Card className={cn("my-6 overflow-hidden", className)}>
        <pre 
          className={cn(
            "overflow-x-auto p-4",
            "bg-muted/50",
            "text-sm font-mono",
          )} 
          {...props}
        >
          {children}
        </pre>
      </Card>
    );
  }

  const codeContent = typeof codeElement.props.children === 'string' 
    ? codeElement.props.children 
    : String(codeElement.props.children || '');
  
  const language = getLanguage(codeElement.props.className);
  const languageDisplay = getLanguageDisplay(language);

  return (
    <Card className={cn("my-6 overflow-hidden group relative", className)}>
      {/* Header with language badge and copy button */}
      <div className="flex items-center justify-between px-4 py-2 border-b bg-muted/30">
        <Badge variant="secondary" className="text-xs font-mono">
          {languageDisplay}
        </Badge>
        <CopyButton text={codeContent.trim()} />
      </div>

      {/* Syntax highlighted code */}
      <div className="overflow-x-auto">
        <SyntaxHighlighter
          language={language}
          style={isDark ? oneDark : oneLight}
          customStyle={{
            margin: 0,
            padding: '1rem',
            background: 'transparent',
            fontSize: '0.875rem',
          }}
          codeTagProps={{
            style: {
              fontFamily: 'var(--font-mono, ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace)',
            },
          }}
        >
          {codeContent.trim()}
        </SyntaxHighlighter>
      </div>
    </Card>
  );
};

// ============================================================================
// TABLE COMPONENTS - Using shadcn Table components directly
// ============================================================================

const MdxTable = ({ children, className, ...props }: ComponentProps<'table'>) => (
  <div className="my-6 w-full overflow-auto">
    <ShadcnTable className={className} {...props}>
      {children}
    </ShadcnTable>
  </div>
);

const MdxTableHead = ({ children, className, ...props }: ComponentProps<'thead'>) => (
  <ShadcnTableHeader className={className} {...props}>
    {children}
  </ShadcnTableHeader>
);

const MdxTableBody = ({ children, className, ...props }: ComponentProps<'tbody'>) => (
  <ShadcnTableBody className={className} {...props}>
    {children}
  </ShadcnTableBody>
);

const MdxTableRow = ({ children, className, ...props }: ComponentProps<'tr'>) => (
  <ShadcnTableRow className={className} {...props}>
    {children}
  </ShadcnTableRow>
);

const MdxTableHeader = ({ children, className, ...props }: ComponentProps<'th'>) => (
  <ShadcnTableHead className={className} {...props}>
    {children}
  </ShadcnTableHead>
);

const MdxTableCell = ({ children, className, ...props }: ComponentProps<'td'>) => (
  <ShadcnTableCell className={className} {...props}>
    {children}
  </ShadcnTableCell>
);

// ============================================================================
// LINK COMPONENT - Enhanced with hover effects
// ============================================================================

const Link = ({ children, className, ...props }: ComponentProps<'a'>) => (
  <a 
    className={cn(
      "font-medium text-primary underline underline-offset-4",
      "hover:text-primary/80 transition-colors",
      "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
      className
    )} 
    {...props}
  >
    {children}
  </a>
);

// ============================================================================
// HORIZONTAL RULE - Using shadcn Separator component
// ============================================================================

const HorizontalRule = ({ className, ...props }: ComponentProps<'hr'>) => (
  <Separator className={cn("my-8", className)} {...props} />
);

// ============================================================================
// EXPORT - Pure shadcn components, no Docusaurus theme
// ============================================================================

export default {
  // Headings
  h1: Heading1,
  h2: Heading2,
  h3: Heading3,
  h4: Heading4,
  h5: Heading5,
  h6: Heading6,
  // Text
  p: Paragraph,
  blockquote: Blockquote,
  strong: Strong,
  em: Em,
  // Lists
  ul: UnorderedList,
  ol: OrderedList,
  li: ListItem,
  // Code
  code: InlineCode,
  pre: Pre,
  // Tables - using shadcn components
  table: MdxTable,
  thead: MdxTableHead,
  tbody: MdxTableBody,
  tr: MdxTableRow,
  th: MdxTableHeader,
  td: MdxTableCell,
  // Other
  a: Link,
  hr: HorizontalRule,
  // Tabs - shadcn tabs for MDX
  Tabs: TabsComponent,
  TabItem: TabItem,
};
