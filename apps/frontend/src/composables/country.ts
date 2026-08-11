import type { ISO3166 } from '@modrinth/api-client'
import { useUserCountry as useInjectedUserCountry } from '@modrinth/ui'

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

export const useUserCountry = useInjectedUserCountry
