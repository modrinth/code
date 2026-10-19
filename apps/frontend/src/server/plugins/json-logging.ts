import { format } from 'node:util'

const levels = ['debug', 'info', 'log', 'warn', 'error'] as const

export default defineNitroPlugin(() => {
	if (import.meta.dev) return

	for (const level of levels) {
		const write = console[level].bind(console)
		console[level] = (...args: unknown[]) =>
			write(JSON.stringify({ time: new Date().toISOString(), level, message: format(...args) }))
	}
})
