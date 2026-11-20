export type WorldType = 'singleplayer' | 'server'
export type DisplayStatus = 'normal' | 'hidden' | 'favorite'
export type SingleplayerGameMode = 'survival' | 'creative' | 'adventure' | 'spectator'
export type ServerPackStatus = 'enabled' | 'disabled' | 'prompt'
export type ServerSource = 'user_synced' | 'modpack' | 'linked_server_project' | 'local_desynced'
export type DesyncServerMode = 'keep_in_other_instances' | 'remove_from_other_instances'

type BaseWorld = {
	name: string
	last_played?: string
	icon?: string
	display_status: DisplayStatus
	type: WorldType
}

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
	server_id?: string
	source?: ServerSource
	address: string
	pack_status: ServerPackStatus
	project_id?: string
	content_kind?: string
}

export type World = SingleplayerWorld | ServerWorld

export type WorldWithInstance = {
	instance_id: string
} & World

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

export type ServerStatus = {
	description?: string | Chat
	players?: {
		max: number
		online: number
		sample: { name: string; id: string }[]
	}
	version?: {
		name: string
		protocol: number
		legacy: boolean
	}
	favicon?: string
	enforces_secure_chat: boolean
	ping?: number
}

export type ProtocolVersion = {
	version: number
	legacy: boolean
}
