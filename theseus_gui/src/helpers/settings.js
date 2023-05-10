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

// An example test function for getting/setting settings
export async function test() {
  // First, print settings and store them to an object
  let settings = await get()
  console.log(JSON.stringify(settings))

  // Then set some random settings in that object
  settings.java_8_path = '/example/path'

  // Set the new settings object
  await set(settings)
  console.log(JSON.stringify(await get()))
}

// Get full settings object
export async function get() {
  return await invoke('settings_get')
}

// Set full settings object
export async function set(settings) {
  return await invoke('settings_set', { settings })
}
