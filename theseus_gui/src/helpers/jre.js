/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank Profile object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/tauri'

/*

JavaVersion {
    path: Path
    version: String
}
 
*/

/// Get all JREs that exist on the system
// Returns an array of JavaVersion
export async function get_all_jre() {
  return await invoke('jre_get_all_jre')
}

// Finds all the installation of Java 7, if it exists
// Returns [JavaVersion]
export async function find_jre_8_jres() {
  return await invoke('jre_find_jre_8_jres')
}

// Finds the installation of Java 17, if it exists
// Returns [JavaVersion]
export async function find_jre_17_jres() {
  return await invoke('jre_find_jre_17_jres')
}

// Finds the highest version of Java 18+, if it exists
// Returns [JavaVersion]
export async function find_jre_18plus_jres() {
  return await invoke('jre_find_jre_18plus_jres')
}

// Validates globals. Recommend directing the user to reassigned the globals if this returns false
// Returns [JavaVersion]
export async function validate_globals() {
  return await invoke('jre_validate_globals')
}

// Gets key for the optimal JRE to use, for a given profile path
// The key can be used in the hashmap contained by JavaGlobals in Settings (if it exists)
export async function get_optimal_jre_key_by_path(path) {
  return await invoke('jre_get_optimal_jre_key_by_path', { path })
}

// Gets key for the optimal JRE to use, for a given profile
// The key can be used in the hashmap contained by JavaGlobals in Settings (if it exists)
export async function get_optimal_jre_ke(path) {
  return await invoke('jre_get_optimal_jre_key', { path })
}

// Autodetect Java globals, by searching the users computer.
// Returns a *NEW* JavaGlobals that can be put into Settings
export async function autodetect_java_globals(path) {
  return await invoke('jre_autodetect_java_globals', { path })
}
