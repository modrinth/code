import { invoke } from '@tauri-apps/api/tauri'
import { Manifest, VersionManifest } from './manifest'

/// Gets the game versions from daedalus
export async function get_game_versions(): Promise<VersionManifest> {
  return await invoke('metadata_get_game_versions')
}

// Gets the fabric versions from daedalus
export async function get_fabric_versions(): Promise<Manifest> {
  return await invoke('metadata_get_fabric_versions')
}

// Gets the forge versions from daedalus
export async function get_forge_versions(): Promise<Manifest> {
  return await invoke('metadata_get_forge_versions')
}

// Gets the quilt versions from daedalus
export async function get_quilt_versions(): Promise<Manifest> {
  return await invoke('metadata_get_quilt_versions')
}
