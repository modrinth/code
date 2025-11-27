import type { Archon } from '@modrinth/api-client'
import { createContext } from '@modrinth/ui'
import type { ComputedRef, Ref } from 'vue'

export interface ModrinthServerContext {
	readonly serverId: string
	readonly server: Ref<Archon.Servers.v0.Server>

	// Websocket state
	readonly isConnected: Ref<boolean>
	readonly powerState: Ref<Archon.Websocket.v0.PowerState>
	readonly isServerRunning: ComputedRef<boolean>
}

export const [injectModrinthServerContext, provideModrinthServerContext] =
	createContext<ModrinthServerContext>('[id].vue', 'modrinthServerContext')
