import { invoke } from '@tauri-apps/api/core'

export async function search_user(query) {
	return await invoke('plugin:users|search_user', { query })
}
