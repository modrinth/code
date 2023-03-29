/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank Profile object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/tauri';

/// Authenticate a user with Hydra
export async function authenticate(browser_url) {
    return await invoke('auth_authenticate', browser_url);
}

/// Refresh some credentials using Hydra, if needed
export async function refresh(user, update_name) {
    return await invoke('auth_refresh', user, update_name);
}

/// Remove a user account from the database
export async function remove_user(user) {
    return await invoke('auth_remove_user', user);
}

// Add a path as a profile in-memory
export async function has_user(user) {
    return await invoke('auth_has_user', user);
}

// Remove a profile
export async function users() {
    return await invoke('auth_users');
}
