/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank instance object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/core'

// Gets cached category tags
export async function get_categories() {
	return await invoke('plugin:tags|tags_get_categories')
}

// Gets cached loaders tags
export async function get_loaders() {
	return await invoke('plugin:tags|tags_get_loaders')
}

// Gets cached loaders tags an instance can use, with vanilla first
export async function get_modpack_loaders() {
	const loaders = await get_loaders()
	return loaders
		.filter((item) => item.supported_project_types.includes('modpack') || item.name === 'vanilla')
		.sort((a, b) => (a.name === 'vanilla' ? -1 : b.name === 'vanilla' ? 1 : 0))
}

// Gets cached game_versions tags
export async function get_game_versions() {
	return await invoke('plugin:tags|tags_get_game_versions')
}

// Gets cached donation_platforms tags
export async function get_donation_platforms() {
	return await invoke('plugin:tags|tags_get_donation_platforms')
}

// Gets cached licenses tags
export async function get_report_types() {
	return await invoke('plugin:tags|tags_get_report_types')
}
