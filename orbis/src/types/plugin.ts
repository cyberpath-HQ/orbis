// Plugin types matching the Rust backend
export interface PluginInfo {
  id: string;
  name: string;
  version: string;
  description: string;
  state: 'Loaded' | 'Running' | 'Disabled' | 'Error';
}

export interface PluginPage {
  plugin: string;
  route: string;
  title: string;
  icon?: string;
  description?: string;
  show_in_menu: boolean;
  menu_order?: number;
  layout: ComponentSchema;
}

// UI Component Schema types
export type ComponentSchema = 
  | ContainerComponent
  | TextComponent
  | HeadingComponent
  | ButtonComponent
  | InputComponent
  | FormComponent
  | TableComponent
  | CardComponent
  | ListComponent
  | ImageComponent
  | IconComponent
  | LinkComponent
  | BadgeComponent
  | AlertComponent
  | ProgressComponent
  | TabsComponent
  | AccordionComponent
  | ModalComponent
  | DropdownComponent
  | TooltipComponent
  | GridComponent
  | FlexComponent
  | SpacerComponent
  | DividerComponent
  | CustomComponent;

export interface BaseComponent {
  id?: string;
  className?: string;
  style?: Record<string, string>;
  visible?: boolean | string; // boolean or expression
}

export interface ContainerComponent extends BaseComponent {
  type: 'Container';
  children: ComponentSchema[];
}

export interface TextComponent extends BaseComponent {
  type: 'Text';
  content: string;
  variant?: 'body' | 'caption' | 'label' | 'code';
}

export interface HeadingComponent extends BaseComponent {
  type: 'Heading';
  level: 1 | 2 | 3 | 4 | 5 | 6;
  text: string;
}

export interface ButtonComponent extends BaseComponent {
  type: 'Button';
  label: string;
  variant?: 'primary' | 'secondary' | 'danger' | 'ghost';
  size?: 'sm' | 'md' | 'lg';
  disabled?: boolean | string;
  onClick?: string; // Handler name
}

export interface InputComponent extends BaseComponent {
  type: 'Input';
  name: string;
  inputType?: 'text' | 'password' | 'email' | 'number' | 'date' | 'time' | 'datetime' | 'textarea' | 'checkbox' | 'radio' | 'select' | 'file';
  label?: string;
  placeholder?: string;
  required?: boolean;
  disabled?: boolean | string;
  defaultValue?: string;
  options?: { value: string; label: string }[]; // For select/radio
  validation?: {
    min?: number;
    max?: number;
    minLength?: number;
    maxLength?: number;
    pattern?: string;
    message?: string;
  };
}

export interface FormComponent extends BaseComponent {
  type: 'Form';
  onSubmit: string; // Handler name
  fields: InputComponent[];
  submitLabel?: string;
}

export interface TableColumn {
  key: string;
  header: string;
  sortable?: boolean;
  width?: string;
  render?: ComponentSchema;
}

export interface TableComponent extends BaseComponent {
  type: 'Table';
  columns: TableColumn[];
  dataSource: string; // Expression to get data
  rowKey?: string;
  pagination?: boolean;
  pageSize?: number;
}

export interface CardComponent extends BaseComponent {
  type: 'Card';
  title?: string;
  subtitle?: string;
  header?: ComponentSchema;
  content: ComponentSchema;
  footer?: ComponentSchema;
  hoverable?: boolean;
}

export interface ListComponent extends BaseComponent {
  type: 'List';
  dataSource: string;
  itemTemplate: ComponentSchema;
  emptyText?: string;
}

export interface ImageComponent extends BaseComponent {
  type: 'Image';
  src: string;
  alt?: string;
  width?: string;
  height?: string;
  fit?: 'contain' | 'cover' | 'fill' | 'none';
}

export interface IconComponent extends BaseComponent {
  type: 'Icon';
  name: string;
  size?: 'sm' | 'md' | 'lg';
  color?: string;
}

export interface LinkComponent extends BaseComponent {
  type: 'Link';
  href: string;
  text: string;
  external?: boolean;
}

export interface BadgeComponent extends BaseComponent {
  type: 'Badge';
  text: string;
  variant?: 'default' | 'success' | 'warning' | 'error' | 'info';
}

export interface AlertComponent extends BaseComponent {
  type: 'Alert';
  variant: 'info' | 'success' | 'warning' | 'error';
  title?: string;
  message: string;
  dismissible?: boolean;
}

export interface ProgressComponent extends BaseComponent {
  type: 'Progress';
  value: number | string;
  max?: number;
  showLabel?: boolean;
}

export interface TabItem {
  key: string;
  label: string;
  content: ComponentSchema;
  disabled?: boolean;
}

export interface TabsComponent extends BaseComponent {
  type: 'Tabs';
  items: TabItem[];
  defaultTab?: string;
}

export interface AccordionItem {
  key: string;
  title: string;
  content: ComponentSchema;
}

export interface AccordionComponent extends BaseComponent {
  type: 'Accordion';
  items: AccordionItem[];
  allowMultiple?: boolean;
}

export interface ModalComponent extends BaseComponent {
  type: 'Modal';
  trigger: ComponentSchema;
  title?: string;
  content: ComponentSchema;
  footer?: ComponentSchema;
  size?: 'sm' | 'md' | 'lg' | 'xl';
}

export interface DropdownItem {
  key: string;
  label: string;
  icon?: string;
  disabled?: boolean;
  onClick?: string;
}

export interface DropdownComponent extends BaseComponent {
  type: 'Dropdown';
  trigger: ComponentSchema;
  items: DropdownItem[];
}

export interface TooltipComponent extends BaseComponent {
  type: 'Tooltip';
  content: string;
  children: ComponentSchema;
  position?: 'top' | 'bottom' | 'left' | 'right';
}

export interface GridComponent extends BaseComponent {
  type: 'Grid';
  columns: number;
  gap?: string;
  children: ComponentSchema[];
}

export interface FlexComponent extends BaseComponent {
  type: 'Flex';
  direction?: 'row' | 'column';
  justify?: 'start' | 'end' | 'center' | 'between' | 'around';
  align?: 'start' | 'end' | 'center' | 'stretch';
  gap?: string;
  wrap?: boolean;
  children: ComponentSchema[];
}

export interface SpacerComponent extends BaseComponent {
  type: 'Spacer';
  size: 'xs' | 'sm' | 'md' | 'lg' | 'xl';
}

export interface DividerComponent extends BaseComponent {
  type: 'Divider';
  orientation?: 'horizontal' | 'vertical';
  label?: string;
}

export interface CustomComponent extends BaseComponent {
  type: 'Custom';
  component: string; // Registered custom component name
  props?: Record<string, unknown>;
}

// API Response types
export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: {
    code: string;
    message: string;
  };
}

export interface PluginListResponse {
  plugins: PluginInfo[];
  count: number;
}

export interface PluginPagesResponse {
  pages: PluginPage[];
  count: number;
}

// App mode types
export interface AppModeInfo {
  mode: 'standalone' | 'client' | 'server';
  is_standalone: boolean;
  is_client: boolean;
  is_server: boolean;
}

export interface ProfileInfo {
  name: string;
  server_url?: string;
}

export interface ProfileListResponse {
  profiles: {
    name: string;
    is_active: boolean;
    is_default: boolean;
  }[];
  active: string;
}
