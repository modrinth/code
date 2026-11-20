import { invoke } from '@tauri-apps/api/core'

import type {
	DesyncServerMode,
	DisplayStatus,
	ProtocolVersion,
	ServerPackStatus,
	ServerStatus,
	SingleplayerWorld,
	World,
	WorldType,
	WorldWithInstance,
} from './types'

export function getRecentWorlds(
	limit: number,
	displayStatuses?: DisplayStatus[],
): Promise<WorldWithInstance[]> {
	return invoke('plugin:worlds|get_recent_worlds', { limit, displayStatuses })
}

export function getInstanceWorlds(instanceId: string): Promise<World[]> {
	return invoke('plugin:worlds|get_instance_worlds', { instanceId })
}

export function getSingleplayerWorld(instance: string, world: string): Promise<SingleplayerWorld> {
	return invoke('plugin:worlds|get_singleplayer_world', { instance, world })
}

export function setWorldDisplayStatus(
	instance: string,
	worldType: WorldType,
	worldId: string,
	displayStatus: DisplayStatus,
): Promise<void> {
	return invoke('plugin:worlds|set_world_display_status', {
		instance,
		worldType,
		worldId,
		displayStatus,
	})
}

export function renameWorld(instance: string, world: string, newName: string): Promise<void> {
	return invoke('plugin:worlds|rename_world', { instance, world, newName })
}

export function resetWorldIcon(instance: string, world: string): Promise<void> {
	return invoke('plugin:worlds|reset_world_icon', { instance, world })
}

export function backupWorld(instance: string, world: string): Promise<number> {
	return invoke('plugin:worlds|backup_world', { instance, world })
}

export function deleteWorld(instance: string, world: string): Promise<void> {
	return invoke('plugin:worlds|delete_world', { instance, world })
}

export function addServerToInstance(
	instanceId: string,
	name: string,
	address: string,
	packStatus: ServerPackStatus,
	projectId?: string,
	contentKind?: string,
): Promise<number> {
	return invoke('plugin:worlds|add_server_to_instance', {
		instanceId,
		name,
		address,
		packStatus,
		projectId,
		contentKind,
	})
}

export function editServerInInstance(
	instanceId: string,
	index: number,
	name: string,
	address: string,
	packStatus: ServerPackStatus,
): Promise<void> {
	return invoke('plugin:worlds|edit_server_in_instance', {
		instanceId,
		index,
		name,
		address,
		packStatus,
	})
}

export function removeServerFromInstance(instanceId: string, index: number): Promise<void> {
	return invoke('plugin:worlds|remove_server_from_instance', { instanceId, index })
}

export function ensureManagedServerInInstance(
	instanceId: string,
	name: string,
	address: string,
): Promise<void> {
	return invoke('plugin:worlds|ensure_managed_server_in_instance', { instanceId, name, address })
}

export function desyncServer(
	instanceId: string,
	serverId: string,
	mode: DesyncServerMode,
): Promise<void> {
	return invoke('plugin:worlds|desync_server', { instanceId, serverId, mode })
}

export function getInstanceProtocolVersion(instanceId: string): Promise<ProtocolVersion | null> {
	return invoke('plugin:worlds|get_instance_protocol_version', { instanceId })
}

export function getServerStatus(
	address: string,
	protocolVersion: ProtocolVersion | null = null,
): Promise<ServerStatus> {
	return invoke('plugin:worlds|get_server_status', { address, protocolVersion })
}

export function startJoinSingleplayerWorld(instanceId: string, world: string): Promise<unknown> {
	return invoke('plugin:worlds|start_join_singleplayer_world', { instanceId, world })
}

export function startJoinServer(instanceId: string, address: string): Promise<unknown> {
	return invoke('plugin:worlds|start_join_server', { instanceId, address })
}
