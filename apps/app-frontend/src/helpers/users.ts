import { invoke } from '@tauri-apps/api/core'

export type SearchUser = {
	id: string
	username: string
	avatar_url: string | null
}

export async function search_user(query: string): Promise<SearchUser[]> {
	return await invoke<SearchUser[]>('plugin:users|search_user', { query })
}
