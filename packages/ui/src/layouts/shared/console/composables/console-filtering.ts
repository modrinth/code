import type { Terminal } from '@xterm/xterm'
import { ref } from 'vue'
import type { LogLevel } from '../types'
import { detectLogLevel } from './log-level'

export type FilterPredicate = (lineText: string) => boolean

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
		return (line: string) => {
			const level = detectLogLevel(line)
			if (level === null) return true
			return allowed.has(level)
		}
	}

	return { activeFilters, toggleFilter, buildFilterPredicate }
}

export function rewriteTerminal(
	terminal: Terminal,
	allLines: string[],
	predicate: FilterPredicate | null,
) {
	terminal.clear()
	terminal.write('\x1b[?25l')

	const filtered = predicate ? allLines.filter(predicate) : allLines
	if (filtered.length === 0) return

	terminal.write('\x1b[?2026h')
	terminal.write(filtered.join('\r\n'))
	terminal.write('\x1b[?2026l')
}
