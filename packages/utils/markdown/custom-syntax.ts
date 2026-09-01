import { defineComarkPlugin } from 'comark/parse'

export interface InlineMarkerConfig {
	markers: (string | [open: string, close: string])[]
	tag: string
	attrs?: Record<string, string>
}

interface MarkerCandidate {
	open: string
	close: string
	tag: string
	attrs: Record<string, string>
}

function buildLookup(configs: InlineMarkerConfig[]): Record<string, MarkerCandidate[]> {
	const lookup: Record<string, MarkerCandidate[]> = {}
	for (const { markers, tag, attrs = {} } of configs) {
		for (const marker of markers) {
			const [open, close] = Array.isArray(marker) ? marker : [marker, marker]
			;(lookup[open[0]] ??= []).push({ open, close, tag, attrs })
		}
	}
	for (const candidates of Object.values(lookup)) {
		candidates.sort((a, b) => b.open.length - a.open.length)
	}
	return lookup
}

function inlineMarkersRule(configs: InlineMarkerConfig[]) {
	const lookup = buildLookup(configs)

	return (md: any) => {
		md.inline.ruler.before(
			'text',
			'inline-markers',
			(state: any, silent: boolean) => {
				const start = state.pos
				const src = state.src
				const candidates = lookup[src[start]]
				const candidate = candidates?.find((c) => src.startsWith(c.open, start))
				if (!candidate) return false

				const { open, close, tag, attrs } = candidate
				const openEnd = start + open.length
				let pos = openEnd
				const max = state.posMax
				let closeStart = -1
				while (pos < max) {
					if (src.startsWith(close, pos)) {
						closeStart = pos
						break
					}
					pos++
				}
				if (closeStart === -1 || closeStart === openEnd) return false

				if (silent) return true

				state.push('mdc_inline_component', tag, 1)

				const oldPos = state.pos
				const oldPosMax = state.posMax
				state.pos = openEnd
				state.posMax = closeStart
				state.md.inline.tokenize(state)
				state.pos = oldPos
				state.posMax = oldPosMax

				state.push('mdc_inline_component', tag, -1)

				const attrEntries = Object.entries(attrs)
				if (attrEntries.length > 0) {
					const props = state.push('mdc_inline_props', 'span', 0)
					props.attrs = attrEntries
					props.hidden = true
				}

				state.pos = closeStart + close.length
				return true
			},
			{ alt: ['emphasis'] },
		)
	}
}

export const inlineMarkers = defineComarkPlugin((configs: InlineMarkerConfig[] = []) => ({
	name: 'inline-markers',
	markdownItPlugins: [inlineMarkersRule(configs)],
}))
