/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank Profile object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/core'

export type ModrinthCredentials = {
	session: string
	expires: string
	user_id: string
	active: boolean
}

export async function login(): Promise<ModrinthCredentials> {
	return await invoke('plugin:mr-auth|modrinth_login')
}

export async function logout(): Promise<void> {
	return await invoke('plugin:mr-auth|logout')
}

export async function get(): Promise<ModrinthCredentials | null> {
	return await invoke('plugin:mr-auth|get')
}

export async function cancelLogin(): Promise<void> {
	return await invoke('plugin:mr-auth|cancel_modrinth_login')
}
