/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank Profile object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/tauri'

// Installs pack from a version ID
export async function install(
  projectId: string,
  versionId: string,
  packTitle: string,
  packIcon?: string
): Promise<string> {
  return await invoke('pack_install_version_id', { projectId, versionId, packTitle, packIcon })
}

// Installs pack from a path
export async function install_from_file(path: string): Promise<string> {
  return await invoke('pack_install_file', { path })
}
