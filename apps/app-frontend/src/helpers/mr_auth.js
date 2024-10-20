/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank Profile object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/core'

export async function login(provider) {
  return await invoke('modrinth_auth_login', { provider })
}

export async function login_pass(username, password, challenge) {
  return await invoke('plugin:mr-auth|login_pass', { username, password, challenge })
}

export async function login_2fa(code, flow) {
  return await invoke('plugin:mr-auth|login_2fa', { code, flow })
}

export async function create_account(username, email, password, challenge, signUpNewsletter) {
  return await invoke('plugin:mr-auth|create_account', {
    username,
    email,
    password,
    challenge,
    signUpNewsletter,
  })
}

export async function logout() {
  return await invoke('plugin:mr-auth|logout')
}

export async function get() {
  return await invoke('plugin:mr-auth|get')
}
