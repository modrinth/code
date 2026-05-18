import type { ComarkNode } from 'comark'
import { defineComarkPlugin } from 'comark/parse'

type AttributeMap = Record<string, unknown>
type MutableComarkElement = [string | null, AttributeMap, ...ComarkNode[]]

export interface ModrinthMarkdownSecurityOptions {
	baseUrl?: string
	stripBaseUrl?: string
}

const ALLOWED_TAGS = new Set([
	'a',
	'abbr',
	'address',
	'area',
	'article',
	'aside',
	'audio',
	'b',
	'bdi',
	'bdo',
	'big',
	'blockquote',
	'br',
	'caption',
	'center',
	'cite',
	'code',
	'col',
	'colgroup',
	'dd',
	'del',
	'details',
	'div',
	'dl',
	'dt',
	'em',
	'figcaption',
	'figure',
	'font',
	'footer',
	'h1',
	'h2',
	'h3',
	'h4',
	'h5',
	'h6',
	'header',
	'hr',
	'i',
	'iframe',
	'img',
	'input',
	'ins',
	'kbd',
	'li',
	'map',
	'mark',
	'nav',
	'ol',
	'p',
	'picture',
	'pre',
	's',
	'section',
	'small',
	'source',
	'span',
	'strike',
	'strong',
	'sub',
	'summary',
	'sup',
	'table',
	'tbody',
	'td',
	'tfoot',
	'th',
	'thead',
	'tr',
	'tt',
	'u',
	'ul',
	'video',
])

const ALLOWED_ATTRIBUTES: Record<string, Set<string>> = {
	a: new Set(['href', 'rel', 'target', 'title']),
	abbr: new Set(['title']),
	area: new Set(['alt', 'coords', 'href', 'shape']),
	audio: new Set(['autoplay', 'controls', 'crossorigin', 'loop', 'muted', 'preload', 'src']),
	blockquote: new Set(['cite']),
	col: new Set(['align', 'span', 'valign', 'width']),
	colgroup: new Set(['align', 'span', 'valign', 'width']),
	del: new Set(['datetime']),
	details: new Set(['open']),
	font: new Set(['color', 'face', 'size']),
	h1: new Set(['id']),
	h2: new Set(['id']),
	h3: new Set(['id']),
	h4: new Set(['id']),
	h5: new Set(['id']),
	h6: new Set(['id']),
	iframe: new Set(['allowfullscreen', 'frameborder', 'height', 'src', 'width', 'start', 'end']),
	img: new Set(['align', 'alt', 'height', 'loading', 'src', 'style', 'title', 'usemap', 'width']),
	input: new Set(['checked', 'disabled', 'type']),
	ins: new Set(['datetime']),
	kbd: new Set(['id']),
	map: new Set(['name']),
	p: new Set(['align']),
	div: new Set(['align']),
	pre: new Set(['class', 'style', 'tabindex']),
	code: new Set(['class']),
	span: new Set(['class', 'style']),
	source: new Set(['media', 'sizes', 'src', 'srcset', 'type']),
	table: new Set(['align', 'border', 'valign', 'width']),
	tbody: new Set(['align', 'valign']),
	td: new Set(['align', 'colspan', 'rowspan', 'style', 'valign', 'width']),
	tfoot: new Set(['align', 'valign']),
	th: new Set(['align', 'colspan', 'rowspan', 'style', 'valign', 'width']),
	thead: new Set(['align', 'valign']),
	tr: new Set(['align', 'rowspan', 'valign']),
	video: new Set([
		'autoplay',
		'controls',
		'crossorigin',
		'height',
		'loop',
		'muted',
		'playsinline',
		'poster',
		'preload',
		'src',
		'width',
	]),
}

const MEDIA_URL_TAGS = new Set(['audio', 'img', 'source', 'video'])
const MEDIA_URL_ATTRIBUTES = new Set(['poster', 'src', 'srcset'])

const ALLOWED_MEDIA_HOSTNAMES = new Set([
	'imgur.com',
	'i.imgur.com',
	'cdn-raw.modrinth.com',
	'cdn.modrinth.com',
	'staging-cdn-raw.modrinth.com',
	'staging-cdn.modrinth.com',
	'github.com',
	'raw.githubusercontent.com',
	'img.shields.io',
	'i.postimg.cc',
	'wsrv.nl',
	'cf.way2muchnoise.eu',
	'bstats.org',
])

