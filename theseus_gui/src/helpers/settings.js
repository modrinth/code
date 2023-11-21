/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank Profile object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/tauri'

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

// Get full settings object
export async function get() {
  return await invoke('plugin:settings|settings_get')
}

// Set full settings object
export async function set(settings) {
  return await invoke('plugin:settings|settings_set', { settings })
}

// Changes the config dir
// Seizes the entire application state until its done
export async function change_config_dir(newConfigDir) {
  return await invoke('plugin:settings|settings_change_config_dir', { newConfigDir })
}

export async function is_dir_writeable(newConfigDir) {
  return await invoke('plugin:settings|settings_is_dir_writeable', { newConfigDir })
}
