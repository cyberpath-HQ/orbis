import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  AppModeInfo,
  ProfileInfo,
  ProfileListResponse,
  PluginListResponse,
  PluginPagesResponse,
} from '../types';

// ============================================================================
// Error Handling & Retry Logic
// ============================================================================

/** API error with structured information */
export class ApiError extends Error {
  constructor(
    message: string,
    public readonly code: string = 'UNKNOWN',
    public readonly retryable: boolean = false,
    public readonly cause?: unknown
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

/** Error codes for categorization */
export const ErrorCodes = {
  NETWORK: 'NETWORK_ERROR',
  TIMEOUT: 'TIMEOUT_ERROR',
  AUTH: 'AUTH_ERROR',
  VALIDATION: 'VALIDATION_ERROR',
  NOT_FOUND: 'NOT_FOUND',
  SERVER: 'SERVER_ERROR',
  CANCELLED: 'CANCELLED',
  UNKNOWN: 'UNKNOWN',
} as const;

/** Retry configuration */
interface RetryConfig {
  maxRetries: number;
  baseDelay: number;
  maxDelay: number;
  backoffMultiplier: number;
}

const DEFAULT_RETRY_CONFIG: RetryConfig = {
  maxRetries: 3,
  baseDelay: 500,
  maxDelay: 10000,
  backoffMultiplier: 2,
};

/** Calculate exponential backoff delay */
function calculateBackoff(attempt: number, config: RetryConfig): number {
  const delay = config.baseDelay * Math.pow(config.backoffMultiplier, attempt);
  return Math.min(delay, config.maxDelay);
}

/** Sleep for given milliseconds */
function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

/** Parse error and determine if retryable */
function parseError(error: unknown): ApiError {
  if (error instanceof ApiError) {
    return error;
  }

  const message = error instanceof Error ? error.message : String(error);

  // Check for known error patterns
  if (message.includes('network') || message.includes('connection')) {
    return new ApiError(message, ErrorCodes.NETWORK, true, error);
  }
  if (message.includes('timeout')) {
    return new ApiError(message, ErrorCodes.TIMEOUT, true, error);
  }
  if (message.includes('unauthorized') || message.includes('not authenticated')) {
    return new ApiError(message, ErrorCodes.AUTH, false, error);
  }
  if (message.includes('not found')) {
    return new ApiError(message, ErrorCodes.NOT_FOUND, false, error);
  }
  if (message.includes('cancelled') || message.includes('aborted')) {
    return new ApiError(message, ErrorCodes.CANCELLED, false, error);
  }

  return new ApiError(message, ErrorCodes.UNKNOWN, false, error);
}

/** Error interceptor callback */
type ErrorInterceptor = (error: ApiError) => void | Promise<void>;

/** Global error interceptors */
const errorInterceptors: Set<ErrorInterceptor> = new Set();

/** Register an error interceptor */
export function onApiError(interceptor: ErrorInterceptor): () => void {
  errorInterceptors.add(interceptor);
  return () => errorInterceptors.delete(interceptor);
}

/** Notify all error interceptors */
async function notifyErrorInterceptors(error: ApiError): Promise<void> {
  for (const interceptor of errorInterceptors) {
    try {
      await interceptor(error);
    } catch (e) {
      console.error('Error interceptor failed:', e);
    }
  }
}

/** Invoke with retry logic */
async function invokeWithRetry<T>(
  command: string,
  args?: Record<string, unknown>,
  config: Partial<RetryConfig> = {}
): Promise<T> {
  const retryConfig = { ...DEFAULT_RETRY_CONFIG, ...config };
  let lastError: ApiError;

  for (let attempt = 0; attempt <= retryConfig.maxRetries; attempt++) {
    try {
      return await invoke<T>(command, args);
    } catch (error) {
      lastError = parseError(error);

      // Don't retry non-retryable errors
      if (!lastError.retryable || attempt === retryConfig.maxRetries) {
        await notifyErrorInterceptors(lastError);
        throw lastError;
      }

      // Wait before retrying
      const delay = calculateBackoff(attempt, retryConfig);
      console.warn(`Command ${command} failed, retrying in ${delay}ms (attempt ${attempt + 1}/${retryConfig.maxRetries})`);
      await sleep(delay);
    }
  }

  // This should never be reached, but TypeScript needs it
  throw lastError!;
}

// ============================================================================
// Request Cancellation Support
// ============================================================================

/** Create a cancellable request wrapper */
export function createCancellableRequest<T>(
  commandFn: () => Promise<T>
): { promise: Promise<T>; cancel: () => void } {
  let cancelled = false;

  const promise = new Promise<T>((resolve, reject) => {
    commandFn()
      .then(result => {
        if (cancelled) {
          reject(new ApiError('Request cancelled', ErrorCodes.CANCELLED, false));
        } else {
          resolve(result);
        }
      })
      .catch(error => {
        if (cancelled) {
          reject(new ApiError('Request cancelled', ErrorCodes.CANCELLED, false));
        } else {
          reject(error);
        }
      });
  });

  return {
    promise,
    cancel: () => {
      cancelled = true;
    },
  };
}

// ============================================================================
// Core API Functions
// ============================================================================

/**
 * Health check - verifies backend is working
 */
export async function healthCheck(): Promise<{ status: string; mode: string; timestamp: string }> {
  return invokeWithRetry('health_check');
}

/**
 * Get current application mode
 */
export async function getMode(): Promise<AppModeInfo> {
  return invokeWithRetry('get_mode');
}

// ============================================================================
// Profile Management
// ============================================================================

/**
 * Get active profile
 */
export async function getProfile(): Promise<ProfileInfo> {
  return invokeWithRetry('get_profile');
}

/**
 * List all profiles
 */
export async function listProfiles(): Promise<ProfileListResponse> {
  return invokeWithRetry('list_profiles');
}

/**
 * Create a new profile
 */
export async function createProfile(
  name: string,
  mode: 'standalone' | 'client',
  serverUrl?: string
): Promise<{ success: boolean; message: string }> {
  return invokeWithRetry('create_profile', { name, mode, serverUrl });
}

/**
 * Delete a profile
 */
export async function deleteProfile(name: string): Promise<{ success: boolean; message: string }> {
  return invokeWithRetry('delete_profile', { name });
}

/**
 * Switch to a different profile
 */
export async function switchProfile(name: string): Promise<{ success: boolean; message: string }> {
  return invokeWithRetry('switch_profile', { name });
}

// ============================================================================
// Plugin Management
// ============================================================================

/**
 * Get list of loaded plugins
 */
export async function getPlugins(): Promise<PluginListResponse> {
  return invokeWithRetry('get_plugins');
}

/**
 * Get plugin pages for UI rendering
 */
export async function getPluginPages(): Promise<PluginPagesResponse> {
  return invokeWithRetry('get_plugin_pages');
}

/**
 * Get detailed plugin information
 */
export async function getPluginInfo(name: string): Promise<{
  id: string;
  name: string;
  version: string;
  description?: string;
  author?: string;
  license?: string;
  state: string;
  loaded_at: string;
  permissions: string[];
  routes_count: number;
  pages_count: number;
}> {
  return invokeWithRetry('get_plugin_info', { name });
}

/**
 * Reload a plugin
 */
export async function reloadPlugin(name: string): Promise<{ success: boolean; message: string }> {
  return invokeWithRetry('reload_plugin', { name });
}

/**
 * Enable a plugin
 */
export async function enablePlugin(name: string): Promise<{ success: boolean; message: string }> {
  return invokeWithRetry('enable_plugin', { name });
}

/**
 * Disable a plugin
 */
export async function disablePlugin(name: string): Promise<{ success: boolean; message: string }> {
  return invokeWithRetry('disable_plugin', { name });
}

/**
 * Install a plugin from path
 */
export async function installPlugin(path: string): Promise<{ success: boolean; message: string }> {
  return invokeWithRetry('install_plugin', { path });
}

/**
 * Uninstall a plugin
 */
export async function uninstallPlugin(name: string): Promise<{ success: boolean; message: string }> {
  return invokeWithRetry('uninstall_plugin', { name });
}

// ============================================================================
// Plugin Watcher
// ============================================================================

/** Plugin change event payload */
export interface PluginChangeEvent {
  kind: 'Added' | 'Modified' | 'Removed';
  path: string;
  plugin_id?: string;
}

/**
 * Start watching the plugins directory for changes
 */
export async function startPluginWatcher(): Promise<{ success: boolean; message: string }> {
  return invokeWithRetry('start_plugin_watcher');
}

/**
 * Stop watching the plugins directory
 */
export async function stopPluginWatcher(): Promise<{ success: boolean; message: string }> {
  return invokeWithRetry('stop_plugin_watcher');
}

/**
 * Listen for plugin change events
 */
export async function onPluginChange(
  callback: (event: PluginChangeEvent) => void
): Promise<UnlistenFn> {
  return listen<PluginChangeEvent>('plugin-changed', event => {
    callback(event.payload);
  });
}

// ============================================================================
// Authentication
// ============================================================================

/** Login response */
export interface LoginResponse {
  success: boolean;
  message: string;
  session?: {
    user_id: string;
    username: string;
    email?: string;
    roles: string[];
    is_admin: boolean;
    token: string;
    refresh_token?: string;
    expires_at: string;
  };
}

/** Session info */
export interface SessionInfo {
  authenticated: boolean;
  user_id?: string;
  username?: string;
  email?: string;
  roles: string[];
  is_admin: boolean;
  expires_at?: string;
}

/**
 * Login with credentials
 */
export async function login(username: string, password: string): Promise<LoginResponse> {
  // Don't retry auth requests - they should fail fast
  return invoke('login', { username, password });
}

/**
 * Logout current session
 */
export async function logout(): Promise<{ success: boolean; message: string }> {
  return invoke('logout');
}

/**
 * Get current session
 */
export async function getSession(): Promise<SessionInfo> {
  return invokeWithRetry('get_session');
}

/**
 * Verify current session is valid
 */
export async function verifySession(): Promise<{ valid: boolean; expires_at?: string }> {
  return invokeWithRetry('verify_session');
}
