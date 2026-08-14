import type mitt from 'mitt'

import type { Archon } from '../modules/archon/types'

export type WebSocketEventHandler<
	E extends Archon.Websocket.v0.WSEvent = Archon.Websocket.v0.WSEvent,
> = (event: E) => void

export interface WebSocketConnection {
	serverId: string
	socket: WebSocket
	reconnectAttempts: number
	reconnectTimer?: ReturnType<typeof setTimeout>
	isReconnecting: boolean
}

export interface WebSocketStatus {
	connected: boolean
	reconnecting: boolean
	reconnectAttempts: number
}

type WSEventMap = {
	[K in Archon.Websocket.v0.WSEvent as `${string}:${K['event']}`]: K
}

export abstract class AbstractWebSocketClient {
	protected connections = new Map<string, WebSocketConnection>()
	protected abstract emitter: ReturnType<typeof mitt<WSEventMap>>

	protected readonly MAX_RECONNECT_ATTEMPTS = 10
	protected readonly RECONNECT_BASE_DELAY = 1000
	protected readonly RECONNECT_MAX_DELAY = 30000

	constructor(
		protected client: {
			archon: {
				servers_v0: {
					getWebSocketAuth: (serverId: string) => Promise<Archon.Websocket.v0.WSAuth>
				}
			}
		},
	) {}

	abstract connect(serverId: string, auth: Archon.Websocket.v0.WSAuth): Promise<void>

	abstract disconnect(serverId: string): void

	abstract disconnectAll(): void

	abstract send(serverId: string, message: Archon.Websocket.v0.WSOutgoingMessage): void

	async safeConnect(serverId: string, options?: { force?: boolean }): Promise<void> {
		const status = this.getStatus(serverId)

		if (status?.connected && !options?.force) {
			return
		}

		if (status && !status.connected && !options?.force) {
			return
		}

		if (options?.force && status) {
			this.disconnect(serverId)
		}

		const auth = await this.client.archon.servers_v0.getWebSocketAuth(serverId)
		await this.connect(serverId, auth)
	}

	on<E extends Archon.Websocket.v0.WSEventType>(
		serverId: string,
		eventType: E,
		handler: WebSocketEventHandler<Extract<Archon.Websocket.v0.WSEvent, { event: E }>>,
	): () => void {
		const eventKey = `${serverId}:${eventType}` as keyof WSEventMap

		this.emitter.on(eventKey, handler as () => void)

		return () => {
			this.emitter.off(eventKey, handler as () => void)
		}
	}

	getStatus(serverId: string): WebSocketStatus | null {
		const connection = this.connections.get(serverId)
		if (!connection) return null

		return {
			connected: connection.socket.readyState === WebSocket.OPEN,
			reconnecting: connection.isReconnecting,
			reconnectAttempts: connection.reconnectAttempts,
		}
	}

	protected getReconnectDelay(attempt: number): number {
		const delay = Math.min(
			this.RECONNECT_BASE_DELAY * Math.pow(2, attempt),
			this.RECONNECT_MAX_DELAY,
		)
		return delay + Math.random() * 1000
	}
}
