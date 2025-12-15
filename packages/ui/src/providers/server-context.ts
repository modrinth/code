import type { Archon } from '@modrinth/api-client'
import type { FilesystemOp, FSQueuedOp } from '@modrinth/utils'
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

export interface FilesystemAuth {
	url: string
	token: string
}

export interface ModrinthServerContext {
	readonly serverId: string
	readonly server: Ref<Archon.Servers.v0.Server>

	// Websocket state
	readonly isConnected: Ref<boolean>
	readonly powerState: Ref<Archon.Websocket.v0.PowerState>
	readonly isServerRunning: ComputedRef<boolean>
	readonly backupsState: Reactive<BackupsState>
	markBackupCancelled: (backupId: string) => void

	// Filesystem state (lazy-loaded when files page mounts)
	readonly fsAuth: Ref<FilesystemAuth | null>
	readonly fsOps: Ref<FilesystemOp[]>
	readonly fsQueuedOps: Ref<FSQueuedOp[]>
	refreshFsAuth: () => Promise<void>
}

export const [injectModrinthServerContext, provideModrinthServerContext] =
	createContext<ModrinthServerContext>('[id].vue', 'modrinthServerContext')
