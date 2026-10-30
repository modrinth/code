export interface SourceMatch {
	start: number
	end: number
}

export function parseSourceHighlightWords(value: string): string[] {
	return [...new Set(value.split(/[,\r\n]+/).map((word) => word.trim()).filter(Boolean))]
}

function normalizeWord(word: unknown): string | null {
	if (typeof word !== 'string') return null
	let normalized = word
	if (!normalized.includes('://')) {
		if (normalized.includes('/')) normalized = normalized.split('/').pop() ?? ''
		normalized = normalized.replace(/\.class$/, '')
		normalized = normalized.replace(/\$/g, '.')
	}
	return normalized
}

/** Finds case-insensitive literal substrings, using simple names for Java class paths. */
export function findSourceMatches(lines: string[], value: string): Map<number, SourceMatch[]> {
	const text = normalizeWord(value.trim())
	const matches = new Map<number, SourceMatch[]>()
	if (!text) return matches
	const pattern = new RegExp(text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'gi')
	for (const [lineNumber, line] of lines.entries()) {
		const ranges = [...line.matchAll(pattern)].map((match) => ({
			start: match.index,
			end: match.index + match[0].length,
		}))
		if (ranges.length) matches.set(lineNumber, ranges)
	}
	return matches
}

/** Marks text nodes without changing syntax highlighting or interpreting trace values as HTML. */
export function markSourceLine(
	html: string,
	matches: SourceMatch[],
	document: Document,
	globalMatches: SourceMatch[] = [],
): string {
	const template = document.createElement('template')
	template.innerHTML = html
	const walker = document.createTreeWalker(template.content, 4)
	const nodes: Text[] = []
	while (walker.nextNode()) nodes.push(walker.currentNode as Text)
	let offset = 0
	for (const node of nodes) {
		const text = node.data
		const end = offset + text.length
		const ranges = matches
			.filter((match) => match.start < end && match.end > offset)
			.flatMap((match) => {
				const start = Math.max(offset, match.start)
				const finish = Math.min(end, match.end)
				const boundaries = [
					...new Set([
						start,
						finish,
						...globalMatches
							.flatMap((global) => [global.start, global.end])
							.filter((boundary) => boundary > start && boundary < finish),
					]),
				].sort((a, b) => a - b)
				return boundaries.slice(0, -1).map((boundary, index) => ({
					start: boundary,
					end: boundaries[index + 1],
				}))
			})
		if (ranges.length) {
			const fragment = document.createDocumentFragment()
			let cursor = 0
			for (const range of ranges) {
				const start = Math.max(0, range.start - offset)
				const finish = Math.min(text.length, range.end - offset)
				fragment.append(document.createTextNode(text.slice(cursor, start)))
				const mark = document.createElement('mark')
				mark.setAttribute('data-source-match', '')
				if (globalMatches.some((global) => global.start <= range.start && global.end >= range.end)) {
					mark.setAttribute('data-global-source-match', '')
				}
				mark.setAttribute('data-source-start', String(offset + start))
				mark.setAttribute('data-source-end', String(offset + finish))
				mark.textContent = text.slice(start, finish)
				fragment.append(mark)
				cursor = finish
			}
			fragment.append(document.createTextNode(text.slice(cursor)))
			node.replaceWith(fragment)
		}
		offset = end
	}
	return template.innerHTML
}

/** Collects string values from trace data for highlighting and source navigation. */
export function collectSourceValues(value: unknown): string[] {
	if (Array.isArray(value)) return value.flatMap((item) => collectSourceValues(item))
	if (value && typeof value === 'object') {
		return Object.values(value).flatMap((item) => collectSourceValues(item))
	}
	return typeof value === 'string' ? [value] : []
}

/** Combines overlapping highlights so each source character is marked only once. */
export function mergeSourceMatches(
	groups: Map<number, SourceMatch[]>[],
): Map<number, SourceMatch[]> {
	const lines = new Map<number, SourceMatch[]>()
	for (const group of groups) {
		for (const [line, ranges] of group) {
			lines.set(line, [...(lines.get(line) ?? []), ...ranges])
		}
	}
	for (const [line, ranges] of lines) {
		const merged: SourceMatch[] = []
		for (const range of ranges.sort((a, b) => a.start - b.start || b.end - a.end)) {
			const previous = merged[merged.length - 1]
			if (previous && range.start <= previous.end) previous.end = Math.max(previous.end, range.end)
			else merged.push({ ...range })
		}
		lines.set(line, merged)
	}
	return lines
}
