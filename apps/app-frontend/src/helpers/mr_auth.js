/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank Profile object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/tauri'

export async function authenticate_begin_flow(provider) {
  return await invoke('plugin:mr_auth|authenticate_begin_flow', { provider })
}

export async function authenticate_await_completion() {
  return await invoke('plugin:mr_auth|authenticate_await_completion')
}

export async function cancel_flow() {
  return await invoke('plugin:mr_auth|cancel_flow')
}
export async function login_pass(username, password, challenge) {
  return await invoke('plugin:mr_auth|login_pass', { username, password, challenge })
}

export async function login_2fa(code, flow) {
  return await invoke('plugin:mr_auth|login_2fa', { code, flow })
}

export async function create_account(username, email, password, challenge, signUpNewsletter) {
  return await invoke('plugin:mr_auth|create_account', {
    username,
    email,
    password,
    challenge,
    signUpNewsletter,
  })
}

export async function refresh() {
  return await invoke('plugin:mr_auth|refresh')
}

export async function logout() {
  return await invoke('plugin:mr_auth|logout')
}

export async function get() {
  return await invoke('plugin:mr_auth|get')
}
