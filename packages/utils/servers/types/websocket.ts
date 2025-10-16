import type { ServerState } from './server'
import type { Stats } from './stats'

export interface WSAuth {
	url: string
	token: string
}

export interface WSLogEvent {
	event: 'log'
	message: string
}

type CurrentStats = Stats['current']

export interface WSStatsEvent extends CurrentStats {
	event: 'stats'
}

export interface WSAuthExpiringEvent {
	event: 'auth-expiring'
}

export interface WSPowerStateEvent {
	event: 'power-state'
	state: ServerState
	// if state "crashed"
	oom_killed?: boolean
	exit_code?: number
}

export interface WSAuthIncorrectEvent {
	event: 'auth-incorrect'
}

export interface WSInstallationResultOkEvent {
	event: 'installation-result'
	result: 'ok'
}

export interface WSInstallationResultErrEvent {
	event: 'installation-result'
	result: 'err'
	reason: string
}

export type WSInstallationResultEvent = WSInstallationResultOkEvent | WSInstallationResultErrEvent

export interface WSAuthOkEvent {
	event: 'auth-ok'
}

export interface WSUptimeEvent {
	event: 'uptime'
	uptime: number // seconds
}

export interface WSNewModEvent {
	event: 'new-mod'
}

export type WSBackupTask = 'create' | 'restore'
export type WSBackupState = 'ongoing' | 'done' | 'failed' | 'cancelled' | 'unchanged'

export interface WSBackupProgressEvent {
	event: 'backup-progress'
	task: WSBackupTask
	id: string
	progress: number // percentage
	state: WSBackupState
	ready: boolean
}

export type FSQueuedOpUnarchive = {
	op: 'unarchive'
	src: string
}

export type FSQueuedOp = FSQueuedOpUnarchive

export type FSOpUnarchive = {
	op: 'unarchive'
	progress: number // Note: 1 does not mean it's done
	id: string // UUID

	mime: string
	src: string
	state:
		| 'queued'
		| 'ongoing'
		| 'cancelled'
		| 'done'
		| 'failed-corrupted'
		| 'failed-invalid-path'
		| 'failed-cf-no-serverpack'
		| 'failed-cf-not-available'
		| 'failed-not-reachable'

	current_file: string | null
	failed_path?: string
	bytes_processed: number
	files_processed: number
	started: string
}

export type FilesystemOp = FSOpUnarchive

export interface WSFilesystemOpsEvent {
	event: 'filesystem-ops'
	all: FilesystemOp[]
}

export type WSEvent =
	| WSLogEvent
	| WSStatsEvent
	| WSPowerStateEvent
	| WSAuthExpiringEvent
	| WSAuthIncorrectEvent
	| WSInstallationResultEvent
	| WSAuthOkEvent
	| WSUptimeEvent
	| WSNewModEvent
	| WSBackupProgressEvent
	| WSFilesystemOpsEvent
