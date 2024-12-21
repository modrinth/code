import { invoke } from '@tauri-apps/api/core'

export async function friends() {
  return await invoke('plugin:friends|friends')
}

export async function friend_statuses() {
  return await invoke('plugin:friends|friend_statuses')
}

export async function add_friend(userId) {
  return await invoke('plugin:friends|add_friend', { userId })
}

export async function remove_friend(userId) {
  return await invoke('plugin:friends|remove_friend', { userId })
}
