import type { Archon, UploadState } from '@modrinth/api-client'
import type { Stats } from '@modrinth/utils'
import type { ComputedRef, Ref } from 'vue'

import type { MessageDescriptor } from '#ui/composables/i18n'
import type { FileOperation } from '#ui/layouts/shared/files-tab/types'

import { createContext } from '.'

export interface BusyReason {
	reason: MessageDescriptor
}

export interface FilesystemAuth {
	url: string
	token: string
}

export interface ModrinthServerContext {
	readonly serverId: string
	readonly worldId: Ref<string | null>
	readonly server: Ref<Archon.Servers.v0.Server>

	// Websocket state
	readonly isConnected: Ref<boolean>
	readonly isWsAuthIncorrect: Ref<boolean>
	readonly powerState: Ref<Archon.Websocket.v0.PowerState>
	readonly powerStateDetails: Ref<{ oom_killed?: boolean; exit_code?: number } | undefined>
	readonly isServerRunning: ComputedRef<boolean>
	readonly stats: Ref<Stats>
	readonly uptimeSeconds: Ref<number>

	// Content sync state
	readonly isSyncingContent: Ref<boolean>

	// Busy state — when non-empty, all write operations should be disabled
	readonly busyReasons: ComputedRef<BusyReason[]>

	// Filesystem state
	readonly fsAuth: Ref<FilesystemAuth | null>
	readonly fsOps: Ref<Archon.Websocket.v0.FilesystemOperation[]>
	readonly fsQueuedOps: Ref<Archon.Websocket.v0.QueuedFilesystemOp[]>
	refreshFsAuth: () => Promise<void>

	// File upload state
	readonly uploadState: Ref<UploadState>
	readonly cancelUpload: Ref<(() => void) | null>

	// File operations (extract, move, etc.)
	readonly activeOperations: ComputedRef<FileOperation[]>
	dismissOperation: (opId: string, action: 'dismiss' | 'cancel') => Promise<void>
}

export const [injectModrinthServerContext, provideModrinthServerContext] =
	createContext<ModrinthServerContext>('[id].vue', 'modrinthServerContext')
