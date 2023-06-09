/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank Profile object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/tauri'

export interface JavaVersion {
  path: string
  version: string
}

export type JavaGlobals = Map<string, JavaVersion>

/** Get all JREs that exist on the system */
export async function get_all_jre(): Promise<JavaVersion[]> {
  return await invoke('jre_get_all_jre')
}

/** Finds all the installation of Java 7, if it exists */
export async function find_jre_8_jres(): Promise<JavaVersion[]> {
  return await invoke('jre_find_jre_8_jres')
}

/** Finds the installation of Java 17, if it exists */
export async function find_jre_17_jres(): Promise<JavaVersion[]> {
  return await invoke('jre_find_jre_17_jres')
}

/** Finds the highest version of Java 18+, if it exists */
export async function find_jre_18plus_jres(): Promise<JavaVersion[]> {
  return await invoke('jre_find_jre_18plus_jres')
}

/** Validates globals. Recommend directing the user to reassigned the globals if this returns false */
export async function validate_globals(): Promise<boolean> {
  return await invoke('jre_validate_globals')
}

// Gets java version from a specific path by trying to run 'java -version' on it.
// This also validates it, as it returns null if no valid java version is found at the path
export async function get_jre(path: string) {
  return await invoke('jre_get_jre', { path })
}

// Autodetect Java globals, by searching the users computer.
// Returns a *NEW* JavaGlobals that can be put into Settings
export async function autodetect_java_globals(path: string): Promise<JavaGlobals> {
  return await invoke('jre_autodetect_java_globals', { path })
}

// Automatically installs specified java version
export async function jre_auto_install_java(javaVersion: string) {
  return await invoke('jre_auto_install_java', { javaVersion })
}

// Get max memory in KiB
export async function get_max_memory(): Promise<number> {
  return await invoke('jre_get_max_memory')
}
