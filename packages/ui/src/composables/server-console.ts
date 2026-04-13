import { createGlobalState } from '@vueuse/core'
import { type Ref, shallowRef, triggerRef } from 'vue'

import { detectLogLevel } from '../layouts/shared/console/composables/log-level'
import type { Log4jEvent, LogLevel, LogLine } from '../layouts/shared/console/types'

// Flip to true during development to enable console perf logging.
// Uses a plain constant to avoid turbo env-var declarations.
const DEBUG_PERF = false

// TODO: for true unbounded history, consider IndexedDB or similar
const ARCHIVE_CAPACITY = 500_000

const ENTRY_START_RE = /^\[\d{2}:\d{2}:\d{2}\]/

/**
 * Reorders a batch of log lines so that continuation lines (lines without a
 * timestamp prefix) stay grouped with their parent error/warn entry, even when
 * unrelated timestamped lines arrive between them from the server.
 */
function groupContinuations(lines: LogLine[]): LogLine[] {
	if (lines.length <= 1) return lines

	const groups: LogLine[][] = []

	for (const line of lines) {
		if (ENTRY_START_RE.test(line.text)) {
			groups.push([line])
		} else if (groups.length > 0) {
			let target = groups.length - 1
			const lastEntry = groups[target][0]

			if (lastEntry.level !== 'error' && lastEntry.level !== 'warn') {
				if (line.level === 'error' || line.level === null) {
					for (let i = groups.length - 2; i >= 0; i--) {
						if (groups[i][0].level === 'error' || groups[i][0].level === 'warn') {
							target = i
							break
						}
					}
				}
			}

			groups[target].push(line)
		} else {
			groups.push([line])
		}
	}

	return groups.flat()
}

const batchTimeout = 300
const initialBatchSize = 256

const LogLevelCode = {
	None: 0,
	Trace: 1,
	Debug: 2,
	Info: 3,
	Warn: 4,
	Error: 5,
} as const
type LogLevelCode = (typeof LogLevelCode)[keyof typeof LogLevelCode]

function encodeLevel(level: LogLevel | null): LogLevelCode {
	if (!level) return LogLevelCode.None
	switch (level) {
		case 'trace':
			return LogLevelCode.Trace
		case 'debug':
			return LogLevelCode.Debug
		case 'info':
			return LogLevelCode.Info
		case 'warn':
			return LogLevelCode.Warn
		case 'error':
			return LogLevelCode.Error
	}
}

function decodeLevel(code: LogLevelCode): LogLevel | null {
	switch (code) {
		case LogLevelCode.Trace:
			return 'trace'
		case LogLevelCode.Debug:
			return 'debug'
		case LogLevelCode.Info:
			return 'info'
		case LogLevelCode.Warn:
			return 'warn'
		case LogLevelCode.Error:
			return 'error'
		default:
			return null
	}
}

// Columnar ring buffer: stores text and level in parallel arrays instead of
// LogLine objects, eliminating ~40 bytes of object header per line (~20MB
// saved at 500k lines). Lines are stored by value — get(i) returns a fresh
// LogLine each call, so consumers must not rely on reference identity.
class ColumnarRingBuffer {
	texts: (string | undefined)[]
	levels: Uint8Array
	private head = 0
	private _size = 0

	constructor(readonly capacity: number) {
		this.texts = new Array(capacity)
		this.levels = new Uint8Array(capacity)
	}

	get size(): number {
		return this._size
	}

	push(text: string, level: LogLevel | null): boolean {
		const wrapped = this._size === this.capacity
		this.texts[this.head] = text
		this.levels[this.head] = encodeLevel(level)
		this.head = (this.head + 1) % this.capacity
		if (!wrapped) this._size++
		return wrapped
	}

	get(index: number): LogLine {
		if (index < 0 || index >= this._size) {
			throw new RangeError(`Index ${index} out of bounds [0, ${this._size})`)
		}
		const start = this._size === this.capacity ? this.head : 0
		const physical = (start + index) % this.capacity
		return {
			text: this.texts[physical] as string,
			level: decodeLevel(this.levels[physical] as LogLevelCode),
		}
	}

	toArray(): LogLine[] {
		if (this._size === 0) return []
		const start = this._size === this.capacity ? this.head : 0
		const result = new Array<LogLine>(this._size)
		for (let i = 0; i < this._size; i++) {
			const physical = (start + i) % this.capacity
			result[i] = {
				text: this.texts[physical] as string,
				level: decodeLevel(this.levels[physical] as LogLevelCode),
			}
		}
		return result
	}

	clear(): void {
		this.texts = new Array(this.capacity)
		this.levels = new Uint8Array(this.capacity)
		this.head = 0
		this._size = 0
	}
}

function mapLog4jLevel(level?: string): LogLevel | null {
	if (!level) return null
	switch (level.toUpperCase()) {
		case 'FATAL':
		case 'ERROR':
			return 'error'
		case 'WARN':
			return 'warn'
		case 'INFO':
			return 'info'
		case 'DEBUG':
			return 'debug'
		case 'TRACE':
			return 'trace'
		default:
			return null
	}
}

function formatTimestamp(millis?: number): string {
	if (!millis) return ''
	const date = new Date(millis)
	const h = String(date.getHours()).padStart(2, '0')
	const m = String(date.getMinutes()).padStart(2, '0')
	const s = String(date.getSeconds()).padStart(2, '0')
	return `[${h}:${m}:${s}]`
}

