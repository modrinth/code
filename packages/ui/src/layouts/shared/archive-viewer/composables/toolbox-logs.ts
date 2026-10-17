import { shallowRef } from 'vue'

import type { LogLevel } from '#ui/helpers/toolbox/misc/logging'

export interface ToolboxLogEntry {
	id: number
	time: Date
	level: LogLevel
	logger: string
	message: string
	error?: string
}

const MAX_ENTRIES = 1000

const entries = shallowRef<ToolboxLogEntry[]>([])
let nextId = 0
let capture: Promise<void> | null = null

function errorText(error: unknown) {
	if (error == null) return undefined
	return error instanceof Error ? (error.stack ?? error.message) : String(error)
}

/**
 * Starts keeping the toolbox's logs, including those its workers forward to the main thread.
 * Only the latest entries are kept. Safe to call more than once.
 */
export function startToolboxLogCapture(): Promise<void> {
	capture ??= import('#ui/helpers/toolbox/workers').then(({ messengerRegistry }) => {
		messengerRegistry.getLogFactory().onLog.subscribe((level, logger, message, extra) => {
			const entry: ToolboxLogEntry = {
				id: nextId++,
				time: new Date(),
				level,
				logger,
				message,
				error: errorText(extra?.err),
			}
			entries.value = [...entries.value.slice(-(MAX_ENTRIES - 1)), entry]
		})
	})
	return capture
}

export function useToolboxLogs() {
	return {
		entries,
		clear: () => (entries.value = []),
	}
}
