/** Limits source downloads and drops queued requests when their queries are cancelled. */
export function createSourceRequestQueue(concurrency = 4) {
	let active = 0
	const pending: (() => void)[] = []

	function drain() {
		while (active < concurrency && pending.length) pending.shift()!()
	}

	return function requestSource<T>(request: () => Promise<T>, signal: AbortSignal): Promise<T> {
		return new Promise((resolve, reject) => {
			function cancel() {
				const index = pending.indexOf(start)
				if (index === -1) return
				pending.splice(index, 1)
				reject(signal.reason)
			}

			function start() {
				signal.removeEventListener('abort', cancel)
				if (signal.aborted) {
					reject(signal.reason)
					return
				}
				active++
				Promise.resolve()
					.then(request)
					.then(resolve, reject)
					.finally(() => {
						active--
						drain()
					})
			}

			if (signal.aborted) {
				reject(signal.reason)
				return
			}
			signal.addEventListener('abort', cancel, { once: true })
			pending.push(start)
			drain()
		})
	}
}
