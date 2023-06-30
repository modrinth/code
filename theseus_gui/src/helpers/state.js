/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank Profile object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/tauri'

// Initialize the theseus API state
// This should be called during the initializion/opening of the launcher
export async function initialize_state() {
  return await invoke('initialize_state')
}

// Gets active progress bars
export async function progress_bars_list() {
  return await invoke('plugin:utils|progress_bars_list')
}

// Check if any safe loading bars are active
export async function check_safe_loading_bars_complete() {
  return await invoke('plugin:utils|safety_check_safe_loading_bars')
}

// Get opening command
// For example, if a user clicks on an .mrpack to open the app.
// This should be called once and only when the app is done booting up and ready to receive a command
// Returns a Command struct- see events.js
export async function get_opening_command() {
  return await invoke('plugin:utils|get_opening_command')
}

// Wait for settings to sync
export async function await_sync() {
  return await invoke('plugin:utils|await_sync')
}
