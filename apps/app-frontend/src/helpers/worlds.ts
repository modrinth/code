import { invoke } from '@tauri-apps/api/core'
import type { ServerStatus } from '@/helpers/types'

export type World = {
  name: string
  last_played: string
  icon?: string
} & (
  | {
      type: 'singleplayer'
      path: string
      game_mode: SingleplayerGameMode
      hardcore: boolean
    }
  | {
      type: 'server'
      address: string
      pack_status: ServerPackStatus
    }
)

export type SingleplayerGameMode = 'survival' | 'creative' | 'adventure' | 'spectator'
export type ServerPackStatus = 'enabled' | 'disabled' | 'prompt'

export async function get_profile_worlds(path: string): Promise<World[]> {
  return await invoke('plugin:worlds|get_profile_worlds', { path })
}

export async function get_profile_protocol_version(path: string): Promise<number | null> {
  return await invoke('plugin:worlds|get_profile_protocol_version', { path })
}

export async function get_server_status(address: string, protocol_version: number | null = null): Promise<ServerStatus> {
  return await invoke('plugin:worlds|get_server_status', { address, protocol_version })
}

export async function start_join_singleplayer_world(path: string, world: string): Promise<any> {
  return await invoke('plugin:worlds|start_join_singleplayer_world', { path, world })
}

export async function start_join_server(path: string, address: string): Promise<any> {
  return await invoke('plugin:worlds|start_join_server', { path, address })
}
