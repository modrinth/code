import mitt from 'mitt'

import { AbstractWebSocketClient, type WebSocketConnection } from '../core/abstract-websocket'
import type { Archon } from '../modules/archon/types'
import { getNodeWebSocketUrl } from '../utils/node-url'

type WSEventMap = {
	[K in Archon.Websocket.v0.WSEvent as `${string}:${K['event']}`]: K
}

const NORMAL_CLOSURE = 1000

export class GenericWebSocketClient extends AbstractWebSocketClient {
	protected emitter = mitt<WSEventMap>()
	private authRefreshes = new Set<string>()

	connect(serverId: string, auth: Archon.Websocket.v0.WSAuth): Promise<void> {
		return this.openConnection(serverId, auth, 0)
	}

	private openConnection(
		serverId: string,
		auth: Archon.Websocket.v0.WSAuth,
		reconnectAttempts: number,
	): Promise<void> {
		if (this.connections.has(serverId)) {
			this.closeConnection(serverId)
		}

		return new Promise((resolve, reject) => {
			let settled = false
			let authenticationTimeout: ReturnType<typeof setTimeout> | null = null
			const resolveConnection = () => {
				if (settled) return
				settled = true
				if (authenticationTimeout) clearTimeout(authenticationTimeout)
				resolve()
			}
			const rejectConnection = (error: unknown) => {
				if (settled) return
				settled = true
				if (authenticationTimeout) clearTimeout(authenticationTimeout)
				reject(error)
			}
			try {
				const ws = new WebSocket(getNodeWebSocketUrl(auth.url))

				const connection: WebSocketConnection = {
					serverId,
					socket: ws,
					authenticated: false,
					reconnectAttempts,
					reconnectTimer: undefined,
					isReconnecting: reconnectAttempts > 0,
				}

				this.connections.set(serverId, connection)
				authenticationTimeout = setTimeout(() => {
					rejectConnection(new Error(`WebSocket authentication timed out for server ${serverId}`))
					if (this.connections.get(serverId) === connection) this.closeConnection(serverId)
				}, this.AUTHENTICATION_TIMEOUT)

				ws.onopen = () => {
					ws.send(JSON.stringify({ event: 'auth', jwt: auth.token }))
				}

				ws.onmessage = (messageEvent) => {
					if (this.connections.get(serverId) !== connection) return
					try {
						const data = JSON.parse(messageEvent.data) as Archon.Websocket.v0.WSEvent
						if (data.event === 'auth-ok') {
							if (connection.authenticated) return
							connection.authenticated = true
							connection.reconnectAttempts = 0
							connection.isReconnecting = false
						} else if (data.event === 'auth-incorrect') {
							connection.authenticated = false
						}

						const eventKey = `${serverId}:${data.event}` as keyof WSEventMap
						// eslint-disable-next-line @typescript-eslint/no-explicit-any
						this.emitter.emit(eventKey, data as any)

						if (data.event === 'auth-ok') resolveConnection()

						if (data.event === 'auth-expiring' || data.event === 'auth-incorrect') {
							this.handleAuthExpiring(serverId).catch(console.error)
						}
					} catch (error) {
						console.error('[WebSocket] Failed to parse message:', error)
					}
				}

				ws.onclose = (event) => {
					connection.authenticated = false
					console.debug(`[WebSocket] Closed for server ${serverId}:`, {
						code: event.code,
						reason: event.reason,
						wasClean: event.wasClean,
					})
					rejectConnection(
						new Error(
							`WebSocket closed before authentication for server ${serverId} (code: ${event.code})`,
						),
					)
					if (event.code !== NORMAL_CLOSURE && this.connections.get(serverId) === connection) {
						this.scheduleReconnect(connection)
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
					rejectConnection(
						new Error(
							`WebSocket connection failed for server ${serverId} (readyState: ${readyState})`,
						),
					)
				}
			} catch (error) {
				rejectConnection(error)
			}
		})
	}

	disconnect(serverId: string): void {
		this.closeConnection(serverId)

		this.emitter.all.forEach((_handlers, type) => {
			if (type.toString().startsWith(`${serverId}:`)) {
				this.emitter.all.delete(type)
			}
		})
	}

	private closeConnection(serverId: string): void {
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

		this.connections.delete(serverId)
	}

	closeSocket(serverId: string): void {
		this.closeConnection(serverId)
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

	private scheduleReconnect(connection: WebSocketConnection): void {
		const { serverId } = connection

		if (connection.reconnectAttempts >= this.MAX_RECONNECT_ATTEMPTS) {
			this.disconnect(serverId)
			return
		}

		connection.isReconnecting = true
		connection.reconnectAttempts++

		const delay = this.getReconnectDelay(connection.reconnectAttempts)

		connection.reconnectTimer = setTimeout(() => {
			connection.reconnectTimer = undefined
			void this.reconnect(connection)
		}, delay)
	}

	private async reconnect(connection: WebSocketConnection): Promise<void> {
		const { serverId } = connection
		try {
			const auth = await this.client.archon.servers_v0.getWebSocketAuth(serverId)
			if (this.connections.get(serverId) !== connection) return
			await this.openConnection(serverId, auth, connection.reconnectAttempts)
		} catch (error) {
			console.error(`[WebSocket] Reconnection failed for server ${serverId}:`, error)
			if (this.connections.get(serverId) === connection) this.scheduleReconnect(connection)
		}
	}

	private async handleAuthExpiring(serverId: string): Promise<void> {
		if (this.authRefreshes.has(serverId)) return
		this.authRefreshes.add(serverId)
		try {
			const newAuth = await this.client.archon.servers_v0.getWebSocketAuth(serverId)

			const connection = this.connections.get(serverId)
			if (connection && connection.socket.readyState === WebSocket.OPEN) {
				connection.socket.send(JSON.stringify({ event: 'auth', jwt: newAuth.token }))
			}
		} catch (error) {
			console.error(`[WebSocket] Failed to refresh auth for server ${serverId}:`, error)
			this.disconnect(serverId)
		} finally {
			this.authRefreshes.delete(serverId)
		}
	}
}
