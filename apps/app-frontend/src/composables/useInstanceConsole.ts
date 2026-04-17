import { createConsoleState } from '@modrinth/ui'

import { clear_log_buffer, get_live_log_buffer, get_logs } from '@/helpers/logs'

type ConsoleState = ReturnType<typeof createConsoleState>

interface LogEntry {
	filename: string
	name?: string
	log_type: string
	stdout?: string
	age?: number
	live?: boolean
}

interface InstanceConsoleEntry {
	liveConsole: ConsoleState
	historicalConsole: ConsoleState
	historicalCache: Map<string, string>
	logList: LogEntry[] | null
}

const instances = new Map<string, InstanceConsoleEntry>()

function getOrCreate(profilePathId: string): InstanceConsoleEntry {
	let entry = instances.get(profilePathId)
	if (entry) return entry

	entry = {
		liveConsole: createConsoleState(),
		historicalConsole: createConsoleState(),
		historicalCache: new Map(),
		logList: null,
	}
	instances.set(profilePathId, entry)
	return entry
}

async function hydrate(profilePathId: string): Promise<void> {
	const entry = getOrCreate(profilePathId)
	if (entry.liveConsole.output.value.length > 0) return

	const buffer = await get_live_log_buffer(profilePathId)
	if (buffer) {
		entry.liveConsole.addLegacyLog(buffer)
	}
}

async function getHistoricalLogs(profilePathId: string, instancePath: string): Promise<LogEntry[]> {
	const entry = getOrCreate(profilePathId)
	if (entry.logList) return entry.logList

	const logs: LogEntry[] = await get_logs(instancePath, false)
	entry.logList = logs

	for (const log of logs) {
		if (log.stdout && log.stdout !== '') {
			entry.historicalCache.set(log.filename, log.stdout)
		}
	}

	return logs
}

function getHistoricalContent(profilePathId: string, filename: string): string | undefined {
	return instances.get(profilePathId)?.historicalCache.get(filename)
}

function invalidate(profilePathId: string): void {
	const entry = instances.get(profilePathId)
	if (!entry) return
	entry.historicalCache.clear()
	entry.logList = null
}

async function clearLive(profilePathId: string): Promise<void> {
	const entry = getOrCreate(profilePathId)
	entry.liveConsole.clear()
	await clear_log_buffer(profilePathId).catch(() => {})
}

async function destroy(profilePathId: string): Promise<void> {
	instances.delete(profilePathId)
	await clear_log_buffer(profilePathId).catch(() => {})
}

export function useInstanceConsole(profilePathId: string) {
	const entry = getOrCreate(profilePathId)
	return {
		liveConsole: entry.liveConsole,
		historicalConsole: entry.historicalConsole,
		hydrate: () => hydrate(profilePathId),
		getHistoricalLogs: (instancePath: string) => getHistoricalLogs(profilePathId, instancePath),
		getHistoricalContent: (filename: string) => getHistoricalContent(profilePathId, filename),
		invalidate: () => invalidate(profilePathId),
		clearLive: () => clearLive(profilePathId),
		destroy: () => destroy(profilePathId),
	}
}
