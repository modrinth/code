import { notifyCopied, useModerationKeybinds } from '~/composables/moderation.ts'

function getOfficialOrigin(apiBaseUrl: string): string {
	if (apiBaseUrl.startsWith('https://staging-api.modrinth.com')) {
		return 'https://staging.modrinth.com'
	}
	return 'https://modrinth.com'
}

export default defineNuxtPlugin(() => {
	const config = useRuntimeConfig()
	const keybinds = useModerationKeybinds()

	window.addEventListener('keydown', (event) => {
		if (event.repeat) {
			return
		}

		keybinds.value.handle(event, {
			scope: 'global',
			officialUrl: `${getOfficialOrigin(String(config.public.apiBaseUrl))}${window.location.pathname}${window.location.search}`,
			localhostUrl: `http://localhost:3000${window.location.pathname}${window.location.search}`,
			notifyCopied,
		})
	})
})
