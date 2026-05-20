export type LogLevel = 'error' | 'warn' | 'info' | 'debug' | 'trace'

export interface LogLine {
	text: string
	level: LogLevel | null
}

export interface Log4jEvent {
	logger_name?: string
	level?: string
	thread_name?: string
	timestamp_millis?: number
	message?: string
	throwable?: string
}

export interface LogSource {
	id: string
	name: string
	live: boolean
}
