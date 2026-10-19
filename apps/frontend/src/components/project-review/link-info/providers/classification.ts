import type { LinkMatch, Provider } from '../types'
import { forges } from './forges'

export const classification: Provider = (url) => {
	const host = url.hostname.replace(/^www\./, '')
	const [path, name] = url.pathname.split('/').filter(Boolean)
	const donations: Record<string, string> = {
		'ko-fi.com': 'Ko-fi',
		'patreon.com': 'Patreon',
		'buymeacoffee.com': 'Buy Me a Coffee',
		'paypal.me': 'PayPal',
		'paypal.com': 'PayPal',
		'opencollective.com': 'Open Collective',
	}
	if (donations[host])
		return {
			platform: donations[host],
			kind: 'donation',
			name: host === 'paypal.com' ? undefined : path,
		}
	if (host === 'docs.google.com' || host === 'forms.gle')
		return { platform: 'Google', kind: 'document' }
	if (host === 'sr.ht' || host.endsWith('.sr.ht'))
		return { platform: 'SourceHut', kind: name ? 'repository' : 'user', name: path }
	if (host === 'bilibili.com' || host === 'space.bilibili.com')
		return { platform: 'Bilibili', kind: path === 'video' ? 'video' : 'user', name }
	if (host === 'curseforge.com') return { platform: 'CurseForge', kind: 'project' }
	if (host === 'planetminecraft.com')
		return { platform: 'Planet Minecraft', kind: path === 'member' ? 'user' : 'project', name }
	if (host === 't.me' || host === 'telegram.me')
		return {
			platform: 'Telegram',
			kind: path === 'joinchat' || path?.startsWith('+') ? 'invite' : 'channel',
			name: path,
		}
	if (host === 'twitter.com' || host === 'x.com') return { platform: 'X', kind: 'user', name: path }
	if (host === 'trello.com') return { platform: 'Trello', kind: 'board' }
	if (host.endsWith('.fandom.com') || host.endsWith('.wikia.com'))
		return { platform: 'Fandom', kind: 'wiki' }
	if (host === 'mcmod.cn') return { platform: 'MC百科', kind: 'wiki' }
	if (host === 'moddedmc.wiki') return { platform: 'ModdedMC Wiki', kind: 'wiki' }
}

export function fallback(url: URL): LinkMatch {
	return {
		platform: url.hostname,
		kind: 'website',
		load: async (request, client) => {
			const html = await request<string>(url.href)
			if (typeof html !== 'string') return { profiles: [], warnings: [] }
			const document = new DOMParser().parseFromString(html, 'text/html')
			const generator =
				document.querySelector('meta[name="generator"]')?.getAttribute('content') ?? ''
			const forge = /gitlab/i.test(generator)
				? 'GitLab'
				: /gitea|forgejo/i.test(generator)
					? 'Git forge'
					: undefined
			const match = forge ? forges(url, forge) : undefined
			if (match?.load) return match.load(request, client)
			const resolve = (value: string | null) => {
				if (!value) return undefined
				try {
					const link = new URL(value, url)
					return ['https:', 'http:'].includes(link.protocol) ? link.href : undefined
				} catch {
					return undefined
				}
			}
			const links = new Map<string, string>()
			for (const anchor of document.querySelectorAll('a[href]')) {
				const href = resolve(anchor.getAttribute('href'))
				if (!href || new URL(href).origin === url.origin || links.has(href)) continue
				links.set(href, anchor.textContent?.trim() || href)
				if (links.size === 15) break
			}
			return {
				profiles: [
					{
						title: document.title || url.hostname,
						avatar: resolve(
							document.querySelector('link[rel~="icon"]')?.getAttribute('href') ?? '/favicon.ico',
						),
						details: [],
					},
				],
				warnings: [],
				links: [...links].map(([url, title]) => ({ url, title })),
			}
		},
	}
}
