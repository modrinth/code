import type { ElementNode } from 'comark'
import { defineComarkPlugin } from 'comark/parse'
import { visitAsync } from 'comark/utils'

// Imgur is blocked in UK and Indonesia
const imgurProxyCountries = new Set(['GB', 'ID'])

type MarkdownUserCountryResolver = () => string | null | undefined

let resolveMarkdownUserCountry: MarkdownUserCountryResolver = () => undefined

export const setMarkdownUserCountryResolver = (resolver: MarkdownUserCountryResolver) => {
	resolveMarkdownUserCountry = resolver
}

const shouldProxyImgur = () => {
	try {
		const country = resolveMarkdownUserCountry()
		return country ? imgurProxyCountries.has(country.toUpperCase()) : false
	} catch {
		return false
	}
}

const getImgurProxyUrl = (value: string) => {
	try {
		const url = new URL(value)
		if (url.hostname !== 'imgur.com' && !url.hostname.endsWith('.imgur.com')) {
			return undefined
		}

		return `https://external-content.duckduckgo.com/iu/?u=${encodeURIComponent(url.toString())}.png`
	} catch {
		return undefined
	}
}

const allowedMediaHostnames = [
	'imgur.com',
	'i.imgur.com',
	'cdn-raw.modrinth.com',
	'cdn.modrinth.com',
	'staging-cdn-raw.modrinth.com',
	'staging-cdn.modrinth.com',
	'github.com',
	'raw.githubusercontent.com',
	'user-images.githubusercontent.com',
	'img.shields.io',
	'i.postimg.cc',
	'wsrv.nl',
	'cf.way2muchnoise.eu',
	'bstats.org',
	'cdn.serilum.com',
	'workflow.serilum.com',
	'modfolio.creeperkatze.dev',
	'badges.crowdin.net',
]

const allowedMediaHostnameSuffixes = ['.github.io']

export function resolveMediaSrc(value: unknown): string | undefined {
	if (typeof value !== 'string' || !value || value.startsWith('data:')) {
		return typeof value === 'string' ? value : undefined
	}

	if (shouldProxyImgur()) {
		const proxied = getImgurProxyUrl(value)
		if (proxied) return proxied
	}

	try {
		const url = new URL(value)

		if (url.hostname.includes('wsrv.nl')) {
			url.searchParams.delete('errorredirect')
			url.searchParams.delete('default')
		}

		const allowed =
			allowedMediaHostnames.includes(url.hostname) ||
			allowedMediaHostnameSuffixes.some((suffix) => url.hostname.endsWith(suffix))

		if (!allowed) {
			return `https://wsrv.nl/?url=${encodeURIComponent(url.toString())}&n=-1`
		}

		return url.toString()
	} catch {
		return value
	}
}

const iframeAllowedSources: {
	url: RegExp
	allowedParameters: string[]
}[] = [
	{
		url: /^https?:\/\/(www\.)?youtube(-nocookie)?\.com\/embed\/[a-zA-Z0-9_-]{11}/,
		allowedParameters: ['start', 'end'],
	},
	{
		url: /^https?:\/\/(www\.)?discord\.com\/widget/,
		allowedParameters: ['id'],
	},
]

export function resolveIframeSrc(value: unknown): string | undefined {
	if (typeof value !== 'string' || !value) return undefined

	try {
		const url = new URL(value)

		for (const source of iframeAllowedSources) {
			if (!source.url.test(url.href)) continue

			const newParams = new URLSearchParams()
			url.searchParams.forEach((val, key) => {
				if (source.allowedParameters.includes(key)) newParams.set(key, val)
			})
			url.search = newParams.toString()
			return url.toString()
		}
	} catch {
		/* empty */
	}

	return undefined
}

export function resolveLinkRel(href: unknown): string | undefined {
	if (typeof href !== 'string' || !href) return 'noopener nofollow ugc'

	try {
		const url = new URL(href)
		if (url.hostname === 'modrinth.com') return undefined
	} catch {
		/* empty */
	}

	return 'noopener nofollow ugc'
}

export const resolveMedia = defineComarkPlugin(() => ({
	name: 'resolve-media',
	async post(state) {
		await visitAsync(
			state.tree,
			(node) => typeof node !== 'string' && node[0] !== null,
			(node) => {
				const element = node as ElementNode
				const attrs = element[1]

				switch (element[0].toLowerCase()) {
					case 'a':
						attrs.rel = resolveLinkRel(attrs.href)
						return
					case 'img':
					case 'audio':
						attrs.src = resolveMediaSrc(attrs.src)
						return
					case 'video':
						attrs.src = resolveMediaSrc(attrs.src)
						attrs.poster = resolveMediaSrc(attrs.poster)
						return
					case 'source':
						attrs.src = resolveMediaSrc(attrs.src)
						attrs.srcset = resolveMediaSrc(attrs.srcset)
						return
					case 'iframe': {
						const src = resolveIframeSrc(attrs.src)
						if (!src) return false
						attrs.src = src
						return
					}
				}
			},
		)
	},
}))
