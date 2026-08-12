import type { AbstractModrinthClient } from '@modrinth/api-client'
import { provideUserCountry } from '@modrinth/ui'
import { ref } from 'vue'

export function setupUserCountryProvider(client: AbstractModrinthClient) {
	const country = ref('US')

	void client.labrinth.geoip
		.getCountry()
		.then((detectedCountry) => {
			country.value = detectedCountry ?? country.value
		})
		.catch(() => {})

	return provideUserCountry(country)
}
