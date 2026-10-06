;(() => {
	const internals = window.__TAURI_INTERNALS__
	const label = internals?.metadata?.currentWebview?.label
	if (window.top !== window || label !== 'main') return
	if (typeof internals?.invoke !== 'function') return

	const invoke = internals.invoke

	function serialize(value) {
		try {
			if (value === null || typeof value !== 'object') return String(value)
			const seen = new WeakSet()
			return (
				JSON.stringify(value, (_, item) => {
					if (item instanceof Error) return item.stack || item.message
					if (typeof Element !== 'undefined' && item instanceof Element) return item.outerHTML
					if (['bigint', 'undefined', 'function', 'symbol'].includes(typeof item)) {
						return String(item)
					}
					if (item && typeof item === 'object') {
						if (seen.has(item)) return '[Circular]'
						seen.add(item)
					}
					return item
				}) ?? String(value)
			)
		} catch {
			return '[Unserializable value]'
		}
	}

	for (const method of ['log', 'info', 'warn', 'error', 'debug', 'trace']) {
		const original = console[method]
		if (typeof original !== 'function') continue

		console[method] = function (...args) {
			const result = original.apply(this, args)
			try {
				if (method === 'trace') args.push(new Error('console.trace').stack)
				void invoke('plugin:logs|logs_log_webview', {
					level: method === 'log' ? 'info' : method,
					message: args.map(serialize).join(' '),
				}).catch(() => {})
			} catch {}
			return result
		}
	}
})()
