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

const allowedStyles: Record<string, true | (string | RegExp)[]> = {
	'image-rendering': true,
	'text-align': true,
	float: true,
	border: true,
	'border-width': true,
	'border-style': true,
	'border-color': true,
	'border-radius': true,
	margin: [/^[^-]+$/],
	padding: [/^[^-]+$/],
	'text-transform': true,
	'line-height': true,
	'overflow-wrap': true,
	'word-break': true,
	'text-shadow': true,
	color: true,
	'font-family': true,
	'font-size': [/^(0\.[5-9]|[12](\.\d+)?|3)(em|rem)$/, /^([89]|[1-3]\d|4[0-8])px$/],
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
		const rules = allowedStyles[prop]
		const allowed =
			rules === true ||
			(rules?.some((rule) => (typeof rule === 'string' ? rule === val : rule.test(val))) ?? false)
		if (allowed) kept.push(`${prop}: ${val}`)
	}
	return kept.length ? kept.join('; ') : undefined
}

export const securityOptions = {
	allowedLinkPrefixes: ['https://', 'mailto:'],
	allowedImagePrefixes: ['https://', 'data:image/'],
	allowDataImages: true,
	blockedTags: ['style', 'script', 'head', 'body', 'html', 'base', 'meta'],
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

const knownHtmlTags = new Set(Object.keys(allowedAttributes))

export function security(options: Parameters<typeof comarkSecurity>[0] = securityOptions) {
	const securityPlugin = comarkSecurity(options)
	return defineComarkPlugin(() => ({
		name: 'security',
		async post(state) {
			await visitAsync(
				state.tree,
				(node) => typeof node !== 'string' && node[0] !== null,
				(node) => {
					const element = node as ElementNode
					const fromRawHtml = !!(element[1] as { $?: { html?: number } })?.$?.html
					if (fromRawHtml && !knownHtmlTags.has(element[0].toLowerCase())) return false
				},
			)

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
