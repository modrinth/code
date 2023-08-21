import { invoke } from '@tauri-apps/api/tauri'

/// Gets the game versions from daedalus
// Returns a VersionManifest
export async function get_game_versions() {
  return await invoke('plugin:metadata|metadata_get_game_versions')
}

// Gets the fabric versions from daedalus
// Returns Manifest
export async function get_fabric_versions() {
  const c = await invoke('plugin:metadata|metadata_get_fabric_versions')
  console.log('Getting fabric versions', c)
  return c
}

// Gets the forge versions from daedalus
// Returns Manifest
export async function get_forge_versions() {
  const c = await invoke('plugin:metadata|metadata_get_forge_versions')
  console.log('Getting forge versions', c)
  return c
}

// Gets the quilt versions from daedalus
// Returns Manifest
export async function get_quilt_versions() {
  const c = await invoke('plugin:metadata|metadata_get_quilt_versions')
  console.log('Getting quilt versions', c)
  return c
}

// Gets the neoforge versions from daedalus
// Returns Manifest
export async function get_neoforge_versions() {
  const c = await invoke('plugin:metadata|metadata_get_neoforge_versions')
  console.log('Getting neoforge versions', c)
  return c
}
