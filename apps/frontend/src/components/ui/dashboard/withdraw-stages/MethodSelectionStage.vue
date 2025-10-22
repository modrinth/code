<template>
	<div class="flex flex-col gap-6">
		<Admonition v-if="shouldShowTaxLimitWarning" type="warning">
			<IntlFormatted
				:message-id="messages.taxLimitWarning"
				:values="{
					amount: formatMoney(withdrawContext.maxWithdrawAmount.value),
				}"
			>
				<template #b="{ children }">
					<span class="font-bold">
						<component :is="() => normalizeChildren(children)" />
					</span>
				</template>
				<template #tax-link="{ children }">
					<span class="cursor-pointer text-link" @click="onShowTaxForm">
						<component :is="() => normalizeChildren(children)" />
					</span>
				</template>
			</IntlFormatted>
		</Admonition>
		<div class="flex flex-col gap-2.5">
			<div class="flex flex-row gap-1 align-middle">
				<span class="align-middle font-semibold text-contrast">{{
					formatMessage(messages.region)
				}}</span>
				<UnknownIcon
					v-tooltip="formatMessage(messages.regionTooltip)"
					class="mt-auto size-5 text-secondary"
				/>
			</div>
			<Combobox
				:model-value="selectedCountryCode"
				:options="countries"
				:placeholder="formatMessage(messages.countryPlaceholder)"
				searchable
				:search-placeholder="formatMessage(messages.countrySearchPlaceholder)"
				:max-height="240"
				class="h-10"
				@update:model-value="handleCountryChange"
			/>
		</div>
		<div class="flex flex-col gap-2.5">
			<span class="align-middle font-semibold text-contrast">{{
				formatMessage(messages.selectMethod)
			}}</span>
			<ButtonStyled
				v-for="method in paymentOptions"
				:key="method.value"
				:color="
					withdrawContext.withdrawData.value.selectedMethod === method.value ? 'green' : 'standard'
				"
				:highlighted="withdrawContext.withdrawData.value.selectedMethod === method.value"
				type="chip"
			>
				<button class="!h-10 !justify-start !gap-2" @click="handleMethodSelection(method)">
					<component :is="method.icon" />
					{{ typeof method.label === 'string' ? method.label : formatMessage(method.label) }}
					<span class="ml-auto text-secondary">{{ method.fee }}</span>
				</button>
			</ButtonStyled>
		</div>
	</div>
</template>

<script setup lang="ts">
import {
	GiftIcon,
	HeartIcon,
	LandmarkIcon,
	PayPalIcon,
	PolygonIcon,
	UnknownIcon,
	VenmoIcon,
} from '@modrinth/assets'
import {
	Admonition,
	ButtonStyled,
	Combobox,
	injectNotificationManager,
	paymentMethodMessages,
	useDebugLogger,
} from '@modrinth/ui'
import { formatMoney } from '@modrinth/utils'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { IntlFormatted } from '@vintl/vintl/components'
import { useGeolocation } from '@vueuse/core'
import { all } from 'iso-3166-1'

import { useUserCountry } from '@/composables/country.ts'
import { type PayoutMethod, useWithdrawContext } from '@/providers/creator-withdraw.ts'
import { getBlockchainIcon } from '@/utils/finance-icons'
import { getRailConfig } from '@/utils/muralpay-rails'
import { normalizeChildren } from '@/utils/vue-children.ts'

const debug = useDebugLogger('MethodSelectionStage')
const withdrawContext = useWithdrawContext()
const userCountry = useUserCountry()
const { coords } = useGeolocation()
const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()

const messages = defineMessages({
	taxLimitWarning: {
		id: 'dashboard.creator-withdraw-modal.method-selection.tax-limit-warning',
		defaultMessage:
			'Your withdraw limit is <b>{amount}</b>, <tax-link>complete a tax form</tax-link> to withdraw more.',
	},
	region: {
		id: 'dashboard.creator-withdraw-modal.method-selection.region',
		defaultMessage: 'Region',
	},
	regionTooltip: {
		id: 'dashboard.creator-withdraw-modal.method-selection.region-tooltip',
		defaultMessage: 'Some payout methods are not available in certain regions.',
	},
	countryPlaceholder: {
		id: 'dashboard.creator-withdraw-modal.method-selection.country-placeholder',
		defaultMessage: 'Select your country',
	},
	countrySearchPlaceholder: {
		id: 'dashboard.creator-withdraw-modal.method-selection.country-search-placeholder',
		defaultMessage: 'Search countries...',
	},
	selectMethod: {
		id: 'dashboard.creator-withdraw-modal.method-selection.select-method',
		defaultMessage: 'Select withdraw method',
	},
	errorTitle: {
		id: 'dashboard.creator-withdraw-modal.method-selection.error-title',
		defaultMessage: 'Failed to load payment methods',
	},
	errorText: {
		id: 'dashboard.creator-withdraw-modal.method-selection.error-text',
		defaultMessage: 'Unable to fetch available payment methods. Please try again later.',
	},
	bankTransferFallback: {
		id: 'dashboard.creator-withdraw-modal.method-selection.bank-transfer-fallback',
		defaultMessage: 'Bank transfer ({code})',
	},
})

defineProps<{
	onShowTaxForm: () => void
}>()

const emit = defineEmits<{
	(e: 'close-modal'): void
}>()

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

const loading = ref(false)

