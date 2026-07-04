import type mitt from 'mitt'

import type { Archon } from '../modules/archon/types.js'
import type { RequestOptions } from '../types/request.js'

export type SyncEventType = Archon.Sync.v1.SyncEvent['type']

export type SyncEventOfType<E extends SyncEventType> = Extract<
	Archon.Sync.v1.SyncEvent,
	{ type: E }
>

export type SyncEventHandler<E extends Archon.Sync.v1.SyncEvent = Archon.Sync.v1.SyncEvent> = (
	event: E,
) => void

export type SyncStatusState =
	| 'idle'
	| 'connecting'
	| 'connected'
	| 'reconnecting'
	| 'disconnected'
	| 'error'

export type SyncStatus = {
	state: SyncStatusState
	connected: boolean
	reconnecting: boolean
	reconnectAttempts: number
	retryDelay: number
	lastEventId?: string
	error?: unknown
}

export type SyncStatusHandler = (status: SyncStatus) => void

export type SyncConnectOptions = {
	intent?: Archon.Sync.v1.SyncIntent
	force?: boolean
}

export type SyncConnection = {
	serverId: string
	intent: Archon.Sync.v1.SyncIntent
	controller?: AbortController
	reconnectAttempts: number
	reconnectTimer?: ReturnType<typeof setTimeout>
	reconnectResolve?: () => void
	retryDelay: number
	lastEventId?: string
	stopped: boolean
	status: SyncStatusState
	error?: unknown
}

export type SyncEmitterEvents = Record<string, unknown>

export abstract class AbstractSyncClient {
	protected connections = new Map<string, SyncConnection>()
	protected abstract emitter: ReturnType<typeof mitt<SyncEmitterEvents>>

	constructor(
		protected client: {
			stream: (path: string, options: RequestOptions) => Promise<ReadableStream<Uint8Array>>
		},
	) {}

	abstract safeConnectServer(serverId: string, options?: SyncConnectOptions): Promise<void>

	abstract disconnect(serverId: string): void

	abstract disconnectAll(): void

	on<E extends SyncEventType>(
		serverId: string,
		eventType: E,
		handler: SyncEventHandler<SyncEventOfType<E>>,
	): () => void {
		const eventKey = this.getEventKey(serverId, eventType)
		const wrapped = handler as (event: unknown) => void

		this.emitter.on(eventKey, wrapped)

		return () => {
			this.emitter.off(eventKey, wrapped)
		}
	}

	onAny(serverId: string, handler: SyncEventHandler): () => void {
		const eventKey = this.getAnyEventKey(serverId)
		const wrapped = handler as (event: unknown) => void

		this.emitter.on(eventKey, wrapped)

		return () => {
			this.emitter.off(eventKey, wrapped)
		}
	}

	onStatus(serverId: string, handler: SyncStatusHandler): () => void {
		const eventKey = this.getStatusEventKey(serverId)
		const wrapped = handler as (event: unknown) => void

		this.emitter.on(eventKey, wrapped)

		return () => {
			this.emitter.off(eventKey, wrapped)
		}
	}

	getStatus(serverId: string): SyncStatus | null {
		const connection = this.connections.get(serverId)
		if (!connection) return null

		return this.connectionToStatus(connection)
	}

	protected emitSyncEvent(serverId: string, event: Archon.Sync.v1.SyncEvent): void {
		this.emitter.emit(this.getEventKey(serverId, event.type), event)
		this.emitter.emit(this.getAnyEventKey(serverId), event)
	}

	protected updateStatus(
		connection: SyncConnection,
		status: SyncStatusState,
		error?: unknown,
	): void {
		connection.status = status
		connection.error = error
		this.emitter.emit(
			this.getStatusEventKey(connection.serverId),
			this.connectionToStatus(connection),
		)
	}

	protected clearListeners(serverId: string): void {
		this.emitter.all.forEach((_handlers, type) => {
			if (type.toString().startsWith(`${serverId}:`)) {
				this.emitter.all.delete(type)
			}
		})
	}

	protected connectionToStatus(connection: SyncConnection): SyncStatus {
		return {
			state: connection.status,
			connected: connection.status === 'connected',
			reconnecting: connection.status === 'reconnecting',
			reconnectAttempts: connection.reconnectAttempts,
			retryDelay: connection.retryDelay,
			lastEventId: connection.lastEventId,
			error: connection.error,
		}
	}

	private getEventKey(serverId: string, eventType: string): string {
		return `${serverId}:${eventType}`
	}

	private getAnyEventKey(serverId: string): string {
		return `${serverId}:*`
	}

	private getStatusEventKey(serverId: string): string {
		return `${serverId}:__status`
	}
}
