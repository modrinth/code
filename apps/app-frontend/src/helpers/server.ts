/**
 * Helper bindings for locally-hosted dedicated servers.
 *
 * These wrap the `server` Tauri plugin (see `apps/app/src/api/server.rs`).
 */
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

export type ServerSoftware = 'vanilla' | 'paper' | 'purpur' | 'fabric'

export type ServerInstallStage = 'not_installed' | 'installing' | 'installed' | 'failed'

export interface ServerInstance {
	id: string
	name: string
	software: ServerSoftware
	minecraft_version: string
	software_version: string | null
	jar_file: string | null
	install_stage: ServerInstallStage
	java_path: string | null
	memory_mb: number
	extra_java_args: string[]
	created: string
	modified: string
	last_started: string | null
	icon_path: string | null
}

export interface RunningServerInfo {
	id: string
	pid: number | null
	start_time: string
}

export interface ServerLogPayload {
	server_id: string
	line: string
}

export interface ServerProcessPayload {
	server_id: string
	event: 'launched' | 'stopped'
}

export const SERVER_SOFTWARE: { value: ServerSoftware; label: string; description: string }[] = [
	{ value: 'paper', label: 'Paper', description: 'High-performance server with plugin support' },
	{ value: 'purpur', label: 'Purpur', description: 'Paper fork with extra configuration options' },
	{ value: 'fabric', label: 'Fabric', description: 'Lightweight, modular mod loader' },
	{ value: 'vanilla', label: 'Vanilla', description: 'The official, unmodified Minecraft server' },
]

export async function list(): Promise<ServerInstance[]> {
	return await invoke('plugin:server|server_list')
}

export async function get(id: string): Promise<ServerInstance | null> {
	return await invoke('plugin:server|server_get', { id })
}

export async function create(
	name: string,
	software: ServerSoftware,
	minecraftVersion: string,
): Promise<ServerInstance> {
	return await invoke('plugin:server|server_create', { name, software, minecraftVersion })
}

export async function remove(id: string): Promise<void> {
	return await invoke('plugin:server|server_remove', { id })
}

export async function install(id: string): Promise<ServerInstance> {
	return await invoke('plugin:server|server_install', { id })
}

export async function start(id: string): Promise<RunningServerInfo> {
	return await invoke('plugin:server|server_start', { id })
}

export async function stop(id: string): Promise<void> {
	return await invoke('plugin:server|server_stop', { id })
}

export async function kill(id: string): Promise<void> {
	return await invoke('plugin:server|server_kill', { id })
}

export async function sendCommand(id: string, command: string): Promise<void> {
	return await invoke('plugin:server|server_send_command', { id, command })
}

export async function getLog(id: string): Promise<string[]> {
	return await invoke('plugin:server|server_get_log', { id })
}

export async function getRunning(): Promise<RunningServerInfo[]> {
	return await invoke('plugin:server|server_get_running')
}

export async function isRunning(id: string): Promise<boolean> {
	return await invoke('plugin:server|server_is_running', { id })
}

export async function getConfig(id: string, file: string): Promise<string> {
	return await invoke('plugin:server|server_get_config', { id, file })
}

export async function setConfig(id: string, file: string, contents: string): Promise<void> {
	return await invoke('plugin:server|server_set_config', { id, file, contents })
}

export async function getVersions(software: ServerSoftware): Promise<string[]> {
	return await invoke('plugin:server|server_get_versions', { software })
}

export async function onServerLog(
	callback: (payload: ServerLogPayload) => void,
): Promise<UnlistenFn> {
	return await listen<ServerLogPayload>('server_log', (event) => callback(event.payload))
}

export async function onServerProcess(
	callback: (payload: ServerProcessPayload) => void,
): Promise<UnlistenFn> {
	return await listen<ServerProcessPayload>('server_process', (event) => callback(event.payload))
}
