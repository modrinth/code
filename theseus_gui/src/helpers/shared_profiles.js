/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank Profile object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/tauri'

/// Created shared modpack from profile
export async function share_create(path) {
  return await invoke('plugin:profile_share|profile_share_create', { path })
}

/// Generates a shared profile link
export async function share_generate(path) {
  return await invoke('plugin:profile_share|profile_share_generate_share_link', { path })
}

/// Accepts a shared profile link
export async function share_accept(link) {
  return await invoke('plugin:profile_share|profile_share_accept', { link })
}

/// Install a pack from a shared profile id
export async function share_install(id) {
  return await invoke('plugin:profile_share|profile_share_install', { id })
}

// get all user profiles that are available to the currentt user
export async function get_all(path) {
  return await invoke('plugin:profile_share|profile_share_get_all', { path })
}

// syncs profile to match that on server
export async function inbound_sync(path) {
  return await invoke('plugin:profile_share|profile_share_inbound_sync', { path })
}

// syncs profile to update server
// only allowed if profile is owned by user
export async function outbound_sync(path) {
  return await invoke('plugin:profile_share|profile_share_outbound_sync', { path })
}