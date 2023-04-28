/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank Profile object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/tauri'

// Gets tag bundle of all tags
export async function get_tag_bundle() {
  return await invoke('tags_get_tag_bundle')
}

// Gets cached category tags
export async function get_categories() {
  return await invoke('tags_get_categories')
}

// Gets cached loaders tags
export async function get_loaders() {
  return await invoke('tags_get_loaders')
}

// Gets cached game_versions tags
export async function get_game_versions() {
  return await invoke('tags_get_game_versions')
}

// Gets cached donation_platforms tags
export async function get_donation_platforms() {
  return await invoke('tags_get_donation_platforms')
}

// Gets cached licenses tags
export async function get_report_types() {
  return await invoke('tags_get_report_types')
}
