/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank Profile object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/tauri'

/// Gets if a process has finished by PID
/// Returns bool
export async function has_finished_by_pid(pid) {
  return await invoke('process_has_finished_by_pid', { pid })
}

/// Gets process exit status by PID
/// Returns u32
export async function get_exit_status_by_pid(pid) {
  return await invoke('process_get_exit_status_by_pid', { pid })
}

/// Gets all process IDs
/// Returns [u32]
export async function get_all_pids() {
  return await invoke('process_get_all_pids')
}

/// Gets all running process IDs
/// Returns [u32]
export async function get_all_running_pids() {
  return await invoke('process_get_all_running_pids')
}

/// Gets process stderr by PID
/// Returns String
export async function get_stderr_by_pid(pid) {
  return await invoke('process_get_stderr_by_pid', { pid })
}

/// Gets process stdout by PID
/// Returns String
export async function get_stdout_by_pid(pid) {
  return await invoke('process_get_stdout_by_pid', { pid })
}

/// Kills a process by PID
export async function kill_by_pid(pid) {
  return await invoke('process_kill_by_pid', { pid })
}
