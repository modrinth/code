/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank Profile object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/core'

export interface LoadingBarType {
	type?: string
	version?: string
	profile_path?: string
	pack_name?: string
}

export interface LoadingBar {
	id?: string | number
	loading_bar_uuid?: string | number
	title?: string
	message?: string
	current?: number
	total?: number
	bar_type?: LoadingBarType
}

export type OpeningCommandEvent =
	| 'RunMRPack'
	| 'InstallServer'
	| 'InstallVersion'
	| 'InstallMod'
	| 'InstallModpack'
	| string

export interface OpeningCommand {
	event: OpeningCommandEvent
	id?: string
	path?: string
}

// Initialize the theseus API state
// This should be called during the initializion/opening of the launcher
export async function initialize_state() {
	return await invoke<void>('initialize_state')
}

// Gets active progress bars
export async function progress_bars_list() {
	return await invoke<Record<string, LoadingBar>>('plugin:utils|progress_bars_list')
}

// Get opening command
// For example, if a user clicks on an .mrpack to open the app.
// This should be called once and only when the app is done booting up and ready to receive a command
// Returns a Command struct- see events.js
export async function get_opening_command() {
	return await invoke<OpeningCommand | null>('plugin:utils|get_opening_command')
}
