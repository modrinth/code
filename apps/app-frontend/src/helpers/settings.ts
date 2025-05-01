/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank Profile object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/core'
import type { ColorTheme, FeatureFlag } from '@/store/theme.ts'
import type { Hooks, MemorySettings, WindowSize } from '@/helpers/types'

// Settings object
/*

Settings {
    "memory": MemorySettings,
    "game_resolution": [int int],
    "custom_java_args": [String ...],
    "custom_env_args" : [(string, string) ... ]>,
    "java_globals": Hash of (string, Path),
    "default_user": Uuid string (can be null),
    "hooks": Hooks,
    "max_concurrent_downloads": uint,
    "version": u32,
    "collapsed_navigation": bool,
}

Memorysettings {
    "min": u32, can be null,
    "max": u32,
}

*/

export type AppSettings = {
  max_concurrent_downloads: number
  max_concurrent_writes: number

  theme: ColorTheme
  default_page: 'home' | 'library'
  collapsed_navigation: boolean
  advanced_rendering: boolean
  native_decorations: boolean
  toggle_sidebar: boolean

  telemetry: boolean
  discord_rpc: boolean
  personalized_ads: boolean

  onboarded: boolean

  extra_launch_args: string[]
  custom_env_vars: [string, string][]
  memory: MemorySettings
  force_fullscreen: boolean
  game_resolution: WindowSize
  hide_on_process_start: boolean
  hooks: Hooks

  custom_dir?: string | null
  prev_custom_dir?: string | null
  migrated: boolean

  developer_mode: boolean
  feature_flags: Record<FeatureFlag, boolean>
}

// Get full settings object
export async function get() {
  return (await invoke('plugin:settings|settings_get')) as AppSettings
}

// Set full settings object
export async function set(settings: AppSettings) {
  return await invoke('plugin:settings|settings_set', { settings })
}

export async function cancel_directory_change(): Promise<void> {
  return await invoke('plugin:settings|cancel_directory_change')
}
