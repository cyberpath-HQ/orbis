import type {
    ComponentSchema,
    StateFieldDefinition,
    Action,
    PageLifecycleHooks
} from './schema';

// Plugin types matching the Rust backend
export interface PluginInfo {
    id:          string
    name:        string
    version:     string
    description: string
    state:       `Loaded` | `Running` | `Disabled` | `Error`
}

export interface PluginPage {
    plugin:        string
    route:         string
    title:         string
    icon?:         string
    description?:  string
    show_in_menu:  boolean
    menu_order?:   number
    sections:      Array<ComponentSchema>
    state?:        Record<string, StateFieldDefinition>
    computed?:     Record<string, string>
    actions?:      Record<string, Action>
    hooks?:        PageLifecycleHooks
    dialogs?:      Array<{
        id:           string
        title?:       string
        description?: string
        content:      ComponentSchema
        footer?:      ComponentSchema
        size?:        `sm` | `md` | `lg` | `xl` | `full`
    }>
    requires_auth: boolean
    permissions?:  Array<string>
    roles?:        Array<string>
}

// API Response types
export interface ApiResponse<T> {
    success: boolean
    data?:   T
    error?: {
        code:    string
        message: string
    }
}

export interface PluginListResponse {
    plugins: Array<PluginInfo>
    count:   number
}

export interface PluginPagesResponse {
    pages: Array<PluginPage>
    count: number
}

// App mode types
export interface AppModeInfo {
    mode:          `standalone` | `client` | `server`
    is_standalone: boolean
    is_client:     boolean
    is_server:     boolean
}

export interface ProfileInfo {
    name:        string
    server_url?: string
}

export interface ProfileListResponse {
    profiles: Array<{
        name:       string
        is_active:  boolean
        is_default: boolean
    }>
    active: string
}
