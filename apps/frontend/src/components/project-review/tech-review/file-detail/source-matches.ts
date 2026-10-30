export interface SourceMatch {
	start: number
	end: number
}

/** Matches scanner class paths against both qualified and simple Java names. */
export function findSourceMatches(lines: string[], value: string): Map<number, SourceMatch[]> {
	const text = value.trim()
	const matches = new Map<number, SourceMatch[]>()
	if (!text) return matches
	const terms = new Set([text])
	if (/^\/?(?:[\w$]+\/)*[\w$]+(?:\.class)?$/.test(text) && text.includes('/')) {
		const qualified = text
			.replace(/^\/+/, '')
			.replace(/\.class$/, '')
			.replaceAll('/', '.')
		terms.add(qualified)
		terms.add(qualified.split('.').pop() ?? qualified)
	}
	const patterns = [...terms].filter(Boolean).map((term) => ({
		term,
		identifier: /^[A-Za-z_$][\w$]*(?:\.[A-Za-z_$][\w$]*)*$/.test(term),
	}))
	for (const [lineNumber, line] of lines.entries()) {
		const ranges: SourceMatch[] = []
		for (const { term, identifier } of patterns) {
			let start = line.indexOf(term)
			while (start !== -1) {
				const end = start + term.length
				if (
					!identifier ||
					(!/[\w$]/.test(line[start - 1] ?? '') && !/[\w$]/.test(line[end] ?? ''))
				) {
					ranges.push({ start, end })
				}
				start = line.indexOf(term, end)
			}
		}
		const merged: SourceMatch[] = []
		for (const range of ranges.sort((a, b) => a.start - b.start || b.end - a.end)) {
			const previous = merged[merged.length - 1]
			if (previous && range.start <= previous.end) previous.end = Math.max(previous.end, range.end)
			else merged.push({ ...range })
		}
		if (merged.length) matches.set(lineNumber, merged)
	}
	return matches
}

/** Marks text nodes without changing syntax highlighting or interpreting trace values as HTML. */
export function markSourceLine(html: string, matches: SourceMatch[], document: Document): string {
	const template = document.createElement('template')
	template.innerHTML = html
	const walker = document.createTreeWalker(template.content, 4)
	const nodes: Text[] = []
	while (walker.nextNode()) nodes.push(walker.currentNode as Text)
	let offset = 0
	for (const node of nodes) {
		const text = node.data
		const end = offset + text.length
		const ranges = matches.filter((match) => match.start < end && match.end > offset)
		if (ranges.length) {
			const fragment = document.createDocumentFragment()
			let cursor = 0
			for (const range of ranges) {
				const start = Math.max(0, range.start - offset)
				const finish = Math.min(text.length, range.end - offset)
				fragment.append(document.createTextNode(text.slice(cursor, start)))
				const mark = document.createElement('mark')
				mark.setAttribute('data-source-match', '')
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

/** Collects the list entries and URL fields that can navigate to source matches. */
export function collectSourceValues(value: unknown, matchSource = false): string[] {
	if (Array.isArray(value)) return value.flatMap((item) => collectSourceValues(item, true))
	if (value && typeof value === 'object') {
		return Object.entries(value).flatMap(([key, item]) =>
			collectSourceValues(item, matchSource || key.toLowerCase() === 'url'),
		)
	}
	if (typeof value !== 'string') return []
	return matchSource || /^https?:\/\//i.test(value) ? [value] : []
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
