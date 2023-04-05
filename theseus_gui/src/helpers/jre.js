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

// Finds the installation of Java 7, if it exists
// Returns a JavaVersion
export async function find_jre_8() {
  return await invoke('jre_find_jre_8')
}

// Finds the highest version of Java 17+, if it exists
// Returns a JavaVersion
export async function find_jre_17plus() {
  return await invoke('jre_find_jre_17plus')
}

/// From a Path to a profile, returns the JavaVersion of the optimal JRE to use
/// Returns an error if the profile is not managed by Theseus, or if the optimal JRE could not be detected
/// If successful, returns the optimal JavaVersion
export async function detect_optimal_jre(path) {
  return await invoke('jre_detect_optimal_jre', { path })
}

/// Get all allowed JREs for a given game version that exist on the system
/// Returns an array of JavaVersion
export async function get_all_allowable_jre(path) {
  return await invoke('jre_get_all_allowable_jre', { path })
}