const ALLOWED_MEDIA_HOSTNAME_SUFFIXES = ['.github.io']

const ALLOWED_IFRAME_SOURCES = [
	{
		url: /^https?:\/\/(www\.)?youtube(-nocookie)?\.com\/embed\/[a-zA-Z0-9_-]{11}/,
		allowedParameters: [/start=\d+/, /end=\d+/],
	},
	{
		url: /^https?:\/\/(www\.)?discord\.com\/widget/,
		allowedParameters: [/id=\d{18,19}/],
	},
]

const BARE_URL_PATTERN = /https?:\/\/[^\s<]+/g

function isElement(node: ComarkNode): node is MutableComarkElement {
	return Array.isArray(node)
}

function normalizeAttributeName(name: string): string {
	return name.startsWith(':') ? name.slice(1) : name.toLowerCase()
}

function stringifyAttribute(value: unknown): string | undefined {
	if (typeof value === 'string') return value
	if (typeof value === 'number' || typeof value === 'boolean') return String(value)
	return undefined
}

function normalizeUrl(value: string, options: ModrinthMarkdownSecurityOptions): string {
	if (!options.baseUrl) return value

	try {
		const url = new URL(value, options.baseUrl)
		const href = url.href
		return options.stripBaseUrl && href.startsWith(options.stripBaseUrl)
			? href.slice(options.stripBaseUrl.length)
			: href
	} catch {
		return value
	}
}

function hasSafeUrlProtocol(value: string, allowDataImage = false): boolean {
	const trimmed = value.trim()
	if (!trimmed) return false
	if (trimmed === '#') return true
	if (
		trimmed.startsWith('/') ||
		trimmed.startsWith('./') ||
		trimmed.startsWith('../') ||
		trimmed.startsWith('#')
	) {
		return true
	}

	try {
		const url = new URL(trimmed)
		if (['http:', 'https:', 'mailto:', 'tel:', 'ftp:'].includes(url.protocol)) return true
		return allowDataImage && url.protocol === 'data:' && trimmed.startsWith('data:image/')
	} catch {
		return false
	}
}

function isAllowedMediaHost(hostname: string): boolean {
	return (
		ALLOWED_MEDIA_HOSTNAMES.has(hostname) ||
		ALLOWED_MEDIA_HOSTNAME_SUFFIXES.some((suffix) => hostname.endsWith(suffix))
	)
}

function sanitizeMediaUrl(value: string, attrName: string): string | undefined {
	const allowDataImage = attrName !== 'srcset'
	if (!hasSafeUrlProtocol(value, allowDataImage)) return undefined
	if (value.trim().startsWith('data:')) return value

	try {
		const url = new URL(value)

		if (url.hostname.includes('wsrv.nl')) {
			url.searchParams.delete('errorredirect')
			url.searchParams.delete('default')
		}

		if (!isAllowedMediaHost(url.hostname)) {
			return `https://wsrv.nl/?url=${encodeURIComponent(url.toString().replaceAll('&amp;', '&'))}&n=-1`
		}

		return url.toString()
	} catch {
		return value
	}
}

function sanitizeSrcset(value: string): string | undefined {
	const srcset = value
		.split(',')
		.map((entry) => {
			const parts = entry.trim().split(/\s+/)
			const url = parts.shift()
			if (!url) return undefined

			const sanitized = sanitizeMediaUrl(url, 'srcset')
			return sanitized ? [sanitized, ...parts].join(' ') : undefined
		})
		.filter((entry): entry is string => !!entry)
		.join(', ')

	return srcset || undefined
}

function sanitizeIframeSrc(value: string): string | undefined {
	try {
		const url = new URL(value)

		for (const source of ALLOWED_IFRAME_SOURCES) {
			if (!source.url.test(url.href)) continue

			const searchParams = new URLSearchParams(url.searchParams)
			url.searchParams.forEach((paramValue, key) => {
				if (!source.allowedParameters.some((param) => param.test(`${key}=${paramValue}`))) {
					searchParams.delete(key)
				}
			})

			url.search = searchParams.toString()
			return url.toString()
		}
	} catch {
		return undefined
	}

	return undefined
}

