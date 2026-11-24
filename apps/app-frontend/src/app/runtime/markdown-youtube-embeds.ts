import { setMarkdownIframeSourceRewriter } from '@modrinth/utils'

import { useAppSettings } from '@/composables/use-app-settings.ts'
import { config } from '@/config.ts'

const LOCAL_SITE_URL = 'http://localhost:3000'
const YOUTUBE_HOSTS = new Set([
	'youtube.com',
	'www.youtube.com',
	'youtube-nocookie.com',
	'www.youtube-nocookie.com',
])
const YOUTUBE_EMBED_PATH = /^\/embed\/([a-zA-Z0-9_-]{11})\/?$/

export function setupMarkdownYoutubeEmbeds() {
	const appSettings = useAppSettings()

	setMarkdownIframeSourceRewriter((url) => {
		if (!YOUTUBE_HOSTS.has(url.hostname)) return null

		const videoId = url.pathname.match(YOUTUBE_EMBED_PATH)?.[1]
		if (!videoId) return null

		const siteUrl = appSettings.getFeatureFlag('use_local_youtube_embeds')
			? LOCAL_SITE_URL
			: config.siteUrl

		return `${siteUrl}/embed/youtube/${videoId}${url.search}`
	})
}
