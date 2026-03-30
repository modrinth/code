export type LogLevel = 'error' | 'warn' | 'info' | 'debug' | 'trace'

export interface LogSource {
	id: string
	name: string
	live: boolean
}
