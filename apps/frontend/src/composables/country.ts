import type { ISO3166 } from '@modrinth/api-client'

import { useRequestHeaders, useState } from '#imports'
import { countries, subdivisions } from '~/generated/state.json'

export const useCountries = () => {
	return computed(() => (countries ?? []) as ISO3166.Country[])
}

export const useFormattedCountries = () => {
	const countries = useCountries()

	return computed(() =>
		countries.value.map((country) => {
			let label = country.nameShort

			if (country.alpha2 === 'TW') {
				label = 'Taiwan'
			} else if (country.nameShort.length > 30) {
				label = `${country.nameShort} (${country.alpha2})`
			}

			return {
				value: country.alpha2,
				label,
			}
		}),
	)
}

export const useSubdivisions = (countryCode: ComputedRef<string> | Ref<string> | string) => {
	const code = isRef(countryCode) ? countryCode : ref(countryCode)
	const byCountry = (subdivisions ?? {}) as Record<string, ISO3166.Subdivision[]>

	return computed(() => byCountry[unref(code)] ?? [])
}

export const useUserCountry = () => {
	const country = useState<string>('userCountry', () => 'US')
	const fromServer = useState<boolean>('userCountryFromServer', () => false)

	if (import.meta.server) {
		const headers = useRequestHeaders(['cf-ipcountry', 'accept-language'])
		const cf = headers['cf-ipcountry']
		if (cf) {
			country.value = cf.toUpperCase()
			fromServer.value = true
		} else {
			const al = headers['accept-language'] || ''
			const tag = al.split(',')[0]
			const val = tag.split('-')[1]?.toLowerCase()
			if (val) {
				country.value = val
				fromServer.value = true
			}
		}
	}

	if (import.meta.client) {
		onMounted(() => {
			if (fromServer.value) return
			// @ts-expect-error - ignore TS not knowing about navigator.userLanguage
			const lang = navigator.language || navigator.userLanguage || ''
			const region = lang.split('-')[1]
			if (region) {
				country.value = region.toUpperCase()
			}
		})
	}

	return country
}
