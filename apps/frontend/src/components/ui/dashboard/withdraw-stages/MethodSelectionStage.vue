<template>
	<div class="flex flex-col gap-6">
		<Admonition type="warning" v-if="shouldShowTaxLimitWarning">
			Your withdraw limit is <span class="font-bold">{{ formatMoney(withdrawContext.maxWithdrawAmount.value) }}</span>,
			<span class="text-link cursor-pointer" @click="onShowTaxForm">complete a
				tax form</span> to withdraw more.
		</Admonition>
		<div class="flex flex-col gap-2.5">
			<div class="flex flex-row gap-1 align-middle">
				<span class="text-contrast font-semibold align-middle">Region</span>
				<UnknownIcon v-tooltip="'Some payout methods are not available in certain regions.'"
					class="size-5 mt-auto text-secondary" />
			</div>
			<Combobox :model-value="selectedCountryCode" :options="countries" placeholder="Select your country" searchable
				search-placeholder="Search countries..." :max-height="240" @update:model-value="handleCountryChange"
				class="h-10" />
		</div>
		<div class="flex flex-col gap-2.5">
			<span class="text-contrast font-semibold align-middle">Select withdraw method</span>
			<ButtonStyled v-for="method in paymentMethods" :key="method.value"
				:color="withdrawContext.withdrawData.value.selectedMethod === method.value ? 'green' : 'standard'"
				:highlighted="withdrawContext.withdrawData.value.selectedMethod === method.value"
				type="chip">
				<button @click="handleMethodSelection(method.value)" class="!justify-start !gap-2 !h-10">
					<component :is="method.icon" /> {{ method.label }}
				</button>
			</ButtonStyled>
		</div>
	</div>
</template>

<script setup lang="ts">
import { useUserCountry } from '@/composables/country.ts';
import { useWithdrawContext } from '@/providers/creator-withdraw.ts';
import { GiftIcon, LandmarkIcon, PayPalIcon, PolygonIcon, UnknownIcon, VenmoIcon } from '@modrinth/assets';
import { Admonition, ButtonStyled, Combobox } from '@modrinth/ui';
import { formatMoney } from '@modrinth/utils';
import { useGeolocation } from '@vueuse/core';
import { all } from 'iso-3166-1';

const withdrawContext = useWithdrawContext()
const userCountry = useUserCountry()
const { coords } = useGeolocation()

defineProps<{
	onShowTaxForm: () => void
}>();

const countries = computed(() =>
	all().map((x) => ({
		value: x.alpha2,
		label: x.alpha2 === 'TW' ? 'Taiwan' : x.country,
	})),
)

const selectedCountryCode = computed(() => withdrawContext.withdrawData.value.selectedCountry?.id)

const shouldShowTaxLimitWarning = computed(() => {
	const balance = withdrawContext.balance.value
	if (!balance) return false

	const formIncomplete = balance.form_completion_status !== 'complete'
	const wouldHitLimit = (balance.withdrawn_ytd ?? 0) + (balance.available ?? 0) >= 600

	return formIncomplete && wouldHitLimit
})

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

const paymentMethods = [
	{ value: 'bank', label: 'Bank transfer', icon: LandmarkIcon },
	{ value: 'paypal', label: 'PayPal', icon: PayPalIcon },
	{ value: 'venmo', label: 'Venmo', icon: VenmoIcon },
	{ value: 'crypto', label: 'Polygon (Crypto)', icon: PolygonIcon },
	{ value: 'gift_card', label: 'Gift card', icon: GiftIcon },
]

function handleMethodSelection(methodValue: string) {
	const methodToProvider: Record<string, 'tremendous' | 'muralpay'> = {
		gift_card: 'tremendous',
		paypal: 'tremendous',
		venmo: 'tremendous',
		bank: 'muralpay',
		crypto: 'muralpay',
	}

	withdrawContext.withdrawData.value.selectedMethod = methodValue as any
	withdrawContext.withdrawData.value.selectedProvider = methodToProvider[methodValue]
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
