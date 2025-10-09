<template>
	<div class="flex flex-col gap-4">
		<div class="flex flex-col gap-2.5">
			<div class="flex flex-row gap-1 align-middle">
				<span class="text-contrast font-semibold align-middle">Region</span>
				<UnknownIcon v-tooltip="'Some payout methods are not available in certain regions.'"
					class="size-4 mt-auto text-secondary" />
			</div>
			<Combobox :model-value="selectedCountryCode" :options="countries" placeholder="Select your country" searchable
				search-placeholder="Search countries..." :max-height="240" force-direction="up" @update:model-value="handleCountryChange" />
		</div>
	</div>
</template>

<script setup lang="ts">
import { UnknownIcon } from '@modrinth/assets';

import { useUserCountry } from '@/composables/country.ts';
import { useWithdrawContext } from '@/providers/creator-withdraw.ts';
import { Combobox } from '@modrinth/ui';
import { useGeolocation } from '@vueuse/core';
import { all } from 'iso-3166-1';

const withdrawContext = useWithdrawContext()
const userCountry = useUserCountry()
const { coords } = useGeolocation()

const countries = computed(() =>
	all().map((x) => ({
		value: x.alpha2,
		label: x.alpha2 === 'TW' ? 'Taiwan' : x.country,
	})),
)

const selectedCountryCode = computed(() => withdrawContext.withdrawData.value.selectedCountry?.id)

function handleCountryChange(countryCode: string | null) {
	console.log('handleCountryChange called with:', countryCode)
	if (countryCode) {
		const normalizedCode = countryCode.toUpperCase()
		const country = all().find((c) => c.alpha2 === normalizedCode)
		console.log('Found country:', country)
		if (country) {
			withdrawContext.withdrawData.value.selectedCountry = {
				id: country.alpha2,
				name: country.alpha2 === 'TW' ? 'Taiwan' : country.country,
			}
			console.log('Set selectedCountry to:', withdrawContext.withdrawData.value.selectedCountry)
		}
	} else {
		withdrawContext.withdrawData.value.selectedCountry = null
	}
}

console.log('Setup: userCountry.value =', userCountry.value)
console.log('Setup: current selectedCountry =', withdrawContext.withdrawData.value.selectedCountry)

if (!withdrawContext.withdrawData.value.selectedCountry) {
	const defaultCountryCode = userCountry.value || 'US'
	console.log('Setup: calling handleCountryChange with', defaultCountryCode)
	handleCountryChange(defaultCountryCode)
	console.log('Setup: selectedCountryCode computed =', selectedCountryCode.value)
}

async function getCountryFromGeoIP(lat: number, lon: number): Promise<string | null> {
	try {
		const response = await fetch(`https://api.bigdatacloud.net/data/reverse-geocode-client?latitude=${lat}&longitude=${lon}&localityLanguage=en`)
		const data = await response.json()
		return data.countryCode || null
	} catch {
		return null
	}
}

onMounted(async () => {
	if (withdrawContext.withdrawData.value.selectedCountry?.id === 'US' && !userCountry.value) {
		if (coords.value.latitude && coords.value.longitude) {
			const geoCountry = await getCountryFromGeoIP(coords.value.latitude, coords.value.longitude)
			if (geoCountry) {
				handleCountryChange(geoCountry)
			}
		}
	}
})
</script>
