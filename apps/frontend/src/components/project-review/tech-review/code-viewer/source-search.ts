import type { SourceMatch } from './source-matches'

export type SourceMatchMode = 'closed' | 'search' | 'traces'

export interface SourceSearchOptions {
	matchCase: boolean
	wholeWord: boolean
	regex: boolean
}

/** Searches exact source text unless regular expressions are explicitly enabled. */
export function searchSource(lines: string[], query: string, options: SourceSearchOptions) {
	const matches = new Map<number, SourceMatch[]>()
	if (!query) return { matches, error: '' }
	let pattern: RegExp
	try {
		pattern = new RegExp(
			options.regex ? query : query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'),
			options.matchCase ? 'gu' : 'giu',
		)
	} catch (error) {
		return { matches, error: error instanceof Error ? error.message : 'Invalid regular expression' }
	}
	const wordCharacter = /[\p{L}\p{N}_$]/u
	for (const [lineNumber, line] of lines.entries()) {
		const ranges: SourceMatch[] = []
		for (const match of line.matchAll(pattern)) {
			if (!match[0].length) continue
			const start = match.index
			const end = start + match[0].length
			if (
				options.wholeWord &&
				((start > 0 && wordCharacter.test(line[start - 1])) ||
					(end < line.length && wordCharacter.test(line[end])))
			)
				continue
			ranges.push({ start, end })
		}
		if (ranges.length) matches.set(lineNumber, ranges)
	}
	return { matches, error: '' }
}
