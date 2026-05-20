import type { IDecoration, Terminal } from '@xterm/xterm'
import { ref } from 'vue'

import type { LogLevel, LogLine } from '../types'

export type FilterPredicate = (line: LogLine) => boolean

export function colorize(line: LogLine, _searchQuery?: string): string {
	switch (line.level) {
		case 'error':
			return `\x1b[31;40m${line.text}\x1b[K\x1b[0m`
		case 'warn':
			return `\x1b[33;40m${line.text}\x1b[K\x1b[0m`
		case 'debug':
		case 'trace':
			return `\x1b[90m${line.text}\x1b[0m`
		default:
			return line.text
	}
}

const HIGHLIGHT_BG = '#ffd60a'
const HIGHLIGHT_FG = '#000000'

const terminalDecorations = new WeakMap<Terminal, IDecoration[]>()
const activeQueries = new WeakMap<Terminal, string>()
const highlightVersions = new WeakMap<Terminal, number>()

function getDecorationList(terminal: Terminal): IDecoration[] {
	let list = terminalDecorations.get(terminal)
	if (!list) {
		list = []
		terminalDecorations.set(terminal, list)
	}
	return list
}

function bumpVersion(terminal: Terminal): number {
	const next = (highlightVersions.get(terminal) ?? 0) + 1
	highlightVersions.set(terminal, next)
	return next
}

export function getHighlightVersion(terminal: Terminal): number {
	return highlightVersions.get(terminal) ?? 0
}

export function clearSearchHighlights(terminal: Terminal) {
	const existing = terminalDecorations.get(terminal)
	if (existing) {
		for (const d of existing) d.dispose()
		existing.length = 0
	}
	activeQueries.delete(terminal)
	bumpVersion(terminal)
}

function walkBackToLogicalStart(terminal: Terminal, row: number): number {
	const buffer = terminal.buffer.active
	let y = Math.max(0, row)
	while (y > 0) {
		const line = buffer.getLine(y)
		if (!line?.isWrapped) break
		y--
	}
	return y
}

function scanRange(
	terminal: Terminal,
	query: string,
	startRow: number,
	endRow: number,
	out: IDecoration[],
) {
	const buffer = terminal.buffer.active
	const cols = terminal.cols
	const cursorAbsolute = buffer.baseY + buffer.cursorY
	let y = startRow
	while (y <= endRow) {
		const head = buffer.getLine(y)
		if (!head) break
		const lineStart = y
		let text = head.translateToString(false)
		y++
		while (y < buffer.length) {
			const next = buffer.getLine(y)
			if (!next?.isWrapped) break
			text += next.translateToString(false)
			y++
		}
		const lower = text.toLowerCase()
		let pos = 0
		while (true) {
			const idx = lower.indexOf(query, pos)
			if (idx === -1) break
			let remaining = query.length
			let rowAbs = lineStart + Math.floor(idx / cols)
			let col = idx % cols
			while (remaining > 0) {
				const amount = Math.min(cols - col, remaining)
				const marker = terminal.registerMarker(rowAbs - cursorAbsolute)
				if (marker) {
					const decoration = terminal.registerDecoration({
						marker,
						x: col,
						width: amount,
						layer: 'top',
						backgroundColor: HIGHLIGHT_BG,
						foregroundColor: HIGHLIGHT_FG,
					})
					if (decoration) out.push(decoration)
				}
				remaining -= amount
				rowAbs++
				col = 0
			}
			pos = idx + query.length
		}
	}
}

export function applySearchHighlights(terminal: Terminal, query: string): number {
	const trimmed = query.trim().toLowerCase()
	const list = getDecorationList(terminal)
	for (const d of list) d.dispose()
	list.length = 0
	const version = bumpVersion(terminal)
	if (!trimmed) {
		activeQueries.delete(terminal)
		return version
	}
	activeQueries.set(terminal, trimmed)
	const endRow = terminal.buffer.active.length - 1
	scanRange(terminal, trimmed, 0, endRow, list)
	return version
}

export function highlightAppendedRange(terminal: Terminal, fromRow: number, version: number) {
	if (getHighlightVersion(terminal) !== version) return
	const query = activeQueries.get(terminal)
	if (!query) return
	const scanFrom = walkBackToLogicalStart(terminal, fromRow)
	const list = getDecorationList(terminal)
	const survivors: IDecoration[] = []
	for (const d of list) {
		if (d.marker.line >= scanFrom) {
			d.dispose()
		} else {
			survivors.push(d)
		}
	}
	list.length = 0
	list.push(...survivors)
	const endRow = terminal.buffer.active.length - 1
	if (scanFrom > endRow) return
	scanRange(terminal, query, scanFrom, endRow, list)
}

export type ConditionalLevel = 'debug' | 'trace'

export function useConsoleFilters() {
	const activeFilters = ref<Set<LogLevel | 'all'>>(new Set(['all']))

	function toggleFilter(level: LogLevel | 'all') {
		const next = new Set(activeFilters.value)
		if (level === 'all') {
			next.clear()
			next.add('all')
		} else {
			next.delete('all')
			if (next.has(level)) {
				next.delete(level)
			} else {
				next.add(level)
			}
			if (next.size === 0) {
				next.add('all')
			}
		}
		activeFilters.value = next
	}

	function buildFilterPredicate(): FilterPredicate | null {
		if (activeFilters.value.has('all')) return null
		const allowed = activeFilters.value
		return (line: LogLine) => {
			return allowed.has(line.level ?? 'info')
		}
	}

	return { activeFilters, toggleFilter, buildFilterPredicate }
}

export function rewriteTerminal(
	terminal: Terminal,
	allLines: LogLine[],
	predicate: FilterPredicate | null,
	searchQuery?: string,
	callback?: () => void,
) {
	clearSearchHighlights(terminal)
	terminal.reset()
	terminal.write('\x1b[?25l')

	const filtered = predicate ? allLines.filter(predicate) : allLines
	if (filtered.length === 0) {
		if (searchQuery) applySearchHighlights(terminal, searchQuery)
		callback?.()
		return
	}

	terminal.write('\x1b[?2026h')
	terminal.write(filtered.map((line) => colorize(line)).join('\r\n'), () => {
		terminal.write('\x1b[?2026l')
		if (searchQuery) {
			applySearchHighlights(terminal, searchQuery)
		}
		callback?.()
	})
}
