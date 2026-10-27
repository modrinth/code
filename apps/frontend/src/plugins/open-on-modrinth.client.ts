import { moderationSettings } from '@modrinth/moderation'
import { isStaff } from '@modrinth/utils'

import {
	notifyCopied,
	useModerationKeybinds,
	useModerationSettings,
} from '~/composables/moderation.ts'

function getOfficialOrigin(apiBaseUrl: string): string {
	if (apiBaseUrl.startsWith('https://staging-api.modrinth.com')) {
		return 'https://staging.modrinth.com'
	}
	return 'https://modrinth.com'
}

function getAlternativeOrigin(hostname: string | null | undefined): string | null {
	const trimmed = hostname?.trim()
	if (!trimmed) {
		return null
	}

	try {
		const url = trimmed.includes('://') ? new URL(trimmed) : new URL(`http://${trimmed}`)
		if (url.protocol !== 'http:' && url.protocol !== 'https:') {
			return null
		}
		return url.origin
	} catch {
		return null
	}
}

export default defineNuxtPlugin(async () => {
	const config = useRuntimeConfig()
	const keybinds = useModerationKeybinds()
	const settings = useModerationSettings()
	const auth = await useAuth()

	window.addEventListener('keydown', (event) => {
		if (event.repeat || !isStaff(auth.value.user)) {
			return
		}

		const pathAndSearch = `${window.location.pathname}${window.location.search}`
		const alternativeOrigin = getAlternativeOrigin(
			settings.value.get(moderationSettings.General.AlternativeHostname),
		)

		keybinds.value.handle(event, {
			scope: 'global',
			officialUrl: `${getOfficialOrigin(String(config.public.apiBaseUrl))}${pathAndSearch}`,
			alternativeUrl: alternativeOrigin ? `${alternativeOrigin}${pathAndSearch}` : '',
			notifyCopied,
		})
	})
})
