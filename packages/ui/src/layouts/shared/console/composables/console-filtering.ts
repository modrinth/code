import type { Terminal } from '@xterm/xterm'
import { ref } from 'vue'

import type { LogLevel, LogLine } from '../types'

export type FilterPredicate = (line: LogLine) => boolean

function highlightMatches(text: string, query: string): string {
	if (!query) return text
	const lower = text.toLowerCase()
	let result = ''
	let pos = 0
	while (pos < text.length) {
		const idx = lower.indexOf(query, pos)
		if (idx === -1) {
			result += text.slice(pos)
			break
		}
		result += text.slice(pos, idx)
		result += `\x1b[1;7m${text.slice(idx, idx + query.length)}\x1b[27;22m`
		pos = idx + query.length
	}
	return result
}

export function colorize(line: LogLine, searchQuery?: string): string {
	const text = searchQuery ? highlightMatches(line.text, searchQuery) : line.text
	switch (line.level) {
		case 'error':
			return `\x1b[31;40m${text}\x1b[K\x1b[0m`
		case 'warn':
			return `\x1b[33;40m${text}\x1b[K\x1b[0m`
		case 'debug':
		case 'trace':
			return `\x1b[90m${text}\x1b[0m`
		default:
			return text
	}
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
	terminal.reset()
	terminal.write('\x1b[?25l')

	const filtered = predicate ? allLines.filter(predicate) : allLines
	if (filtered.length === 0) {
		callback?.()
		return
	}

	terminal.write('\x1b[?2026h')
	terminal.write(filtered.map((line) => colorize(line, searchQuery)).join('\r\n') + '\r\n', () => {
		terminal.write('\x1b[?2026l')
		callback?.()
	})
}
