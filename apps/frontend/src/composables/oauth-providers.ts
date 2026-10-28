import {
	DiscordColorIcon,
	GitHubColorIcon,
	GitLabColorIcon,
	GoogleColorIcon,
	MicrosoftColorIcon,
	SteamColorIcon,
} from '@modrinth/assets'

export const oauthProviders = [
	{ id: 'discord', name: 'Discord', icon: DiscordColorIcon },
	{ id: 'github', name: 'GitHub', icon: GitHubColorIcon },
	{ id: 'microsoft', name: 'Microsoft', icon: MicrosoftColorIcon },
	{ id: 'google', name: 'Google', icon: GoogleColorIcon },
	{ id: 'steam', name: 'Steam', icon: SteamColorIcon },
	{ id: 'gitlab', name: 'GitLab', icon: GitLabColorIcon },
] as const

export type OAuthProviderId = (typeof oauthProviders)[number]['id']
