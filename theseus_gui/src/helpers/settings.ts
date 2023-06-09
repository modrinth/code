/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank Profile object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/tauri'
import { JavaGlobals, JavaVersion } from './jre'

export type WindowSize = [number, number]

export interface Hooks {
  pre_launch?: string
  wrapper?: string
  post_exit?: string
}

export interface JavaSettings {
  override_version?: JavaVersion
  extra_arguments?: string[]
  custom_env_args?: [string, string][]
}

export interface MemorySettings {
  maximum: number
}

export interface Settings {
  memory: MemorySettings
  game_resolution: WindowSize
  custom_java_args: string[]
  custom_env_args: [string, string][]
  java_globals: JavaGlobals
  /** Uuid string (can be null) */
  default_user?: string
  hooks: Hooks
  max_concurrent_downloads: number
  version: number
  collapsed_navigation: boolean
}

// Get full settings object
export async function get(): Promise<Settings> {
  return await invoke('settings_get')
}

// Set full settings object
export async function set(settings: Settings) {
  return await invoke('settings_set', { settings })
}
