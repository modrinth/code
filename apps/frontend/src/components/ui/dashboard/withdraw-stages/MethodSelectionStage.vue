<template>
	<div class="flex flex-col gap-6">
		<Admonition v-if="shouldShowTaxLimitWarning" type="warning">
			Your withdraw limit is
			<span class="font-bold">{{ formatMoney(withdrawContext.maxWithdrawAmount.value) }}</span>, <span
				class="cursor-pointer text-link" @click="onShowTaxForm">complete a tax form</span> to
			withdraw more.
		</Admonition>
		<div class="flex flex-col gap-2.5">
			<div class="flex flex-row gap-1 align-middle">
				<span class="align-middle font-semibold text-contrast">Region</span>
				<UnknownIcon v-tooltip="'Some payout methods are not available in certain regions.'"
					class="mt-auto size-5 text-secondary" />
			</div>
			<Combobox :model-value="selectedCountryCode" :options="countries" placeholder="Select your country" searchable
				search-placeholder="Search countries..." :max-height="240" class="h-10"
				@update:model-value="handleCountryChange" />
		</div>
		<div class="flex flex-col gap-2.5">
			<span class="align-middle font-semibold text-contrast">Select withdraw method</span>
			<ButtonStyled v-for="method in paymentOptions" :key="method.value" :color="withdrawContext.withdrawData.value.selectedMethod === method.value ? 'green' : 'standard'
				" :highlighted="withdrawContext.withdrawData.value.selectedMethod === method.value" type="chip">
				<button class="!h-10 !justify-start !gap-2" @click="handleMethodSelection(method)">
					<component :is="method.icon" /> {{ method.label }}
				</button>
			</ButtonStyled>
		</div>
	</div>
</template>

<script setup lang="ts">
import {
	LandmarkIcon,
	PolygonIcon,
	UnknownIcon
} from '@modrinth/assets'
import { Admonition, ButtonStyled, Combobox, injectNotificationManager, useDebugLogger } from '@modrinth/ui'
import { formatMoney } from '@modrinth/utils'
import { useGeolocation } from '@vueuse/core'
import { all } from 'iso-3166-1'

import { useUserCountry } from '@/composables/country.ts'
import { useWithdrawContext } from '@/providers/creator-withdraw.ts'
import { getBlockchainIcon } from '@/utils/blockchain-icons'
import { getRailConfig } from '@/utils/muralpay-rails'

const debug = useDebugLogger('MethodSelectionStage')
const withdrawContext = useWithdrawContext()
const userCountry = useUserCountry()
const { coords } = useGeolocation()
const { addNotification } = injectNotificationManager()

const props = defineProps<{
	onShowTaxForm: () => void
}>()

const emit = defineEmits<{
	(e: 'close-modal'): void
}>();

interface PayoutMethod {
	id: string
	type_: string
	name: string
	supported_countries: string[]
	image_url: string | null
	image_logo_url: string | null
	fee: {
		percentage: number
		min: number
		max: number | null
	}
	config?: {
		fiat?: string | null
		blockchain?: string[]
	}
}

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

const availableMethods = ref<PayoutMethod[]>([])
const loading = ref(false)

watch(
	() => withdrawContext.withdrawData.value.selectedCountry,
	async (country) => {
		console.debug('[MethodSelectionStage] Watch triggered, country:', country)
		if (!country) {
			availableMethods.value = []
			return
		}

		loading.value = true
		console.debug('[MethodSelectionStage] Fetching payout methods for country:', country.id)

		try {
			const methods = await useBaseFetch('payout/methods', {
				apiVersion: 3,
				query: { country: country.id }
			}) as PayoutMethod[]
			console.debug('[MethodSelectionStage] Received payout methods:', methods)
			availableMethods.value = methods
		} catch (e) {
			console.error('[MethodSelectionStage] Failed to fetch payout methods:', e)
			addNotification({
				title: 'Failed to load payment methods',
				text: 'Unable to fetch available payment methods. Please try again later.',
				type: 'error',
			})
			emit('close-modal')
		} finally {
			loading.value = false
		}
	},
	{ immediate: true }
)

const muralPayMethod = computed(() =>
	availableMethods.value.find(m => m.type_ === 'mural_pay' || m.id === 'muralpay')
)

const paymentOptions = computed(() => {
	const muralpay = muralPayMethod.value
	if (!muralpay?.config) return []

	const options = []

	if (muralpay.config.fiat) {
		const fiatRailId = muralpay.config.fiat.toLowerCase()
		options.push({
			value: fiatRailId,
			label: 'Bank transfer',
			icon: LandmarkIcon,
			methodId: muralpay.id,
			type: 'fiat'
		})
	}

	for (const blockchain of muralpay.config.blockchain || []) {
		const blockchainName = blockchain.replace('usdc_', '')
		const railId = `${blockchainName}-usdc`
		const rail = getRailConfig(railId)

		options.push({
			value: railId,
			label: rail?.name || blockchain,
			icon: getBlockchainIcon(blockchain) || PolygonIcon,
			methodId: muralpay.id,
			type: 'crypto'
		})
	}

	return options
})

function handleMethodSelection(option: { value: string; methodId: string }) {
	withdrawContext.withdrawData.value.selectedMethod = option.value
	withdrawContext.withdrawData.value.selectedMethodId = option.methodId
	withdrawContext.withdrawData.value.selectedProvider = 'muralpay'
}

watch(paymentOptions, (newOptions) => {
	const currentMethod = withdrawContext.withdrawData.value.selectedMethod
	if (currentMethod && !newOptions.find(o => o.value === currentMethod)) {
		withdrawContext.withdrawData.value.selectedMethod = null
		withdrawContext.withdrawData.value.selectedMethodId = null
		withdrawContext.withdrawData.value.selectedProvider = null
	}
})

function handleCountryChange(countryCode: string | null) {
	debug('handleCountryChange called with:', countryCode)
	if (countryCode) {
		const normalizedCode = countryCode.toUpperCase()
		const country = all().find((c) => c.alpha2 === normalizedCode)
		debug('Found country:', country)
		if (country) {
			withdrawContext.withdrawData.value.selectedCountry = {
				id: country.alpha2,
				name: country.alpha2 === 'TW' ? 'Taiwan' : country.country,
			}
			debug('Set selectedCountry to:', withdrawContext.withdrawData.value.selectedCountry)
		}
	} else {
		withdrawContext.withdrawData.value.selectedCountry = null
	}
}

debug('Setup: userCountry.value =', userCountry.value)
debug('Setup: current selectedCountry =', withdrawContext.withdrawData.value.selectedCountry)

if (!withdrawContext.withdrawData.value.selectedCountry) {
	const defaultCountryCode = userCountry.value || 'US'
	debug('Setup: calling handleCountryChange with', defaultCountryCode)
	handleCountryChange(defaultCountryCode)
	debug('Setup: selectedCountryCode computed =', selectedCountryCode.value)
}

async function getCountryFromGeoIP(lat: number, lon: number): Promise<string | null> {
	try {
		const response = await fetch(
			`https://api.bigdatacloud.net/data/reverse-geocode-client?latitude=${lat}&longitude=${lon}&localityLanguage=en`,
		)
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
