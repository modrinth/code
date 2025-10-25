<script setup lang="ts">
import {
	AppleIcon,
	BlueskyIcon,
	BuyMeACoffeeIcon,
	CurseForgeIcon,
	DiscordIcon,
	FacebookIcon,
	GithubIcon,
	InstagramIcon,
	KoFiIcon,
	MastodonIcon,
	ModrinthIcon,
	OpenCollectiveIcon,
	PatreonIcon,
	PayPalIcon,
	RedditIcon,
	ReelsIcon,
	SnapchatIcon,
	ThreadsIcon,
	TikTokIcon,
	TumblrIcon,
	TwitchIcon,
	TwitterIcon,
	WindowsIcon,
	YouTubeGaming,
	YouTubeIcon,
	YouTubeShortsIcon,
} from '@modrinth/assets'
import { computed } from 'vue'

const props = defineProps<{
	keyword: string
}>()

const services = [
	{
		icon: AppleIcon,
		keywords: ['apple'],
	},
	{
		icon: BlueskyIcon,
		keywords: ['bluesky', 'bsky', 'blue sky'],
	},
	{
		icon: BuyMeACoffeeIcon,
		keywords: ['buymeacoffee', 'bmac', 'buy me a coffee'],
	},
	{
		icon: DiscordIcon,
		keywords: ['discord'],
	},
	{
		icon: FacebookIcon,
		keywords: ['facebook', 'fb', 'face book'],
	},
	{
		icon: GithubIcon,
		keywords: ['github', 'gh', 'git hub'],
	},
	{
		icon: ThreadsIcon,
		keywords: ['threads'],
	},
	{
		icon: InstagramIcon,
		keywords: ['instagram', 'ig', 'insta'],
	},
	{
		icon: KoFiIcon,
		keywords: ['ko-fi', 'kofi', 'ko fi'],
	},
	{
		icon: MastodonIcon,
		keywords: ['mastodon'],
	},
	{
		icon: OpenCollectiveIcon,
		keywords: ['opencollective', 'open collective'],
	},
	{
		icon: PatreonIcon,
		keywords: ['patreon'],
	},
	{
		icon: PayPalIcon,
		keywords: ['paypal', 'pay pal'],
	},
	{
		icon: RedditIcon,
		keywords: ['reddit'],
	},
	{
		icon: ReelsIcon,
		keywords: ['reels', 'instagram reels', 'facebook reels'],
	},
	{
		icon: SnapchatIcon,
		keywords: ['snapchat'],
	},
	{
		icon: TikTokIcon,
		keywords: ['tiktok', 'tik', 'tok'],
	},
	{
		icon: TumblrIcon,
		keywords: ['tumblr'],
	},
	{
		icon: TwitchIcon,
		keywords: ['twitch', 'twitch.tv'],
	},
	{
		icon: WindowsIcon,
		keywords: ['windows', 'microsoft'],
	},
	{
		icon: YouTubeIcon,
		keywords: ['youtube', 'yt'],
	},
	{
		icon: YouTubeShortsIcon,
		keywords: ['shorts', 'youtube shorts'],
	},
	{
		icon: YouTubeGaming,
		keywords: ['youtube gaming'],
	},
	{
		icon: CurseForgeIcon,
		keywords: ['curseforge', 'cf', 'curse', 'curse forge'],
	},
	{
		icon: ModrinthIcon,
		keywords: ['modrinth', 'mod rinth', 'modrith', 'mr'],
	},
	{
		icon: TwitterIcon,
		keywords: ['twitter', 'x.com', 'x'],
	},
]

const selectedService = computed(() =>
	services.find((service) =>
		service.keywords.some((keyword) => props.keyword.toLowerCase().includes(keyword)),
	),
)
</script>

<template>
	<component :is="selectedService?.icon" v-if="selectedService" />
	<slot v-else />
</template>
