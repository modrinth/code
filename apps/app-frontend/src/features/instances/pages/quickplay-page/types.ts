import type {
	DisplayStatus,
	ServerPackStatus,
	ServerWorld,
	World,
} from '@/platform/app-lib/worlds/types'

export type QuickplayTab = 'servers' | 'worlds'

export type ServerAvailability = 'online' | 'offline' | 'pending'

export type RemoveScope = 'here' | 'all'

export interface ServerFormValues {
	name: string
	address: string
	packStatus: ServerPackStatus
}

export interface AddServerValues extends ServerFormValues {
	play: boolean
}

export interface EditServerValues extends ServerFormValues {
	server: ServerWorld
	displayStatus: DisplayStatus
}

export interface EditWorldValues {
	path: string
	name: string
	removeIcon: boolean
	displayStatus: DisplayStatus
	previousDisplayStatus: DisplayStatus
}

export interface RemoveWorldValues {
	world: World
	scope: RemoveScope
}
