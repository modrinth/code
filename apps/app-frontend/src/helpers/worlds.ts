import { invoke } from '@tauri-apps/api/core'
import { get_full_path } from '@/helpers/profile'
import { openPath } from '@/helpers/utils'
import { autoToHTML } from '@geometrically/minecraft-motd-parser'
import dayjs from 'dayjs'
import type { GameVersion } from '@modrinth/ui'

type BaseWorld = {
  name: string
  last_played?: string
  icon?: string
  display_status: DisplayStatus
  type: WorldType
}

export type WorldType = 'singleplayer' | 'server'
export type DisplayStatus = 'normal' | 'hidden' | 'favorite'

export type SingleplayerWorld = BaseWorld & {
  type: 'singleplayer'
  path: string
  game_mode: SingleplayerGameMode
  hardcore: boolean
  locked: boolean
}

export type ServerWorld = BaseWorld & {
  type: 'server'
  index: number
  address: string
  pack_status: ServerPackStatus
}

export type World = SingleplayerWorld | ServerWorld

export type WorldWithProfile = {
  profile: string
} & World

export type SingleplayerGameMode = 'survival' | 'creative' | 'adventure' | 'spectator'
export type ServerPackStatus = 'enabled' | 'disabled' | 'prompt'

export type ServerStatus = {
  // https://minecraft.wiki/w/Text_component_format
  description?: string | Chat
  players?: {
    max: number
    online: number
    sample: { name: string; id: string }[]
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

export type ServerData = {
  refreshing: boolean
  status?: ServerStatus
  rawMotd?: string | Chat
  renderedMotd?: string
}

export async function get_recent_worlds(
  limit: number,
  displayStatuses?: DisplayStatus[],
): Promise<WorldWithProfile[]> {
  return await invoke('plugin:worlds|get_recent_worlds', { limit, displayStatuses })
}

export async function get_profile_worlds(path: string): Promise<World[]> {
  return await invoke('plugin:worlds|get_profile_worlds', { path })
}

export async function get_singleplayer_world(
  instance: string,
  world: string,
): Promise<SingleplayerWorld> {
  return await invoke('plugin:worlds|get_singleplayer_world', { instance, world })
}

export async function set_world_display_status(
  instance: string,
  worldType: WorldType,
  worldId: string,
  displayStatus: DisplayStatus,
): Promise<void> {
  return await invoke('plugin:worlds|set_world_display_status', {
    instance,
    worldType,
    worldId,
    displayStatus,
  })
}

export async function rename_world(
  instance: string,
  world: string,
  newName: string,
): Promise<void> {
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

export async function add_server_to_profile(
  path: string,
  name: string,
  address: string,
  packStatus: ServerPackStatus,
): Promise<number> {
  return await invoke('plugin:worlds|add_server_to_profile', { path, name, address, packStatus })
}

export async function edit_server_in_profile(
  path: string,
  index: number,
  name: string,
  address: string,
  packStatus: ServerPackStatus,
): Promise<void> {
  return await invoke('plugin:worlds|edit_server_in_profile', {
    path,
    index,
    name,
    address,
    packStatus,
  })
}

export async function remove_server_from_profile(path: string, index: number): Promise<void> {
  return await invoke('plugin:worlds|remove_server_from_profile', { path, index })
}

export async function get_profile_protocol_version(path: string): Promise<number | null> {
  return await invoke('plugin:worlds|get_profile_protocol_version', { path })
}

export async function get_server_status(
  address: string,
  protocolVersion: number | null = null,
): Promise<ServerStatus> {
  return await invoke('plugin:worlds|get_server_status', { address, protocolVersion })
}

export async function start_join_singleplayer_world(path: string, world: string): Promise<unknown> {
  return await invoke('plugin:worlds|start_join_singleplayer_world', { path, world })
}

export async function start_join_server(path: string, address: string): Promise<unknown> {
  return await invoke('plugin:worlds|start_join_server', { path, address })
}

export async function showWorldInFolder(instancePath: string, worldPath: string) {
  const fullPath = await get_full_path(instancePath)
  return await openPath(fullPath + '/saves/' + worldPath)
}

export function getWorldIdentifier(world: World) {
  return world.type === 'singleplayer' ? world.path : world.address
}

export function sortWorlds(worlds: World[]) {
  worlds.sort((a, b) => {
    if (!a.last_played) {
      return 1
    }
    if (!b.last_played) {
      return -1
    }
    return dayjs(b.last_played).diff(dayjs(a.last_played))
  })
}

export function isSingleplayerWorld(world: World): world is SingleplayerWorld {
  return world.type === 'singleplayer'
}

export function isServerWorld(world: World): world is ServerWorld {
  return world.type === 'server'
}

export async function refreshServerData(
  serverData: ServerData,
  protocolVersion: number | null,
  address: string,
): Promise<void> {
  serverData.refreshing = true
  await get_server_status(address, protocolVersion)
    .then((status) => {
      serverData.status = status
      if (status.description) {
        serverData.rawMotd = status.description
        serverData.renderedMotd = autoToHTML(status.description)
      }
    })
    .catch((err) => {
      console.error(`Refreshing addr: ${address}`, err)
    })
    .finally(() => {
      serverData.refreshing = false
    })
}

export async function refreshServers(
  worlds: World[],
  serverData: Record<string, ServerData>,
  protocolVersion: number | null,
) {
  const servers = worlds.filter(isServerWorld)
  servers.forEach((server) => {
    if (!serverData[server.address]) {
      serverData[server.address] = {
        refreshing: true,
      }
    } else {
      serverData[server.address].refreshing = true
    }
  })

  // noinspection ES6MissingAwait - handled with .then by refreshServerData already
  Promise.all(
    Object.keys(serverData).map((address) =>
      refreshServerData(serverData[address], protocolVersion, address),
    ),
  )
}

export async function refreshWorld(worlds: World[], instancePath: string, worldPath: string) {
  const index = worlds.findIndex((w) => w.type === 'singleplayer' && w.path === worldPath)
  const newWorld = await get_singleplayer_world(instancePath, worldPath)
  if (index !== -1) {
    worlds[index] = newWorld
  } else {
    console.info(`Adding new world at path: ${worldPath}.`)
    worlds.push(newWorld)
  }
  sortWorlds(worlds)
}

export async function handleDefaultProfileUpdateEvent(
  worlds: World[],
  instancePath: string,
  e: ProfileEvent,
) {
  if (e.event === 'world_updated') {
    await refreshWorld(worlds, instancePath, e.world)
  }

  if (e.event === 'server_joined') {
    const world = worlds.find(
      (w) =>
        w.type === 'server' &&
        (w.address === `${e.host}:${e.port}` || (e.port == 25565 && w.address == e.host)),
    )
    if (world) {
      world.last_played = e.timestamp
      sortWorlds(worlds)
    } else {
      console.error(`Could not find world for server join event: ${e.host}:${e.port}`)
    }
  }
}

export async function refreshWorlds(instancePath: string): Promise<World[]> {
  const worlds = await get_profile_worlds(instancePath).catch((err) => {
    console.error(`Error refreshing worlds for instance: ${instancePath}`, err)
  })
  if (worlds) {
    sortWorlds(worlds)
  }

  return worlds ?? []
}

const FIRST_QUICK_PLAY_VERSION = '23w14a'

export function hasQuickPlaySupport(gameVersions: GameVersion[], currentVersion: string) {
  if (!gameVersions.length) {
    return false
  }

  const versionIndex = gameVersions.findIndex((v) => v.version === currentVersion)
  const targetIndex = gameVersions.findIndex((v) => v.version === FIRST_QUICK_PLAY_VERSION)

  return versionIndex !== -1 && targetIndex !== -1 && versionIndex <= targetIndex
}

export type ProfileEvent = { profile_path_id: string } & (
  | {
      event: 'servers_updated'
    }
  | {
      event: 'world_updated'
      world: string
    }
  | {
      event: 'server_joined'
      host: string
      port: number
      timestamp: string
    }
)
