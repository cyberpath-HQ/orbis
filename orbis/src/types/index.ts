// Re-export plugin types (legacy) - explicitly list all exports to avoid conflicts
export type {
    PluginInfo,
    PluginPage,
    AppModeInfo,
    ProfileInfo,
    ProfileListResponse,
    PluginListResponse,
    PluginPagesResponse,
    ApiResponse,
    BaseComponent,
    ContainerComponent,
    TextComponent,
    HeadingComponent,
    ButtonComponent,
    InputComponent,
    FormComponent,
    TableColumn as LegacyTableColumn,
    TableComponent,
    CardComponent,
    ListComponent,
    ImageComponent,
    IconComponent as LegacyIconComponent,
    LinkComponent,
    BadgeComponent,
    AlertComponent,
    ProgressComponent,
    TabItem as LegacyTabItem,
    TabsComponent,
    AccordionItem as LegacyAccordionItem,
    AccordionComponent,
    ModalComponent,
    DropdownItem as LegacyDropdownItem,
    DropdownComponent,
    TooltipComponent,
    GridComponent,
    FlexComponent,
    SpacerComponent,
    DividerComponent,
    CustomComponent
} from './plugin';

// Re-export ComponentSchema from plugin with different name to avoid conflict
export type { ComponentSchema as LegacyComponentSchema } from './plugin';

// Re-export all schema types (these are the new types for the UI system)
export * from './schema';
