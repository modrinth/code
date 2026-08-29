import type { ElementNode } from 'comark'
import { defineComarkPlugin } from 'comark/parse'
import security from 'comark/plugins/security'
import { visitAsync } from 'comark/utils'

const allowedClassPrefixes = ['hljs-', 'language-']

const taskListClassByTag: Record<string, string> = {
	input: 'task-list-item-checkbox',
	li: 'task-list-item',
	ul: 'contains-task-list',
}

function filterClassValue(value: unknown, tag?: string): string | undefined {
	if (typeof value !== 'string') return undefined
	const exact = tag ? taskListClassByTag[tag] : undefined
	const kept = value
		.split(/\s+/)
		.filter((cls) => cls === exact || allowedClassPrefixes.some((prefix) => cls.startsWith(prefix)))
	return kept.length ? kept.join(' ') : undefined
}

const allowedStyleDeclarations: [RegExp, RegExp][] = [
	[/^image-rendering$/, /^pixelated$/],
	[/^text-align$/, /^(center|left|right)$/],
	[/^float$/, /^(left|right)$/],
]

export function filterStyleValue(value: unknown): string | undefined {
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
		const match = allowedStyleDeclarations.find(([propPattern]) => propPattern.test(prop))
		if (match && match[1].test(val)) kept.push(`${prop}: ${val}`)
	}
	return kept.length ? kept.join('; ') : undefined
}

export const attributeAllowlist: Record<string, string[]> = {
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
	blockquote: ['cite', 'as'],
	br: [],
	caption: [],
	center: [],
	cite: [],
	code: ['class'],
	col: ['align', 'valign', 'span', 'width'],
	colgroup: ['align', 'valign', 'span', 'width'],
	collection: ['id'],
	dd: [],
	del: ['datetime'],
	details: ['open'],
	div: ['align'],
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
	mermaid: ['content', 'theme', 'theme-dark', 'class'],
	nav: [],
	ol: [],
	organization: ['id'],
	p: ['align'],
	picture: [],
	// Comark's own fence parsing puts the language on both `pre.language` and
	// `code`'s `class="language-x"` — allowing it directly here lets the
	// highlight.js renderer read `pre.language` straight off the node instead
	// of regex-parsing it back out of the class string.
	pre: ['language'],
	project: ['id'],
	s: [],
	section: [],
	small: [],
	source: ['media', 'sizes', 'src', 'srcset', 'type'],
	span: ['class'],
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
	user: ['id'],
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

const allAllowedTags = Object.keys(attributeAllowlist)

export const securityOptions = {
	allowedProtocols: ['http', 'https', 'mailto'],
	allowDataImages: true,
	allowedTags: allAllowedTags,
}

export function modrinthSecurity(options: Parameters<typeof security>[0] = securityOptions) {
	const securityPlugin = security(options)
	return defineComarkPlugin(() => ({
		name: 'modrinth-security',
		async post(state) {
			await securityPlugin.post?.(state)

			await visitAsync(
				state.tree,
				(node) => typeof node !== 'string' && node[0] !== null,
				(node) => {
					const element = node as ElementNode
					const tag = element[0].toLowerCase()
					const attrs = element[1]
					const allowed = new Set(attributeAllowlist[tag] ?? [])

					for (const key of Object.keys(attrs)) {
						const bareKey = key.startsWith(':') ? key.slice(1) : key

						if (bareKey === 'class') {
							if (!allowed.has('class')) {
								delete attrs[key]
								continue
							}
							const filtered = filterClassValue(attrs[key], tag)
							if (filtered) attrs[key] = filtered
							else delete attrs[key]
							continue
						}

						if (bareKey === 'style') {
							if (!allowed.has('style')) {
								delete attrs[key]
								continue
							}
							const filtered = filterStyleValue(attrs[key])
							if (filtered) attrs[key] = filtered
							else delete attrs[key]
							continue
						}

						if (!allowed.has(bareKey)) delete attrs[key]
					}
				},
			)
		},
	}))()
}
