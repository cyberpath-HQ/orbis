import React from 'react';
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@site/src/components/ui/tabs';

interface TabItemProps {
  value: string;
  label: string;
  children: React.ReactNode;
}

interface TabsComponentProps {
  children: React.ReactNode;
  defaultValue?: string;
}

/**
 * TabItem - Individual tab content wrapper
 * Matches Docusaurus TabItem API for compatibility
 */
export function TabItem({ children }: TabItemProps) {
  return <>{children}</>;
}

/**
 * TabsComponent - Main tabs wrapper using shadcn UI
 * Replaces Docusaurus Tabs with shadcn/ui implementation
 */
export default function TabsComponent({ children, defaultValue }: TabsComponentProps) {
  // Extract TabItem children and their props
  const tabItems = React.Children.toArray(children).filter(
    (child): child is React.ReactElement<TabItemProps> =>
      React.isValidElement(child) && child.type === TabItem
  );

  if (tabItems.length === 0) {
    return null;
  }

  // Use first tab's value as default if not specified
  const firstValue = tabItems[0]?.props.value || 'tab-0';
  const activeDefault = defaultValue || firstValue;

  return (
    <Tabs defaultValue={activeDefault} className="w-full">
      <TabsList>
        {tabItems.map((item) => (
          <TabsTrigger key={item.props.value} value={item.props.value}>
            {item.props.label}
          </TabsTrigger>
        ))}
      </TabsList>
      {tabItems.map((item) => (
        <TabsContent key={item.props.value} value={item.props.value}>
          {item.props.children}
        </TabsContent>
      ))}
    </Tabs>
  );
}

// Export both for different import styles
export { TabsComponent as Tabs };
