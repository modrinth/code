/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank Profile object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/core'

export async function login() {
  return await invoke('plugin:mr-auth|modrinth_login')
}

export async function logout() {
  return await invoke('plugin:mr-auth|logout')
}

export async function get() {
  return await invoke('plugin:mr-auth|get')
}
