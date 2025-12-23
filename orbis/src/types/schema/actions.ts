/**
 * Action definitions for the JSON UI Schema event system
 */

import type { Expression } from './base';

// Action types that can be executed in response to events
export type ActionType =
    | `update_state`
    | `call_api`
    | `navigate`
    | `show_toast`
    | `show_dialog`
    | `close_dialog`
    | `debounced_action`
    | `validate_form`
    | `reset_form`
    | `set_loading`
    | `download`
    | `copy`
    | `open_url`
    | `emit`
    | `conditional`
    | `sequence`;

// Base action interface
export interface BaseAction {
    type: ActionType
}

// Update state action - modifies page state
export interface UpdateStateAction extends BaseAction {
    type:   `update_state`
    path:   string
    value?: unknown
    from?:  Expression

    /**
     * If true, merges the value with existing state at the path
     * @deprecated Use `mode: 'merge'` instead
     */
    merge?: boolean
    mode?:  `set` | `merge` | `append` | `prepend` | `remove`
}

// API call action - calls a backend API
export interface CallApiAction extends BaseAction {
    type:             `call_api`
    name?:            string
    api:              string
    method?:          `GET` | `POST` | `PUT` | `PATCH` | `DELETE`
    args_from_state?: Array<string>
    map_args?:       Array<{ from: string
        to:                        string }>
    body?:       Expression | Record<string, unknown>
    headers?:    Record<string, string>
    on_success?: Array<Action>
    on_error?:   Array<Action>
    on_finally?: Array<Action>
}

// Navigate action - changes the current route
export interface NavigateAction extends BaseAction {
    type:     `navigate`
    to:       Expression
    replace?: boolean
    params?:  Record<string, Expression>
}

// Toast notification action
export interface ShowToastAction extends BaseAction {
    type:      `show_toast`
    level:     `info` | `success` | `warning` | `error`
    message:   Expression
    title?:    Expression
    duration?: number
}

// Show dialog action
export interface ShowDialogAction extends BaseAction {
    type:     `show_dialog`
    dialogId: string
    data?:    Record<string, Expression>
}

// Close dialog action
export interface CloseDialogAction extends BaseAction {
    type:      `close_dialog`
    dialogId?: string
}

// Debounced action - delays execution
export interface DebouncedAction extends BaseAction {
    type:    `debounced_action`
    delayMs: number
    action:  Action
    key?:    string
}

// Validate form action
export interface ValidateFormAction extends BaseAction {
    type:       `validate_form`
    formId:     string
    onValid?:   Array<Action>
    onInvalid?: Array<Action>
}

// Reset form action
export interface ResetFormAction extends BaseAction {
    type:   `reset_form`
    formId: string
}

// Set loading action
export interface SetLoadingAction extends BaseAction {
    type:    `set_loading`
    loading: boolean
    target?: string
}

// Download action
export interface DownloadAction extends BaseAction {
    type:      `download`
    url:       Expression
    filename?: Expression
}

// Copy to clipboard action
export interface CopyAction extends BaseAction {
    type:              `copy`
    text:              Expression
    showNotification?: boolean
}

// Open external URL action
export interface OpenUrlAction extends BaseAction {
    type:    `open_url`
    url:     Expression
    newTab?: boolean
}

// Emit custom event action
export interface EmitAction extends BaseAction {
    type:     `emit`
    event:    string
    payload?: Record<string, Expression>
}

// Conditional action
export interface ConditionalAction extends BaseAction {
    type:      `conditional`
    condition: Expression
    then:      Array<Action>
    else?:     Array<Action>
}

// Sequence action - runs actions in order
export interface SequenceAction extends BaseAction {
    type:         `sequence`
    actions:      Array<Action>
    stopOnError?: boolean
}

// Union of all action types
export type Action =
    | UpdateStateAction
    | CallApiAction
    | NavigateAction
    | ShowToastAction
    | ShowDialogAction
    | CloseDialogAction
    | DebouncedAction
    | ValidateFormAction
    | ResetFormAction
    | SetLoadingAction
    | DownloadAction
    | CopyAction
    | OpenUrlAction
    | EmitAction
    | ConditionalAction
    | SequenceAction;

// Event handler types
export interface EventHandlers {
    on_click?:            Array<Action>
    on_change?:           Array<Action>
    on_submit?:           Array<Action>
    on_focus?:            Array<Action>
    on_blur?:             Array<Action>
    on_key_down?:         Array<Action>
    on_key_up?:           Array<Action>
    on_mouse_enter?:      Array<Action>
    on_mouse_leave?:      Array<Action>
    on_double_click?:     Array<Action>
    on_row_click?:        Array<Action>
    on_row_double_click?: Array<Action>
    on_select?:           Array<Action>
    on_clear?:            Array<Action>
    on_search?:           Array<Action>
    on_page_change?:      Array<Action>
    on_sort_change?:      Array<Action>
    on_filter_change?:    Array<Action>
    on_load?:             Array<Action>
    on_error?:            Array<Action>
    on_close?:            Array<Action>
    on_open?:             Array<Action>
}
