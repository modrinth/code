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
  return await invoke('plugin:jre|jre_get_all_jre')
}

// Finds all the installation of Java 7, if it exists
// Returns [JavaVersion]
export async function find_jre_8_jres() {
  const jres = await invoke('plugin:jre|jre_get_all_jre')
  const version = '1.8'
  const allowHigher = false
  return await invoke('plugin:jre|jre_find_filtered_jres', { jres, version, allowHigher })
}

// Finds the installation of Java 17, if it exists
// Returns [JavaVersion]
export async function find_jre_17_jres() {
  const jres = await invoke('plugin:jre|jre_get_all_jre')
  const version = '1.17'
  const allowHigher = false
  return await invoke('plugin:jre|jre_find_filtered_jres', { jres, version, allowHigher })
}

// Finds the highest version of Java 18+, if it exists
// Returns [JavaVersion]
export async function find_jre_18plus_jres() {
  const jres = await invoke('plugin:jre|jre_get_all_jre')
  const version = '1.18'
  const allowHigher = true
  return await invoke('plugin:jre|jre_find_filtered_jres', { jres, version, allowHigher })
}

// Validates globals. Recommend directing the user to reassigned the globals if this returns false
// Returns [JavaVersion]
export async function validate_globals() {
  return await invoke('plugin:jre|jre_validate_globals')
}

// Gets java version from a specific path by trying to run 'java -version' on it.
// This also validates it, as it returns null if no valid java version is found at the path
export async function get_jre(path) {
  return await invoke('plugin:jre|jre_get_jre', { path })
}

// Tests JRE version by running 'java -version' on it.
// Returns true if the version is valid, and matches given (after extraction)
export async function test_jre(path, majorVersion, minorVersion) {
  return await invoke('plugin:jre|jre_test_jre', { path, majorVersion, minorVersion })
}

// Autodetect Java globals, by searching the users computer.
// Returns a *NEW* JavaGlobals that can be put into Settings
export async function autodetect_java_globals() {
  const java8 = await find_jre_8_jres()
  const java17 = await find_jre_17_jres()
  const java18plus = await find_jre_18plus_jres()
  return await invoke('plugin:jre|jre_autodetect_java_globals', { java8, java17, java18plus })
}

// Automatically installs specified java version
export async function auto_install_java(javaVersion) {
  return await invoke('plugin:jre|jre_auto_install_java', { javaVersion })
}

// Get max memory in KiB
export async function get_max_memory() {
  return await invoke('plugin:jre|jre_get_max_memory')
}
