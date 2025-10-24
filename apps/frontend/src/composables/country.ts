import { all } from 'iso-3166-1'

import { useRequestHeaders, useState } from '#imports'

const shortCountryNames: Record<string, string> = {
	GB: 'United Kingdom',
	US: 'United States',
	SH: 'Saint Helena',
	GS: 'South Georgia',
	KP: "Dem. People's Rep. of Korea",
	UM: 'US Minor Outlying Islands',
	VI: 'US Virgin Islands',
	VE: 'Venezuela',
	HM: 'Heard & McDonald Islands',
	BQ: 'Bonaire, Sint Eustatius & Saba',
	LA: "Lao People's Dem. Rep.",
	VC: 'St. Vincent & Grenadines',
}

export const useFormattedCountries = () => {
	return computed(() =>
		all().map((country) => {
			let label = country.country

			if (shortCountryNames[country.alpha2]) {
				label = `${shortCountryNames[country.alpha2]} (${country.alpha2})`
			} else if (country.alpha2 === 'TW') {
				label = 'Taiwan'
			} else if (country.country.length > 30) {
				label = `${country.country} (${country.alpha2})`
			}

			return {
				value: country.alpha2,
				label,
			}
		}),
	)
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
