export interface WebSocketPingOptions {
	count?: number
	intervalMs?: number
	settleDelayMs?: number
	timeoutMs?: number
	signal?: AbortSignal
}

export async function pingWebSocketUrl(
	url: string,
	options: WebSocketPingOptions = {},
): Promise<number> {
	const count = options.count ?? 5
	const intervalMs = options.intervalMs ?? 200
	const settleDelayMs = options.settleDelayMs ?? 1000
	const timeoutMs = options.timeoutMs ?? count * intervalMs + settleDelayMs + 1000

	if (options.signal?.aborted) return -1

	return await new Promise<number>((resolve) => {
		const samples: number[] = []
		const timers = new Set<ReturnType<typeof setTimeout>>()
		let socket: WebSocket | undefined
		let settled = false

		const setTrackedTimeout = (callback: () => void, ms: number) => {
			const timer = setTimeout(() => {
				timers.delete(timer)
				callback()
			}, ms)
			timers.add(timer)
			return timer
		}

		const cleanup = () => {
			for (const timer of timers) clearTimeout(timer)
			timers.clear()
			options.signal?.removeEventListener('abort', abort)

			if (
				socket &&
				(socket.readyState === WebSocket.CONNECTING || socket.readyState === WebSocket.OPEN)
			) {
				socket.close()
			}
		}

		const finish = (ping: number) => {
			if (settled) return
			settled = true
			cleanup()
			resolve(ping)
		}

		const abort = () => finish(-1)
		options.signal?.addEventListener('abort', abort, { once: true })

		try {
			socket = new WebSocket(url)
		} catch {
			finish(-1)
			return
		}

		setTrackedTimeout(() => finish(-1), timeoutMs)

		socket.onopen = () => {
			for (let i = 0; i < count; i++) {
				setTrackedTimeout(() => {
					if (socket?.readyState === WebSocket.OPEN) {
						socket.send(String(performance.now()))
					}
				}, i * intervalMs)
			}

			setTrackedTimeout(
				() => {
					const ping =
						samples.length > 0
							? Math.round([...samples].sort((a, b) => a - b)[Math.floor(samples.length / 2)])
							: -1
					finish(ping)
				},
				count * intervalMs + settleDelayMs,
			)
		}

		socket.onmessage = (event) => {
			samples.push(performance.now() - Number(event.data))
		}

		socket.onerror = () => finish(-1)
		socket.onclose = () => {
			if (!settled && samples.length === 0) finish(-1)
		}
	})
}