function formatLog4jLines(event: Log4jEvent): LogLine[] {
	const level = mapLog4jLevel(event.level)
	const time = formatTimestamp(event.timestamp_millis)
	const thread = event.thread_name ?? ''
	const levelStr = event.level ?? ''
	const message = event.message?.trim() ?? ''

	const prefix = time ? `${time} [${thread}/${levelStr}]: ` : `[${thread}/${levelStr}]: `
	const messageLines = message.split(/[\r\n]+/)
	const lines: LogLine[] = [{ text: prefix + messageLines[0], level }]
	for (let i = 1; i < messageLines.length; i++) {
		if (!messageLines[i]) continue
		lines.push({ text: messageLines[i], level })
	}

	if (event.throwable) {
		for (const line of event.throwable.split(/[\r\n]+/)) {
			if (!line) continue
			lines.push({ text: line, level: 'error' })
		}
	}

	return lines
}

function textToLogLine(text: string): LogLine {
	return { text, level: detectLogLevel(text) }
}

export function createConsoleState() {
	const archive = new ColumnarRingBuffer(ARCHIVE_CAPACITY)
	const output: Ref<LogLine[]> = shallowRef<LogLine[]>([])
	const WS_EVENT_HISTORY_MAX = 25000
	const wsEventHistory: unknown[] = []
	let wsEventCaptureEnabled = false

	let lineBuffer: LogLine[] = []
	let batchTimer: NodeJS.Timeout | null = null

	let wrapCount = 0
	let lastFlushMs = 0

	const flushBuffer = (): void => {
		if (lineBuffer.length === 0) return

		const t0 = DEBUG_PERF ? performance.now() : 0
		const arr = output.value
		const lines = groupContinuations(lineBuffer)
		const flushedCount = lines.length
		let didWrap = false

		for (const line of lines) {
			if (archive.push(line.text, line.level)) didWrap = true
			arr.push(line)
		}

		if (didWrap) {
			const evictedCount = Math.max(0, arr.length - archive.size)
			if (evictedCount > 0) {
				arr.splice(0, evictedCount)
			}
			wrapCount++
		}

		lineBuffer = []
		batchTimer = null
		triggerRef(output)

		if (DEBUG_PERF) {
			lastFlushMs = performance.now() - t0
			if (arr.length !== archive.size) {
				console.error(
					`[mr-console] drift: output.length=${arr.length} !== archive.size=${archive.size}`,
				)
			}
			console.debug(
				`[mr-console] flush: ${flushedCount} lines in ${lastFlushMs.toFixed(2)}ms` +
					` | buffer: ${archive.size} | wrap: ${didWrap}`,
			)
		}
	}

	const addLines = (lines: LogLine[]): void => {
		if (output.value.length === 0 && lines.length >= initialBatchSize) {
			lineBuffer = lines
			flushBuffer()
			return
		}

		lineBuffer.push(...lines)

		if (!batchTimer) {
			batchTimer = setTimeout(flushBuffer, batchTimeout)
		}
	}

	const addLog4jEvent = (event: Log4jEvent): void => {
		addLines(formatLog4jLines(event))
	}

	const recordWsEvent = (event: unknown): void => {
		if (!wsEventCaptureEnabled) return
		wsEventHistory.push(event)
		if (wsEventHistory.length > WS_EVENT_HISTORY_MAX) {
			wsEventHistory.splice(0, wsEventHistory.length - WS_EVENT_HISTORY_MAX)
		}
	}

	const getWsEventHistory = (): unknown[] => wsEventHistory.slice()

	const setWsEventCaptureEnabled = (enabled: boolean): void => {
		wsEventCaptureEnabled = enabled
		if (!enabled) wsEventHistory.length = 0
	}

	const addLegacyLog = (message: string): void => {
		const logLines = message
			.split(/[\r\n]+/)
			.filter((l) => l)
			.map(textToLogLine)

		let parentLevel: LogLevel | null = null
		for (const line of logLines) {
			if (ENTRY_START_RE.test(line.text)) {
				parentLevel = line.level
			} else if (line.level === null && parentLevel !== null) {
				line.level = parentLevel
			}
		}

		addLines(logLines)
	}

	const clear = (): void => {
		const t0 = DEBUG_PERF ? performance.now() : 0
		archive.clear()
		output.value = []
		lineBuffer = []
		wsEventHistory.length = 0
		wrapCount = 0
		if (batchTimer) {
			clearTimeout(batchTimer)
			batchTimer = null
		}
		if (DEBUG_PERF) {
			console.debug(`[mr-console] clear in ${(performance.now() - t0).toFixed(2)}ms`)
		}
	}

	const __debugStats = ():
		| { enabled: false }
		| {
				enabled: true
				bufferSize: number
				heapEstimate: number
				recentFlushMs: number
				wrapCount: number
		  } => {
		if (!DEBUG_PERF) return { enabled: false }
		const heapEstimate =
			archive.texts.reduce<number>((a, s) => a + (s?.length ?? 0) * 2, 0) +
			archive.levels.byteLength
		return {
			enabled: true,
			bufferSize: archive.size,
			heapEstimate,
			recentFlushMs: lastFlushMs,
			wrapCount,
		}
	}

	return {
		output,
		addLines,
		addLog4jEvent,
		addLegacyLog,
		recordWsEvent,
		getWsEventHistory,
		setWsEventCaptureEnabled,
		clear,
		__debugStats,
	}
}

export const useModrinthServersConsole = createGlobalState(createConsoleState)
