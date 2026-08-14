import { provideUserCountry } from '@modrinth/ui'
import { onMounted } from 'vue'

import { useRequestHeaders, useState } from '#imports'

function getCountryFromLocale(locale: string | undefined) {
	if (!locale) return undefined

	try {
		return new Intl.Locale(locale).region?.toUpperCase()
	} catch {
		return undefined
	}
}

export function setupUserCountryProvider() {
	const country = useState<string>('userCountry', () => 'US')
	const fromServer = useState<boolean>('userCountryFromServer', () => false)

	if (import.meta.server) {
		const headers = useRequestHeaders(['cf-ipcountry', 'accept-language'])
		const cloudflareCountry = headers['cf-ipcountry']
		const locale = headers['accept-language']?.split(',')[0]?.split(';')[0]?.trim()
		const detectedCountry = cloudflareCountry?.toUpperCase() ?? getCountryFromLocale(locale)

		if (detectedCountry) {
			country.value = detectedCountry
			fromServer.value = true
		}
	}

	if (import.meta.client) {
		onMounted(() => {
			if (!fromServer.value) {
				country.value = getCountryFromLocale(navigator.language) ?? country.value
			}
		})
	}

	return provideUserCountry(country)
}
