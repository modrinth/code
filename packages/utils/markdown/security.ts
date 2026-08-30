import type { ElementNode } from 'comark'
import { defineComarkPlugin } from 'comark/parse'
import comarkSecurity from 'comark/plugins/security'
import { visitAsync } from 'comark/utils'

export const allowedAttributes: Record<string, string[]> = {
	a: ['href', 'target', 'title', 'rel'],
	abbr: ['title'],
	address: [],
	area: ['shape', 'coords', 'href', 'alt'],
	article: [],
	aside: [],
	audio: ['autoplay', 'controls', 'crossorigin', 'loop', 'muted', 'preload', 'src'],
	b: [],
	bdi: ['dir'],
	bdo: ['dir'],
	big: [],
	blockquote: ['cite', 'as', 'type', 'title', 'foldable', 'open', 'noBody'],
	br: [],
	caption: [],
	center: [],
	cite: [],
	code: ['class'],
	col: ['align', 'valign', 'span', 'width'],
	colgroup: ['align', 'valign', 'span', 'width'],
	dd: [],
	del: ['datetime'],
	details: ['open', 'class'],
	div: ['align', 'class', 'id'],
	dl: [],
	dt: [],
	em: [],
	figcaption: [],
	figure: [],
	font: ['color', 'size', 'face'],
	footer: [],
	h1: ['id'],
	h2: ['id'],
	h3: ['id'],
	h4: ['id'],
	h5: ['id'],
	h6: ['id'],
	header: [],
	hr: [],
	i: [],
	iframe: ['src', 'width', 'height', 'allowfullscreen', 'frameborder'],
	img: ['src', 'alt', 'title', 'width', 'height', 'loading', 'usemap', 'style', 'align'],
	input: ['checked', 'disabled', 'type', 'class'],
	ins: ['datetime'],
	kbd: ['id'],
	li: ['class'],
	map: ['name'],
	mark: [],
	nav: [],
	ol: [],
	p: ['align'],
	picture: [],
	pre: ['language'],
	s: [],
	section: [],
	small: [],
	source: ['media', 'sizes', 'src', 'srcset', 'type'],
	span: ['class', 'tabindex'],
	strike: [],
	strong: [],
	sub: [],
	summary: [],
	sup: [],
	table: ['width', 'border', 'align', 'valign'],
	tbody: ['align', 'valign'],
	td: ['width', 'rowspan', 'colspan', 'align', 'valign', 'style'],
	tfoot: ['align', 'valign'],
	th: ['width', 'rowspan', 'colspan', 'align', 'valign', 'style'],
	thead: ['align', 'valign'],
	tr: ['rowspan', 'align', 'valign'],
	tt: [],
	u: [],
	ul: ['class'],
	video: [
		'autoplay',
		'controls',
		'crossorigin',
		'loop',
		'muted',
		'playsinline',
		'poster',
		'preload',
		'src',
		'height',
		'width',
	],
}

const allowedStyles: Record<string, string[]> = {
	'image-rendering': ['pixelated'],
	'text-align': ['center', 'left', 'right', 'justify'],
	float: ['left', 'right'],
}

const allowedStylePatterns: Record<string, RegExp> = {
	display: /^(flex|grid)$/,
	'flex-direction': /^(row|column|row-reverse|column-reverse)$/,
	'flex-wrap': /^(wrap|nowrap|wrap-reverse)$/,
	flex: /^(1 1 0%|1 1 auto|0 1 auto|none)$/,
	'flex-grow': /^[01]$/,
	'flex-shrink': /^[01]$/,
	order: /^-?\d+$/,
	'grid-auto-flow': /^(row|column|dense|row dense|column dense)$/,
	'grid-auto-columns': /^(auto|min-content|max-content|minmax\(0,1fr\))$/,
	'grid-auto-rows': /^(auto|min-content|max-content|minmax\(0,1fr\))$/,
	'grid-template-columns': /^(none|repeat\(\d+,minmax\(0,1fr\)\))$/,
	'grid-template-rows': /^(none|repeat\(\d+,minmax\(0,1fr\)\))$/,
	'grid-column': /^(1 \/ -1|span \d+ \/ span \d+)$/,
	'grid-row': /^(1 \/ -1|span \d+ \/ span \d+)$/,
	'grid-column-start': /^\d+$/,
	'grid-column-end': /^\d+$/,
	'grid-row-start': /^\d+$/,
	'grid-row-end': /^\d+$/,
	columns: /^\d+$/,
	'column-span': /^all$/,
	'justify-content': /^(flex-start|flex-end|center|space-between|space-around|space-evenly|stretch)$/,
	'justify-items': /^(start|end|center|stretch)$/,
	'justify-self': /^(auto|start|end|center|stretch)$/,
	'align-content': /^(flex-start|flex-end|center|space-between|space-around|space-evenly|stretch)$/,
	'align-items': /^(flex-start|flex-end|center|baseline|stretch)$/,
	'align-self': /^(auto|flex-start|flex-end|center|baseline|stretch)$/,
	'place-content': /^(start|end|center|space-between|space-around|space-evenly|stretch)$/,
	'place-items': /^(start|end|center|stretch)$/,
	'place-self': /^(auto|start|end|center|stretch)$/,
	'aspect-ratio': /^(auto|1 \/ 1|16 \/ 9)$/,
	'break-after': /^(auto|avoid|all|avoid-page|page|left|right|column)$/,
	'break-before': /^(auto|avoid|all|avoid-page|page|left|right|column)$/,
	'break-inside': /^(auto|avoid|avoid-page|avoid-column)$/,
	gap: /^(1px|[\d.]+rem)$/,
	'column-gap': /^(1px|[\d.]+rem)$/,
	'row-gap': /^(1px|[\d.]+rem)$/,
}

function filterStyleValue(value: unknown): string | undefined {
	if (typeof value !== 'string') return undefined
	const kept: string[] = []
	for (const declaration of value.split(';')) {
		const idx = declaration.indexOf(':')
		if (idx === -1) continue
		const prop = declaration.slice(0, idx).trim().toLowerCase()
		const val = declaration
			.slice(idx + 1)
			.trim()
			.toLowerCase()
		if (!prop || !val) continue
		if (allowedStyles[prop]?.includes(val) || allowedStylePatterns[prop]?.test(val)) kept.push(`${prop}: ${val}`)
	}
	return kept.length ? kept.join('; ') : undefined
}

export const securityOptions = {
	allowedLinkPrefixes: ['https://', 'mailto:'],
	allowedImagePrefixes: ['https://', 'data:image/'],
	allowDataImages: true,
}

function filterElementAttrs(tag: string, attrs: Record<string, unknown>) {
	const knownAttrs = allowedAttributes[tag]

	for (const key of Object.keys(attrs)) {
		const bareKey = key.startsWith(':') ? key.slice(1) : key

		if (bareKey === 'style') {
			const filtered = filterStyleValue(attrs[key])
			if (filtered) attrs[key] = filtered
			else delete attrs[key]
			continue
		}

		if (knownAttrs && !knownAttrs.includes(bareKey)) delete attrs[key]
	}
}

export function security(options: Parameters<typeof comarkSecurity>[0] = securityOptions) {
	const securityPlugin = comarkSecurity(options)
	return defineComarkPlugin(() => ({
		name: 'security',
		async post(state) {
			await securityPlugin.post?.(state)

			await visitAsync(
				state.tree,
				(node) => typeof node !== 'string' && node[0] !== null,
				(node) => {
					const element = node as ElementNode
					filterElementAttrs(element[0].toLowerCase(), element[1])
				},
			)
		},
	}))()
}
