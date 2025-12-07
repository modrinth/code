import type { Archon } from '@modrinth/api-client'
import type { ComputedRef, Reactive, Ref } from 'vue'

import { createContext } from '.'

export type BackupTaskState = {
	progress: number
	state: Archon.Backups.v1.BackupState
}

export type BackupProgressEntry = {
	file?: BackupTaskState
	create?: BackupTaskState
	restore?: BackupTaskState
}

export type BackupsState = Map<string, BackupProgressEntry>

export interface ModrinthServerContext {
	readonly serverId: string
	readonly server: Ref<Archon.Servers.v0.Server>

	// Websocket state
	readonly isConnected: Ref<boolean>
	readonly powerState: Ref<Archon.Websocket.v0.PowerState>
	readonly isServerRunning: ComputedRef<boolean>
	readonly backupsState: Reactive<BackupsState>
	markBackupCancelled: (backupId: string) => void
}

export const [injectModrinthServerContext, provideModrinthServerContext] =
	createContext<ModrinthServerContext>('[id].vue', 'modrinthServerContext')
