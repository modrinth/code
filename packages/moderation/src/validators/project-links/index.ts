export interface ProjectContentLinkBlocklistEntry {
	label: string
	domains: readonly string[]
}

export const PROJECT_CONTENT_LINK_SHORTENERS = [
	'bit.ly',
	'adf.ly',
	'tinyurl.com',
	'short.io',
	'is.gd',
] as const

export const PROJECT_CONTENT_LINK_BLOCKLIST: readonly ProjectContentLinkBlocklistEntry[] = [
	{
		label: 'URL shortener',
		domains: PROJECT_CONTENT_LINK_SHORTENERS,
	},
	{ label: 'Twitter', domains: ['twitter.com', 'x.com'] },
	{ label: 'Instagram', domains: ['instagram.com'] },
	{ label: 'Facebook', domains: ['facebook.com'] },
	{ label: 'TikTok', domains: ['tiktok.com'] },
	{ label: 'Telegram', domains: ['telegram.org', 't.me'] },
	{ label: 'Bilibili', domains: ['bilibili.com'] },
	{ label: 'Bluesky', domains: ['bsky.app'] },
	{ label: 'Twitch', domains: ['twitch.tv'] },
	{ label: 'Reddit', domains: ['reddit.com', 'redd.it'] },
	{ label: 'Modrinth', domains: ['modrinth.com'] },
	{ label: 'Minecraft', domains: ['minecraft.net'] },
	{
		label: 'Mod distribution platform',
		domains: ['curseforge.com', 'planetminecraft.com', '9minecraft.net', 'mcmod.cn'],
	},
	{
		label: 'AI mod generation platform',
		domains: ['creativemode.net', 'orcaclient.com', 'autoforged.cn'],
	},
]

export interface BlockedProjectContentLink extends Record<string, unknown> {
	label: string
	url: string
}

function isIpAddress(hostname: string) {
	const strippedHostname = hostname.replace(/^\[|]$/g, '')
	return /^\d{1,3}(?:\.\d{1,3}){3}$/.test(strippedHostname) || strippedHostname.includes(':')
}

export function getBlockedProjectContentLink(url: string): BlockedProjectContentLink | null {
	let hostname: string
	try {
		hostname = new URL(url).hostname.toLowerCase().replace(/\.$/, '')
	} catch {
		return null
	}

	if (isIpAddress(hostname)) return { label: 'IP address', url }

	const entry = PROJECT_CONTENT_LINK_BLOCKLIST.find(({ domains }) =>
		domains.some((domain) => hostname === domain || hostname.endsWith(`.${domain}`)),
	)

	return entry ? { label: entry.label, url } : null
}
