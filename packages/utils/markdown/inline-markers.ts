import { defineComarkPlugin } from 'comark/parse'

type MarkerSpec = string | { marker: string; double?: boolean }

interface InlineMarkerConfig {
	marker: MarkerSpec | MarkerSpec[]
	tag: string
	attrs?: Record<string, string>
}

function normalizeMarker(spec: MarkerSpec): { marker: string; double: boolean } {
	return typeof spec === 'string' ? { marker: spec, double: false } : { marker: spec.marker, double: spec.double ?? false }
}

function inlineMarkerRule({ marker, tag, attrs = {} }: InlineMarkerConfig) {
	const specs = (Array.isArray(marker) ? marker : [marker]).map(normalizeMarker)

	return (md: any) => {
		for (const { marker, double } of specs) {
			const markerCode = marker.charCodeAt(0)
			const markerLen = double ? 2 : 1
			const ruleName = `inline_${tag}_${markerCode}${double ? '_double' : ''}`

			md.inline.ruler.before(
				'text',
				ruleName,
				(state: any, silent: boolean) => {
					const start = state.pos
					const src = state.src
					if (src.charCodeAt(start) !== markerCode) return false
					if (double !== (src.charCodeAt(start + 1) === markerCode)) return false

					const openEnd = start + markerLen
					let pos = openEnd
					const max = state.posMax
					let closeStart = -1
					while (pos < max) {
						if (
							src.charCodeAt(pos) === markerCode &&
							(!double || src.charCodeAt(pos + 1) === markerCode)
						) {
							closeStart = pos
							break
						}
						pos++
					}
					if (closeStart === -1 || closeStart === openEnd) return false

					if (silent) return true

					const content = src.slice(openEnd, closeStart)
					state.push('mdc_inline_component', tag, 1)
					const text = state.push('text', '', 0)
					text.content = content
					state.push('mdc_inline_component', tag, -1)
					const attrEntries = Object.entries(attrs)
					if (attrEntries.length > 0) {
						const props = state.push('mdc_inline_props', 'span', 0)
						props.attrs = attrEntries
						props.hidden = true
					}

					state.pos = closeStart + markerLen
					return true
				},
				{ alt: ['emphasis'] },
			)
		}
	}
}

export const inlineMarkers = defineComarkPlugin(() => ({
	name: 'inline-markers',
	markdownItPlugins: [
		inlineMarkerRule({ marker: '^', tag: 'sup' }),
		inlineMarkerRule({ marker: '~', tag: 'sub' }),
		inlineMarkerRule({
			marker: [
				{ marker: '!', double: true },
				{ marker: '|', double: true },
			],
			tag: 'span',
			attrs: { class: 'spoiler', tabindex: '0' },
		}),
	],
}))
