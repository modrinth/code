import type { Labrinth } from '@modrinth/api-client'
import { invoke } from '@tauri-apps/api/core'

// Converts user profile links from rendered Markdown/any dynamic content into app routes.
export function parse_modrinth_user_link(href: string): string | null {
	try {
		const url = new URL(href)
		if (url.hostname !== 'modrinth.com' && url.hostname !== 'www.modrinth.com') return null

		const segments = url.pathname.split('/').filter(Boolean)
		if (segments[0]?.toLowerCase() !== 'user' || !segments[1] || segments.length > 3) return null

		const path = `/user/${encodeURIComponent(decodeURIComponent(segments[1]))}`
		return segments[2] ? `${path}/${encodeURIComponent(decodeURIComponent(segments[2]))}` : path
	} catch {
		return null
	}
}

export async function search_user(query: string): Promise<Labrinth.Users.v3.SearchUser[]> {
	return await invoke<Labrinth.Users.v3.SearchUser[]>('plugin:users|search_user', { query })
}

export async function get_user_profile(userId: string): Promise<Labrinth.Users.v3.User> {
	return await invoke<Labrinth.Users.v3.User>('plugin:users|get_user_profile', { userId })
}

export async function get_user_projects(userId: string): Promise<Labrinth.Projects.v3.Project[]> {
	return await invoke<Labrinth.Projects.v3.Project[]>('plugin:users|get_user_projects', {
		userId,
	})
}

export async function get_user_organizations(
	userId: string,
): Promise<Labrinth.Organizations.v3.Organization[]> {
	return await invoke<Labrinth.Organizations.v3.Organization[]>(
		'plugin:users|get_user_organizations',
		{ userId },
	)
}

export async function get_user_collections(
	userId: string,
): Promise<Labrinth.Collections.Collection[]> {
	return await invoke<Labrinth.Collections.Collection[]>('plugin:users|get_user_collections', {
		userId,
	})
}

export async function patch_user(
	userId: string,
	patch: Partial<Pick<Labrinth.Users.v2.User, 'badges' | 'bio' | 'role' | 'username'>>,
): Promise<void> {
	await invoke('plugin:users|patch_user', { userId, patch })
}

export async function change_user_avatar(
	userId: string,
	image: Uint8Array,
	extension: string,
): Promise<void> {
	await invoke('plugin:users|change_user_avatar', { userId, image, extension })
}

export async function delete_user_avatar(userId: string): Promise<void> {
	await invoke('plugin:users|delete_user_avatar', { userId })
}

export async function block_user(userId: string): Promise<void> {
	await invoke('plugin:users|block_user', { userId })
}

export async function unblock_user(userId: string): Promise<void> {
	await invoke('plugin:users|unblock_user', { userId })
}

export async function get_blocked_users(): Promise<Labrinth.BlockedUsers.v3.BlockedUserId[]> {
	return await invoke<Labrinth.BlockedUsers.v3.BlockedUserId[]>('plugin:users|get_blocked_users')
}
