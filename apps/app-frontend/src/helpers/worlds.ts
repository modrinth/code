import { invoke } from '@tauri-apps/api/core'

export type World = {
  name: string
  last_played: string
  icon?: string
  pinned: boolean
} & (
  | {
      type: 'singleplayer'
      path: string
      game_mode: GameMode
      hardcore: boolean
    }
  | {
      type: 'server'
      address: string
    }
)

export type GameMode = 'creative' | 'survival' | 'adventure' | 'spectator'

export type ServerStatus = {
  ping?: number
} & CraftPingResponse

export interface CraftPingResponse {
  version: string
  protocol: number
  enforces_secure_chat?: boolean
  previews_chat?: boolean
  max_players: number
  online_players: number
  sample?: Player[]
  description: Chat
  favicon?: Uint8Array
  mod_info?: ModInfo
  forge_data?: ForgeData
}

export interface ModInfo {
  mod_type: string
  mod_list: ModInfoItem[]
}

export interface ModInfoItem {
  mod_id: string
  version: string
}

export interface Player {
  name: string
  id: string
}

export interface ForgeData {
  channels: ForgeChannel[]
  mods: ForgeMod[]
  fml_network_version: number
}

export interface ForgeChannel {
  res: string
  version: string
  required: boolean
}

export interface ForgeMod {
  mod_id: string
  mod_marker: string
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

export async function get_server_status(address: string): Promise<CraftPingResponse> {
  return await invoke('plugin:worlds|get_server_status', { address })
}