function isSafeStyleValue(value: string): boolean {
	return !/[<>]/.test(value) && !/url\s*\(|expression\s*\(/i.test(value)
}

function sanitizeStyle(value: string, tag: string): string | undefined {
	const declarations = value
		.split(';')
		.map((declaration) => {
			const [propertyRaw, ...valueParts] = declaration.split(':')
			const property = propertyRaw?.trim().toLowerCase()
			const propertyValue = valueParts.join(':').trim()

			if (!property || !propertyValue) return undefined
			if (!isSafeStyleValue(propertyValue)) return undefined

			if (property === 'image-rendering' && propertyValue === 'pixelated') {
				return `${property}: ${propertyValue}`
			}

			if (property === 'text-align' && /^(center|left|right)$/.test(propertyValue)) {
				return `${property}: ${propertyValue}`
			}

			if (property === 'float' && /^(left|right)$/.test(propertyValue)) {
				return `${property}: ${propertyValue}`
			}

			if (tag === 'pre' && property.startsWith('--shiki-')) {
				return `${property}: ${propertyValue}`
			}

			if (tag === 'span') {
				if (
					['color', 'background-color'].includes(property) &&
					/^(#[\da-f]{3,8}|rgb\([\d\s,.%]+\)|rgba\([\d\s,.%]+\)|hsl\([\d\s,.%]+\)|hsla\([\d\s,.%]+\)|var\(--shiki-[\w-]+\)|currentcolor|transparent|inherit)$/i.test(
						propertyValue,
					)
				) {
					return `${property}: ${propertyValue}`
				}

				if (property === 'font-style' && /^(normal|italic)$/.test(propertyValue)) {
					return `${property}: ${propertyValue}`
				}

				if (property === 'font-weight' && /^(normal|bold|[1-9]00)$/.test(propertyValue)) {
					return `${property}: ${propertyValue}`
				}

				if (property === 'text-decoration' && /^(none|underline|line-through)$/.test(propertyValue)) {
					return `${property}: ${propertyValue}`
				}

				if (property === 'display' && /^(inline|inline-block|block)$/.test(propertyValue)) {
					return `${property}: ${propertyValue}`
				}
			}

			return undefined
		})
		.filter((declaration): declaration is string => !!declaration)

	return declarations.length > 0 ? declarations.join('; ') : undefined
}

function sanitizeClass(tag: string, value: string): string | undefined {
	const classes = value
		.split(/\s+/)
		.filter(
			(className) =>
				className.startsWith('language-') ||
				(tag === 'pre' &&
					/^(shiki|shiki-themes|material-theme-lighter|material-theme-palenight)$/.test(className)) ||
				(tag === 'span' && ['line', 'highlight'].includes(className)),
		)

	return classes.length > 0 ? classes.join(' ') : undefined
}

function sanitizeHref(value: string, options: ModrinthMarkdownSecurityOptions): string | undefined {
	const normalized = normalizeUrl(value, options)
	return hasSafeUrlProtocol(normalized) ? normalized : undefined
}

function isExactModrinthLink(href: string): boolean {
	try {
		return new URL(href).hostname === 'modrinth.com'
	} catch {
		return false
	}
}

function sanitizeAttribute(
	tag: string,
	rawName: string,
	rawValue: unknown,
	options: ModrinthMarkdownSecurityOptions,
): [string, unknown] | undefined {
	const name = normalizeAttributeName(rawName)
	if (name.startsWith('on')) return undefined

	const allowedAttributes = ALLOWED_ATTRIBUTES[tag]
	if (!allowedAttributes?.has(name)) return undefined

	const value = stringifyAttribute(rawValue)
	if (value === undefined) return undefined

	if (tag === 'iframe' && name === 'src') {
		const src = sanitizeIframeSrc(value)
		return src ? [name, src] : undefined
	}

	if (tag === 'a' || tag === 'area') {
		if (name === 'href') {
			const href = sanitizeHref(value, options)
			return href ? [name, href] : undefined
		}
		if (name === 'target' && !['_blank', '_self', '_parent', '_top'].includes(value)) {
			return undefined
		}
	}

	if (MEDIA_URL_TAGS.has(tag) && MEDIA_URL_ATTRIBUTES.has(name)) {
		if (name === 'srcset') {
			const srcset = sanitizeSrcset(normalizeUrl(value, options))
			return srcset ? [name, srcset] : undefined
		}

		const normalized = normalizeUrl(value, options)
		const sanitized = sanitizeMediaUrl(normalized, name)
		return sanitized ? [name, sanitized] : undefined
	}

	if (name === 'style') {
		const style = sanitizeStyle(value, tag)
		return style ? [name, style] : undefined
	}

	if (name === 'class') {
		const className = sanitizeClass(tag, value)
		return className ? [name, className] : undefined
	}

	if (name === 'tabindex' && !/^-?\d+$/.test(value)) return undefined

	if (tag === 'input' && name === 'type' && value !== 'checkbox') return undefined

	return [name, value]
}

function sanitizeElement(
	element: MutableComarkElement,
	options: ModrinthMarkdownSecurityOptions,
): false | void {
	const tag = element[0]
	if (typeof tag !== 'string' || !ALLOWED_TAGS.has(tag)) return false

	const attrs = element[1] as AttributeMap
	const sanitizedAttrs: AttributeMap = {}

	for (const [rawName, rawValue] of Object.entries(attrs)) {
		const sanitized = sanitizeAttribute(tag, rawName, rawValue, options)
		if (!sanitized) continue

		const [name, value] = sanitized
		sanitizedAttrs[name] = value
	}

	if (tag === 'a') {
		const href = stringifyAttribute(sanitizedAttrs.href)
		if (!href) return false
		if (!isExactModrinthLink(href)) {
			sanitizedAttrs.rel = 'noopener nofollow ugc'
		}
	}

	if (tag === 'area' && !sanitizedAttrs.href) return false
	if (tag === 'iframe' && !sanitizedAttrs.src) return false

	element[1] = sanitizedAttrs
}

function splitTextByBareUrls(text: string): ComarkNode[] {
	const nodes: ComarkNode[] = []
	let lastIndex = 0

	for (const match of text.matchAll(BARE_URL_PATTERN)) {
		const url = match[0].replace(/[),.;:!?]+$/, '')
		const index = match.index ?? 0

		if (index > lastIndex) nodes.push(text.slice(lastIndex, index))
		nodes.push(['a', { href: url }, url])
		lastIndex = index + url.length
	}

	if (lastIndex === 0) return [text]
	if (lastIndex < text.length) nodes.push(text.slice(lastIndex))
	return nodes
}

function linkifyTextNodes(nodes: ComarkNode[], parentTag?: string): void {
	const shouldSkip = parentTag === 'a' || parentTag === 'code' || parentTag === 'pre'
	if (shouldSkip) return

	for (let i = 0; i < nodes.length; i++) {
		const node = nodes[i]

		if (typeof node === 'string' && !shouldSkip) {
			const replacements = splitTextByBareUrls(node)
			if (replacements.length > 1) {
				nodes.splice(i, 1, ...replacements)
				i += replacements.length - 1
			}
			continue
		}

		if (isElement(node)) {
			const children = node.slice(2)
			linkifyTextNodes(children, node[0] as string | undefined)
			node.splice(2, node.length - 2, ...children)
		}
	}
}

function sanitizeNodes(nodes: ComarkNode[], options: ModrinthMarkdownSecurityOptions): void {
	for (let i = nodes.length - 1; i >= 0; i--) {
		const node = nodes[i]
		if (!isElement(node)) continue

		const result = sanitizeElement(node, options)
		if (result === false) {
			nodes.splice(i, 1)
			continue
		}

		const children = node.slice(2)
		sanitizeNodes(children, options)
		node.splice(2, node.length - 2, ...children)
	}
}

export const modrinthMarkdownSecurity = defineComarkPlugin(
	(options: ModrinthMarkdownSecurityOptions = {}) => ({
		name: 'modrinth-markdown-security',
		post(state) {
			linkifyTextNodes(state.tree.nodes)
			sanitizeNodes(state.tree.nodes, options)
		},
	}),
)
