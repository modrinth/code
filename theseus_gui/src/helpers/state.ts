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
  return await invoke('progress_bars_list')
}
