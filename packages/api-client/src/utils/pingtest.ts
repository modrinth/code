export interface WebSocketPingOptions {
	count?: number
	intervalMs?: number
	settleDelayMs?: number
	signal?: AbortSignal
}

/**
 * Measures the latency to a ping test WebSocket, which echoes back every message. The connection stays open between
 * measurements until {@link WebSocketPinger.close} is called.
 */
export class WebSocketPinger {
	/** Resolves to whether the connection opened within the connect timeout. */
	public readonly connected: Promise<boolean>

	private readonly socket: WebSocket | null
	private sampleListener: ((sentAt: number, receivedAt: number) => void) | null = null
	private closed = false

	constructor(url: string, connectTimeoutMs: number) {
		let socket: WebSocket | null = null
		try {
			socket = new WebSocket(url)
		} catch {
			this.closed = true
		}
		this.socket = socket

		this.connected = new Promise<boolean>((resolve) => {
			if (!socket) {
				resolve(false)
				return
			}

			const timeout = setTimeout(() => {
				resolve(false)
				this.close()
			}, connectTimeoutMs)

			socket.onopen = () => {
				clearTimeout(timeout)
				resolve(true)
			}
			socket.onerror = () => {
				clearTimeout(timeout)
				resolve(false)
				this.close()
			}
			socket.onclose = () => {
				clearTimeout(timeout)
				this.closed = true
				resolve(false)
			}
			socket.onmessage = (event) => {
				this.sampleListener?.(Number(event.data), performance.now())
			}
		})
	}

	public get isOpen(): boolean {
		return !this.closed && this.socket?.readyState === WebSocket.OPEN
	}

	/**
	 * Sends `count` pings over the connection and resolves to the latency in ms of every ping that came back, once
	 * all of them came back or `settleDelayMs` after the last one was sent.
	 */
	public async measure(options: WebSocketPingOptions = {}): Promise<number[]> {
		const count = options.count ?? 5
		const intervalMs = options.intervalMs ?? 200
		const settleDelayMs = options.settleDelayMs ?? 1000
		const signal = options.signal

		if (!(await this.connected) || !this.isOpen || signal?.aborted) return []

		return await new Promise<number[]>((resolve) => {
			const startedAt = performance.now()
			const samples: number[] = []
			const timers: ReturnType<typeof setTimeout>[] = []

			const finish = (result: number[]) => {
				for (const timer of timers) clearTimeout(timer)
				signal?.removeEventListener('abort', abort)
				this.sampleListener = null
				resolve(result)
			}
			const abort = () => finish([])
			signal?.addEventListener('abort', abort, { once: true })

			this.sampleListener = (sentAt, receivedAt) => {
				if (sentAt < startedAt) return
				samples.push(receivedAt - sentAt)
				if (samples.length === count) finish(samples)
			}

			for (let i = 0; i < count; i++) {
				timers.push(
					setTimeout(() => {
						if (this.isOpen) this.socket?.send(String(performance.now()))
					}, i * intervalMs),
				)
			}

			timers.push(setTimeout(() => finish(samples), (count - 1) * intervalMs + settleDelayMs))
		})
	}

	public close() {
		this.closed = true
		if (
			this.socket &&
			(this.socket.readyState === WebSocket.CONNECTING || this.socket.readyState === WebSocket.OPEN)
		) {
			this.socket.close()
		}
	}
}
