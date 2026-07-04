import mitt from 'mitt'

import {
	AbstractSyncClient,
	type SyncConnection,
	type SyncConnectOptions,
	type SyncEmitterEvents,
} from '../core/abstract-sync.js'
import type { Archon } from '../modules/archon/types.js'
import { type ParsedSseItem, parseSyncEventData, SseParser } from '../utils/sse.js'

type StreamReadResult = 'closed' | 'protocol-reconnect'

const DEFAULT_RETRY_DELAY = 1000
const MAX_RECONNECT_DELAY = 30000
const JITTER_MS = 1000

export class GenericSyncClient extends AbstractSyncClient {
	protected emitter = mitt<SyncEmitterEvents>()

	async safeConnectServer(serverId: string, options: SyncConnectOptions = {}): Promise<void> {
		const existing = this.connections.get(serverId)
		if (existing && !options.force && !existing.stopped && existing.status !== 'disconnected') {
			return
		}

		if (existing) {
			this.closeConnection(serverId)
		}

		const connection: SyncConnection = {
			serverId,
			intent: options.intent ?? 'all',
			reconnectAttempts: 0,
			retryDelay: DEFAULT_RETRY_DELAY,
			stopped: false,
			status: 'idle',
		}

		this.connections.set(serverId, connection)
		void this.runConnection(connection)
	}

	disconnect(serverId: string): void {
		this.closeConnection(serverId)
		this.clearListeners(serverId)
	}

	disconnectAll(): void {
		for (const serverId of this.connections.keys()) {
			this.disconnect(serverId)
		}
	}

	private async runConnection(connection: SyncConnection): Promise<void> {
		while (!connection.stopped) {
			const hadConnected = connection.status === 'connected'
			this.updateStatus(connection, hadConnected ? 'reconnecting' : 'connecting')

			const controller = new AbortController()
			connection.controller = controller

			try {
				const stream = await this.client.stream('/sync', {
					api: 'archon',
					version: 1,
					method: 'GET',
					params: {
						scope: `server:${connection.serverId}`,
						intent: this.intentToParam(connection.intent),
					},
					headers: connection.lastEventId
						? {
								'Last-Event-Id': connection.lastEventId,
							}
						: undefined,
					signal: controller.signal,
					retry: false,
					circuitBreaker: false,
				})

				if (connection.stopped) return

				connection.reconnectAttempts = 0
				this.updateStatus(connection, 'connected')

				const result = await this.consumeStream(connection, stream)
				connection.controller = undefined
				if (connection.stopped) return

				if (result === 'protocol-reconnect') {
					connection.reconnectAttempts = 0
					continue
				}

				await this.waitForReconnect(connection)
			} catch (error) {
				connection.controller = undefined
				if (connection.stopped || this.isAbortError(error)) return

				connection.reconnectAttempts++
				this.updateStatus(connection, 'error', error)
				console.warn(`[Sync] Connection failed for server ${connection.serverId}:`, error)
				await this.waitForReconnect(connection)
			}
		}
	}

	private async consumeStream(
		connection: SyncConnection,
		stream: ReadableStream<Uint8Array>,
	): Promise<StreamReadResult> {
		const reader = stream.getReader()
		const decoder = new TextDecoder()
		const parser = new SseParser()

		try {
			while (!connection.stopped) {
				const { done, value } = await reader.read()
				if (done) break

				const chunk = decoder.decode(value, { stream: true })
				const result = this.processParsedItems(connection, parser.feed(chunk))
				if (result === 'protocol-reconnect') {
					await reader.cancel()
					connection.controller?.abort()
					return result
				}
			}

			const finalChunk = decoder.decode()
			const finalItems = finalChunk ? parser.feed(finalChunk) : []
			const result = this.processParsedItems(connection, [...finalItems, ...parser.end()])
			if (result === 'protocol-reconnect') {
				await reader.cancel()
				connection.controller?.abort()
				return result
			}
		} finally {
			reader.releaseLock()
		}

		return 'closed'
	}

	private processParsedItems(connection: SyncConnection, items: ParsedSseItem[]): StreamReadResult {
		for (const item of items) {
			if (item.kind === 'retry') {
				connection.retryDelay = Math.min(item.retry, MAX_RECONNECT_DELAY)
				continue
			}

			this.updateLastEventId(connection, item.id)

			const event = parseSyncEventData(item.data)
			if (!event) {
				console.warn('[Sync] Dropping malformed SSE payload:', {
					serverId: connection.serverId,
					event: item.event,
					data: item.data,
				})
				continue
			}

			this.emitSyncEvent(connection.serverId, event)

			if (event.type === 'protocol.reset' || event.type === 'protocol.invalid') {
				connection.lastEventId = undefined
				return 'protocol-reconnect'
			}
		}

		return 'closed'
	}

	private async waitForReconnect(connection: SyncConnection): Promise<void> {
		if (connection.stopped) return

		this.updateStatus(connection, 'reconnecting')
		const delay = this.getReconnectDelay(connection)

		await new Promise<void>((resolve) => {
			connection.reconnectResolve = resolve
			connection.reconnectTimer = setTimeout(() => {
				connection.reconnectTimer = undefined
				connection.reconnectResolve = undefined
				resolve()
			}, delay)
		})
	}

	private closeConnection(serverId: string): void {
		const connection = this.connections.get(serverId)
		if (!connection) return

		connection.stopped = true
		connection.controller?.abort()

		if (connection.reconnectTimer) {
			clearTimeout(connection.reconnectTimer)
			connection.reconnectTimer = undefined
		}
		connection.reconnectResolve?.()
		connection.reconnectResolve = undefined

		this.updateStatus(connection, 'disconnected')
		this.connections.delete(serverId)
	}

	private getReconnectDelay(connection: SyncConnection): number {
		const exponentialDelay =
			connection.retryDelay * Math.pow(2, Math.max(connection.reconnectAttempts - 1, 0))
		return Math.min(exponentialDelay, MAX_RECONNECT_DELAY) + Math.random() * JITTER_MS
	}

	private updateLastEventId(connection: SyncConnection, id: string | undefined): void {
		if (id === undefined) return
		connection.lastEventId = id || undefined
	}

	private intentToParam(intent: Archon.Sync.v1.SyncIntent): string {
		return Array.isArray(intent) ? intent.join(',') : intent
	}

	private isAbortError(error: unknown): boolean {
		if (!(error instanceof Error)) return false
		return error.name === 'AbortError' || error.message.toLowerCase().includes('abort')
	}
}
