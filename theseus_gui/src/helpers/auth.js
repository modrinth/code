/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank Profile object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/tauri'

// Example function:
// User goes to auth_url to complete flow, and when completed, authenticate_await_completion() returns the credentials
// export async function authenticate() {
//   const auth_url = await authenticate_begin_flow()
//   console.log(auth_url)
//   await authenticate_await_completion()
// }

/// Authenticate a user with Hydra - part 1
/// This begins the authentication flow quasi-synchronously
/// This returns a URL to be opened in a browser
export async function authenticate_begin_flow() {
  return await invoke('auth_authenticate_begin_flow')
}

/// Authenticate a user with Hydra - part 2
/// This completes the authentication flow quasi-synchronously, returning the sign-in credentials
/// (and also adding the credentials to the state)
/// This returns a Credentials object
export async function authenticate_await_completion() {
  return await invoke('auth_authenticate_await_completion')
}

/// Refresh some credentials using Hydra, if needed
/// user is UUID
/// update_name is bool
/// Returns a Credentials object
export async function refresh(user, update_name) {
  return await invoke('auth_refresh', { user, update_name })
}

/// Remove a user account from the database
/// user is UUID
export async function remove_user(user) {
  return await invoke('auth_remove_user', { user })
}

// Add a path as a profile in-memory
// user is UUID
/// Returns a bool
export async function has_user(user) {
  return await invoke('auth_has_user', { user })
}

/// Returns a list of users
/// Returns an Array of Credentials
export async function users() {
  return await invoke('auth_users')
}

// Get a user by UUID
// Prefer to use refresh() instead of this because it will refresh the credentials
// user is UUID
// Returns Credentials (of user)
export async function get_user(user) {
  return await invoke('auth_get_user', { user })
}
