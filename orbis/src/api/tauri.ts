import { invoke } from '@tauri-apps/api/core';
import type {
  AppModeInfo,
  ProfileInfo,
  ProfileListResponse,
  PluginListResponse,
  PluginPagesResponse,
} from '../types';

/**
 * Health check - verifies backend is working
 */
export async function healthCheck(): Promise<{ status: string; mode: string; timestamp: string }> {
  return invoke('health_check');
}

/**
 * Get current application mode
 */
export async function getMode(): Promise<AppModeInfo> {
  return invoke('get_mode');
}

/**
 * Get active profile
 */
export async function getProfile(): Promise<ProfileInfo> {
  return invoke('get_profile');
}

/**
 * List all profiles
 */
export async function listProfiles(): Promise<ProfileListResponse> {
  return invoke('list_profiles');
}

/**
 * Switch to a different profile
 */
export async function switchProfile(name: string): Promise<{ success: boolean; message: string }> {
  return invoke('switch_profile', { name });
}

/**
 * Get list of loaded plugins
 */
export async function getPlugins(): Promise<PluginListResponse> {
  return invoke('get_plugins');
}

/**
 * Get plugin pages for UI rendering
 */
export async function getPluginPages(): Promise<PluginPagesResponse> {
  return invoke('get_plugin_pages');
}
