export interface ProjectLinkBlocklistEntry {
	label: string
	domains: readonly string[]
}

export const PROJECT_LINK_SHORTENERS = [
	'bit.ly',
	'adf.ly',
	'tinyurl.com',
	'short.io',
	'is.gd',
] as const

const URL_SHORTENER_BLOCKLIST_ENTRY: ProjectLinkBlocklistEntry = {
	label: 'URL shortener',
	domains: PROJECT_LINK_SHORTENERS,
}

export const PROJECT_CONTENT_LINK_BLOCKLIST: readonly ProjectLinkBlocklistEntry[] = [
	URL_SHORTENER_BLOCKLIST_ENTRY,
]

export const PROJECT_EXTERNAL_LINK_BLOCKLIST: readonly ProjectLinkBlocklistEntry[] = [
	URL_SHORTENER_BLOCKLIST_ENTRY,
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

export interface BlockedProjectLink extends Record<string, unknown> {
	label: string
	url: string
}

function isIpAddress(hostname: string) {
	const strippedHostname = hostname.replace(/^\[|]$/g, '')
	return /^\d{1,3}(?:\.\d{1,3}){3}$/.test(strippedHostname) || strippedHostname.includes(':')
}

function getBlockedProjectLink(
	url: string,
	blocklist: readonly ProjectLinkBlocklistEntry[],
): BlockedProjectLink | null {
	let hostname: string
	try {
		hostname = new URL(url).hostname.toLowerCase().replace(/\.$/, '')
	} catch {
		return null
	}

	if (isIpAddress(hostname)) return { label: 'IP address', url }

	const entry = blocklist.find(({ domains }) =>
		domains.some((domain) => hostname === domain || hostname.endsWith(`.${domain}`)),
	)

	return entry ? { label: entry.label, url } : null
}

export function getBlockedProjectContentLink(url: string): BlockedProjectLink | null {
	return getBlockedProjectLink(url, PROJECT_CONTENT_LINK_BLOCKLIST)
}

export function getBlockedProjectExternalLink(url: string): BlockedProjectLink | null {
	return getBlockedProjectLink(url, PROJECT_EXTERNAL_LINK_BLOCKLIST)
}
