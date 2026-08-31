export const URL_SHORTENERS = ['bit.ly', 'adf.ly', 'tinyurl.com', 'short.io', 'is.gd'] as const

export const EXTERNAL_LINKS_BLOCK_LIST = [
	{ label: 'Twitter', domains: ['twitter.com', 'x.com'] },
	{ label: 'Instagram', domains: ['instagram.com'] },
	{ label: 'Facebook', domains: ['facebook.com'] },
	{ label: 'TikTok', domains: ['tiktok.com'] },
	{ label: 'Telegram', domains: ['telegram.org', 't.me'] },
	{ label: 'Bilibili', domains: ['bilibili.com'] },
	{ label: 'Bluesky', domains: ['bsky.app'] },
	{ label: 'Twitch', domains: ['twitch.tv'] },
	{ label: 'YouTube', domains: ['youtube.com', 'youtu.be'] },
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
] as const
