import { invoke } from '@tauri-apps/api/core'

/// Gets the game versions from daedalus
// Returns a VersionManifest
export async function get_game_versions() {
  return await invoke('plugin:metadata|metadata_get_game_versions')
}

// Gets the given loader versions from daedalus
// Returns Manifest
export async function get_loader_versions(loader) {
  return await invoke('plugin:metadata|metadata_get_loader_versions', { loader })
}
