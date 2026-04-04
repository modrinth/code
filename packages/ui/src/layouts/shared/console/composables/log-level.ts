import type { LogLevel } from '../types'

const ERROR_TRIGGERS = ['/ERROR', 'Exception:', ':?]', 'Error', '[thread', '\tat']

export function detectLogLevel(lineText: string): LogLevel | null {
	if (lineText.includes('/INFO') || lineText.includes('[System] [CHAT]')) return 'info'
	if (lineText.includes('/WARN')) return 'warn'
	if (lineText.includes('/DEBUG')) return 'debug'
	if (lineText.includes('/TRACE')) return 'trace'
	for (const trigger of ERROR_TRIGGERS) {
		if (lineText.includes(trigger)) return 'error'
	}
	return null
}
