import mitt from 'mitt'

import { AbstractWebSocketClient, type WebSocketConnection } from '../core/abstract-websocket'
import type { Archon } from '../modules/archon/types'

type WSEventMap = {
	[K in Archon.Websocket.v0.WSEvent as `${string}:${K['event']}`]: K
}

const NORMAL_CLOSURE = 1000

export class GenericWebSocketClient extends AbstractWebSocketClient {
	protected emitter = mitt<WSEventMap>()

	async connect(serverId: string, auth: Archon.Websocket.v0.WSAuth): Promise<void> {
		if (this.connections.has(serverId)) {
			this.disconnect(serverId)
		}

		return new Promise((resolve, reject) => {
			try {
				const ws = new WebSocket(`wss://${auth.url}`)

				const connection: WebSocketConnection = {
					serverId,
					socket: ws,
					reconnectAttempts: 0,
					reconnectTimer: undefined,
					isReconnecting: false,
				}

				this.connections.set(serverId, connection)

				ws.onopen = () => {
					ws.send(JSON.stringify({ event: 'auth', jwt: auth.token }))

					connection.reconnectAttempts = 0
					connection.isReconnecting = false

					resolve()
				}

				ws.onmessage = (messageEvent) => {
					try {
						const data = JSON.parse(messageEvent.data) as Archon.Websocket.v0.WSEvent

						const eventKey = `${serverId}:${data.event}` as keyof WSEventMap
						// eslint-disable-next-line @typescript-eslint/no-explicit-any
						this.emitter.emit(eventKey, data as any)

						if (data.event === 'auth-expiring' || data.event === 'auth-incorrect') {
							this.handleAuthExpiring(serverId).catch(console.error)
						}
					} catch (error) {
						console.error('[WebSocket] Failed to parse message:', error)
					}
				}

				ws.onclose = (event) => {
					console.debug(`[WebSocket] Closed for server ${serverId}:`, {
						code: event.code,
						reason: event.reason,
						wasClean: event.wasClean,
					})
					if (event.code !== NORMAL_CLOSURE) {
						this.scheduleReconnect(serverId, auth)
					}
				}

				ws.onerror = (event) => {
					const url = ws.url
					const readyState = ws.readyState
					console.error(`[WebSocket] Error for server ${serverId}:`, {
						url,
						readyState,
						readyStateLabel: ['CONNECTING', 'OPEN', 'CLOSING', 'CLOSED'][readyState],
						type: (event as Event).type,
					})
					reject(
						new Error(
							`WebSocket connection failed for server ${serverId} (readyState: ${readyState})`,
						),
					)
				}
			} catch (error) {
				reject(error)
			}
		})
	}

	disconnect(serverId: string): void {
		const connection = this.connections.get(serverId)
		if (!connection) return

		if (connection.reconnectTimer) {
			clearTimeout(connection.reconnectTimer)
			connection.reconnectTimer = undefined
		}

		if (
			connection.socket.readyState === WebSocket.OPEN ||
			connection.socket.readyState === WebSocket.CONNECTING
		) {
			connection.socket.close(NORMAL_CLOSURE, 'Client disconnecting')
		}

		this.emitter.all.forEach((_handlers, type) => {
			if (type.toString().startsWith(`${serverId}:`)) {
				this.emitter.all.delete(type)
			}
		})

		this.connections.delete(serverId)
	}

	disconnectAll(): void {
		for (const serverId of this.connections.keys()) {
			this.disconnect(serverId)
		}
	}

	send(serverId: string, message: Archon.Websocket.v0.WSOutgoingMessage): void {
		const connection = this.connections.get(serverId)
		if (!connection || connection.socket.readyState !== WebSocket.OPEN) {
			console.warn(`Cannot send message: WebSocket not connected for server ${serverId}`)
			return
		}
		connection.socket.send(JSON.stringify(message))
	}

	private scheduleReconnect(serverId: string, auth: Archon.Websocket.v0.WSAuth): void {
		const connection = this.connections.get(serverId)
		if (!connection) return

		if (connection.reconnectAttempts >= this.MAX_RECONNECT_ATTEMPTS) {
			this.disconnect(serverId)
			return
		}

		connection.isReconnecting = true
		connection.reconnectAttempts++

		const delay = this.getReconnectDelay(connection.reconnectAttempts)

		connection.reconnectTimer = setTimeout(() => {
			this.connect(serverId, auth).catch((error) => {
				console.error(`[WebSocket] Reconnection failed for server ${serverId}:`, error)
			})
		}, delay)
	}

	private async handleAuthExpiring(serverId: string): Promise<void> {
		try {
			const newAuth = await this.client.archon.servers_v0.getWebSocketAuth(serverId)

			const connection = this.connections.get(serverId)
			if (connection && connection.socket.readyState === WebSocket.OPEN) {
				connection.socket.send(JSON.stringify({ event: 'auth', jwt: newAuth.token }))
			}
		} catch (error) {
			console.error(`[WebSocket] Failed to refresh auth for server ${serverId}:`, error)
			this.disconnect(serverId)
		}
	}
}
