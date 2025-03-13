import { invoke } from '@tauri-apps/api/core'

export async function getProfileWorlds(path) {
  return await invoke('plugin:worlds|get_profile_worlds', { path })
}

export async function getServerStatus(address) {
  return await invoke('plugin:worlds|get_server_status', { address })
}