watch(
	() => withdrawContext.withdrawData.value.selectedCountry,
	async (country) => {
		console.debug('[MethodSelectionStage] Watch triggered, country:', country)
		if (!country) {
			withdrawContext.availableMethods.value = []
			return
		}

		loading.value = true
		console.debug('[MethodSelectionStage] Fetching payout methods for country:', country.id)

		try {
			const methods = (await useBaseFetch('payout/methods', {
				apiVersion: 3,
				query: { country: country.id },
			})) as PayoutMethod[]
			console.debug('[MethodSelectionStage] Received payout methods:', methods)
			withdrawContext.availableMethods.value = methods
		} catch (e) {
			console.error('[MethodSelectionStage] Failed to fetch payout methods:', e)
			addNotification({
				title: formatMessage(messages.errorTitle),
				text: formatMessage(messages.errorText),
				type: 'error',
			})
			emit('close-modal')
		} finally {
			loading.value = false
		}
	},
	{ immediate: true },
)

const paymentOptions = computed(() => {
	const methods = withdrawContext.availableMethods.value
	if (!methods || methods.length === 0) {
		debug('No payment methods available')
		return []
	}

	debug('Available methods:', methods)

	const options = []

	const tremendousMethods = methods.filter((m) => m.type === 'tremendous')

	const paypalMethods = tremendousMethods.filter((m) => m.category === 'paypal')
	if (paypalMethods.length > 0) {
		options.push({
			value: 'paypal',
			label: paymentMethodMessages.paypal,
			icon: PayPalIcon,
			methodId: paypalMethods[0].id,
			fee: '≈ 6%, max $25',
			type: 'tremendous',
		})
	}

	const venmoMethods = tremendousMethods.filter((m) => m.category === 'venmo')
	if (venmoMethods.length > 0) {
		options.push({
			value: 'venmo',
			label: paymentMethodMessages.venmo,
			icon: VenmoIcon,
			methodId: venmoMethods[0].id,
			fee: '≈ 6%, max $25',
			type: 'tremendous',
		})
	}

	const merchantMethods = tremendousMethods.filter(
		(m) => m.category === 'merchant_card' || m.category === 'merchant_cards',
	)
	if (merchantMethods.length > 0) {
		options.push({
			value: 'merchant_card',
			label: paymentMethodMessages.giftCard,
			icon: GiftIcon,
			methodId: undefined,
			fee: '≈ 0%',
			type: 'tremendous',
		})
	}

	const charityMethods = tremendousMethods.filter((m) => m.category === 'charity')
	if (charityMethods.length > 0) {
		options.push({
			value: 'charity',
			label: paymentMethodMessages.charity,
			icon: HeartIcon,
			methodId: undefined,
			fee: '≈ 0%',
			type: 'tremendous',
		})
	}

	const muralPayMethods = methods.filter((m) => m.type === 'muralpay')
	for (const method of muralPayMethods) {
		const methodId = method.id

		if (methodId.startsWith('fiat_')) {
			const railCode = methodId.replace('fiat_', '')
			const rail = getRailConfig(methodId)

			if (!rail) {
				debug('Warning: No rail config found for', methodId)
				continue
			}

			options.push({
				value: methodId,
				label:
					rail.name ||
					formatMessage(messages.bankTransferFallback, { code: railCode.toUpperCase() }),
				icon: LandmarkIcon,
				methodId: method.id,
				fee: rail.fee,
				type: 'fiat',
			})
		} else if (methodId.startsWith('blockchain_')) {
			const rail = getRailConfig(methodId)

			if (!rail) {
				debug('Warning: No rail config found for', methodId)
				continue
			}

			options.push({
				value: methodId,
				label: rail.name || method.name,
				icon: getBlockchainIcon(rail.blockchain || 'POLYGON') || PolygonIcon,
				methodId: method.id,
				fee: rail.fee,
				type: 'crypto',
			})
		}
	}

	const sortOrder: Record<string, number> = {
		fiat: 1,
		paypal: 2,
		venmo: 3,
		visa_card: 4,
		merchant_card: 5,
		charity: 6,
		crypto: 7,
	}
	options.sort((a, b) => {
		const aOrder = sortOrder[a.type] ?? sortOrder[a.value] ?? 999
		const bOrder = sortOrder[b.type] ?? sortOrder[b.value] ?? 999
		return aOrder - bOrder
	})

	debug('Payment options computed:', options)
	return options
})

function handleMethodSelection(option: {
	value: string
	methodId: string | undefined
	type: string
}) {
	withdrawContext.withdrawData.value.selectedMethod = option.value
	withdrawContext.withdrawData.value.selectedMethodId = option.methodId ?? null

	if (option.type === 'tremendous') {
		withdrawContext.withdrawData.value.selectedProvider = 'tremendous'
	} else if (option.type === 'fiat' || option.type === 'crypto') {
		withdrawContext.withdrawData.value.selectedProvider = 'muralpay'
	} else {
		withdrawContext.withdrawData.value.selectedProvider = 'muralpay'
	}
}

watch(paymentOptions, (newOptions) => {
	withdrawContext.withdrawData.value.selectedMethod = null
	withdrawContext.withdrawData.value.selectedMethodId = null
	withdrawContext.withdrawData.value.selectedProvider = null

	if (newOptions.length === 1) {
		const option = newOptions[0]
		handleMethodSelection(option)
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
