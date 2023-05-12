import { invoke } from '@tauri-apps/api/tauri'

/// Gets the game versions from daedalus
// Returns a VersionManifest
export async function get_game_versions() {
  return await invoke('metadata_get_game_versions')
}

// Gets the fabric versions from daedalus
// Returns Manifest
export async function get_fabric_versions() {
  return await invoke('metadata_get_fabric_versions')
}

// Gets the forge versions from daedalus
// Returns Manifest
export async function get_forge_versions() {
  return await invoke('metadata_get_forge_versions')
}

// Gets the quilt versions from daedalus
// Returns Manifest
export async function get_quilt_versions() {
  return await invoke('metadata_get_quilt_versions')
}
