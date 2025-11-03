function getCallerLocation(): string {
	try {
		const stack = new Error().stack
		if (!stack) return ''

		const lines = stack.split('\n')
		const callerLine = lines[3]
		if (!callerLine) return ''

		const match = callerLine.match(/(https?:\/\/.+?|file:\/\/.+?|\/.*?):(\d+):\d+/)
		if (!match) return ''

		const [, fullPath, line] = match
		const fileName = fullPath.split('/').pop()?.split('?')[0] || fullPath
		return `${fileName}:${line}`
	} catch {
		return ''
	}
}

export function useDebugLogger(namespace: string) {
	// eslint-disable-next-line
	return (...args: any[]) => {
		const location = getCallerLocation()
		const prefix = location ? `[${namespace}] [${location}]` : `[${namespace}]`
		console.debug(prefix, ...args)
	}
}
