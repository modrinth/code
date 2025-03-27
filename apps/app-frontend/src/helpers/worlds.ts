import { invoke } from '@tauri-apps/api/core'
import { get_full_path } from '@/helpers/profile'
import { openPath } from '@/helpers/utils'

type BaseWorld = {
  name: string
  last_played?: string
  icon?: string
}

export type SingleplayerWorld = BaseWorld & {
  type: 'singleplayer'
  path: string
  game_mode: SingleplayerGameMode
  hardcore: boolean
}

export type ServerWorld = BaseWorld & {
  type: 'server'
  index: number
  address: string
  pack_status: ServerPackStatus
}

export type World = SingleplayerWorld | ServerWorld

export type SingleplayerGameMode = 'survival' | 'creative' | 'adventure' | 'spectator'
export type ServerPackStatus = 'enabled' | 'disabled' | 'prompt'

export type ServerStatus = {
  // https://minecraft.wiki/w/Text_component_format
  description?: string | Chat
  players?: {
    max: number
    online: number
    sample: { name: string, id: string }[]
  }
  version?: {
    name: string
    protocol: number
  }
  favicon?: string
  enforces_secure_chat: boolean
  ping?: number
}

export interface Chat {
  text: string
  bold: boolean
  italic: boolean
  underlined: boolean
  strikethrough: boolean
  obfuscated: boolean
  color?: string
  extra: Chat[]
}

export async function get_profile_worlds(path: string): Promise<World[]> {
  return await invoke('plugin:worlds|get_profile_worlds', { path })
}

export async function rename_world(instance: string, world: string, newName: string): Promise<void> {
  return await invoke('plugin:worlds|rename_world', { instance, world, newName })
}

export async function reset_world_icon(instance: string, world: string): Promise<void> {
  return await invoke('plugin:worlds|reset_world_icon', { instance, world })
}

export async function backup_world(instance: string, world: string): Promise<number> {
  return await invoke('plugin:worlds|backup_world', { instance, world })
}

export async function delete_world(instance: string, world: string): Promise<void> {
  return await invoke('plugin:worlds|delete_world', { instance, world })
}

export async function add_server_to_profile(path: string, name: string, address: string, packStatus: ServerPackStatus): Promise<number> {
  return await invoke('plugin:worlds|add_server_to_profile', { path, name, address, packStatus })
}

export async function edit_server_in_profile(path: string, index: number, name: string, address: string, packStatus: ServerPackStatus): Promise<void> {
  return await invoke('plugin:worlds|edit_server_in_profile', { path, index, name, address, packStatus })
}

export async function remove_server_from_profile(path: string, index: number): Promise<void> {
  return await invoke('plugin:worlds|remove_server_from_profile', { path, index })
}

export async function get_profile_protocol_version(path: string): Promise<number | null> {
  return await invoke('plugin:worlds|get_profile_protocol_version', { path })
}

export async function get_server_status(address: string, protocolVersion: number | null = null): Promise<ServerStatus> {
  return await invoke('plugin:worlds|get_server_status', { address, protocolVersion })
}

export async function start_join_singleplayer_world(path: string, world: string): Promise<any> {
  return await invoke('plugin:worlds|start_join_singleplayer_world', { path, world })
}

export async function start_join_server(path: string, address: string): Promise<any> {
  return await invoke('plugin:worlds|start_join_server', { path, address })
}

export async function showProfileInFolder(path) {
  const fullPath = await get_full_path(path)
  return await openPath(fullPath)
}
