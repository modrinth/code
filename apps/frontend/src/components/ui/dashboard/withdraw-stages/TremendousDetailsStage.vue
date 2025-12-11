<template>
	<div class="flex flex-col gap-4 sm:gap-5">
		<Transition
			enter-active-class="transition-all duration-300 ease-out"
			enter-from-class="opacity-0 max-h-0"
			enter-to-class="opacity-100 max-h-40"
			leave-active-class="transition-all duration-200 ease-in"
			leave-from-class="opacity-100 max-h-40"
			leave-to-class="opacity-0 max-h-0"
		>
			<div v-if="isUnverifiedEmail" class="overflow-hidden">
				<Admonition type="warning" :header="formatMessage(messages.unverifiedEmailHeader)">
					{{ formatMessage(messages.unverifiedEmailMessage) }}
				</Admonition>
			</div>
		</Transition>

		<Transition
			enter-active-class="transition-all duration-300 ease-out"
			enter-from-class="opacity-0 max-h-0"
			enter-to-class="opacity-100 max-h-40"
			leave-active-class="transition-all duration-200 ease-in"
			leave-from-class="opacity-100 max-h-40"
			leave-to-class="opacity-0 max-h-0"
		>
			<div v-if="shouldShowUsdWarning" class="overflow-hidden">
				<Admonition type="warning" :header="formatMessage(messages.usdPaypalWarningHeader)">
					<IntlFormatted :message-id="messages.usdPaypalWarningMessage">
						<template #direct-paypal-link="{ children }">
							<span class="cursor-pointer text-link" @click="switchToDirectPaypal">
								<component :is="() => normalizeChildren(children)" />
							</span>
						</template>
					</IntlFormatted>
				</Admonition>
			</div>
		</Transition>

		<div v-if="!showGiftCardSelector && selectedMethodDisplay" class="flex flex-col gap-2.5">
			<label>
				<span class="text-md font-semibold text-contrast">
					{{ formatMessage(messages.paymentMethod) }}
				</span>
			</label>
			<div
				class="flex min-h-[44px] items-center gap-2 rounded-[14px] bg-surface-2 px-4 py-2.5 sm:min-h-0"
			>
				<component :is="selectedMethodDisplay.icon" class="size-5 shrink-0" />
				<span class="break-words text-sm font-semibold text-contrast sm:text-[1rem]">{{
					typeof selectedMethodDisplay.label === 'string'
						? selectedMethodDisplay.label
						: formatMessage(selectedMethodDisplay.label)
				}}</span>
			</div>
		</div>

		<div class="flex flex-col gap-2.5">
			<label>
				<span class="text-md font-semibold text-contrast"
					>{{ formatMessage(formFieldLabels.email) }} <span class="text-red">*</span></span
				>
			</label>
			<input
				v-model="deliveryEmail"
				type="email"
				:placeholder="formatMessage(formFieldPlaceholders.emailPlaceholder)"
				autocomplete="email"
				class="w-full rounded-[14px] bg-surface-4 px-4 py-3 text-contrast placeholder:text-secondary sm:py-2.5"
			/>
		</div>

		<div v-if="showGiftCardSelector" class="flex flex-col gap-1">
			<div class="flex flex-col gap-2.5">
				<label>
					<span class="text-md font-semibold text-contrast"
						>{{ categoryLabel }} <span class="text-red">*</span></span
					>
				</label>
				<Combobox
					v-model="selectedGiftCardId"
					:options="rewardOptions"
					:placeholder="`Select ${categoryLabel.toLowerCase()}`"
					searchable
					:search-placeholder="`Search ${categoryLabelPlural.toLowerCase()}...`"
					class="h-10"
				>
					<template #selected>
						<div v-if="selectedRewardOption" class="flex items-center gap-2">
							<img
								v-if="selectedRewardOption.imageUrl"
								:src="selectedRewardOption.imageUrl"
								:alt="selectedRewardOption.label"
								class="size-5 rounded-full object-cover"
								loading="lazy"
							/>
							<span class="font-semibold leading-tight">{{ selectedRewardOption.label }}</span>
						</div>
					</template>
					<template v-for="option in rewardOptions" :key="option.value" #[`option-${option.value}`]>
						<div class="flex items-center gap-2">
							<img
								v-if="option.imageUrl"
								:src="option.imageUrl"
								:alt="option.label"
								class="size-5 rounded-full object-cover"
								loading="lazy"
							/>
							<span class="font-semibold leading-tight">{{ option.label }}</span>
						</div>
					</template>
				</Combobox>
			</div>
			<span v-if="selectedMethodDetails" class="text-secondary">
				{{ formatMoney(effectiveMinAmount) }} min,
				{{ formatMoney(selectedMethodDetails.interval?.standard?.max ?? effectiveMaxAmount) }}
				max withdrawal amount.
			</span>
		</div>

		<div class="flex flex-col gap-2.5">
			<label>
				<span class="text-md font-semibold text-contrast"
					>{{ formatMessage(formFieldLabels.amount) }} <span class="text-red">*</span></span
				>
			</label>

			<div v-if="showGiftCardSelector && useFixedDenominations" class="flex flex-col gap-2.5">
				<Chips
					v-model="selectedDenomination"
					:items="denominationOptions"
					:format-label="(amt: number) => formatMoney(amt)"
					:never-empty="false"
					:capitalize="false"
				/>
				<span v-if="denominationOptions.length === 0" class="text-error text-sm">
					No denominations available for your current balance
				</span>
			</div>

			<div v-else class="flex flex-col gap-2">
				<RevenueInputField
					v-model="formData.amount"
					v-model:selected-currency="selectedCurrency"
					:max-amount="effectiveMaxAmount"
					:min-amount="effectiveMinAmount"
					:show-currency-selector="showPayPalCurrencySelector"
					:currency-options="currencyOptions"
				/>
			</div>

			<WithdrawFeeBreakdown
				v-if="allRequiredFieldsFilled"
				:amount="formData.amount || 0"
				:fee="calculatedFee"
				:fee-loading="feeLoading"
				:exchange-rate="exchangeRate"
				:local-currency="showPayPalCurrencySelector ? selectedCurrency : undefined"
			/>

			<Checkbox v-model="agreedTerms">
				<span>
					<IntlFormatted :message-id="financialMessages.rewardsProgramTermsAgreement">
						<template #terms-link="{ children }">
							<nuxt-link to="/legal/cmp" class="text-link">
								<component :is="() => normalizeChildren(children)" />
							</nuxt-link>
						</template>
					</IntlFormatted>
				</span>
			</Checkbox>
		</div>
	</div>
</template>

<script setup lang="ts">
import {
	Admonition,
	Checkbox,
	Chips,
	Combobox,
	financialMessages,
	formFieldLabels,
	formFieldPlaceholders,
	normalizeChildren,
	paymentMethodMessages,
	useDebugLogger,
} from '@modrinth/ui'
import { formatMoney } from '@modrinth/utils'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { IntlFormatted } from '@vintl/vintl/components'
import { useDebounceFn } from '@vueuse/core'
import { computed, onMounted, ref, watch } from 'vue'

import RevenueInputField from '@/components/ui/dashboard/RevenueInputField.vue'
import WithdrawFeeBreakdown from '@/components/ui/dashboard/WithdrawFeeBreakdown.vue'
import { useAuth } from '@/composables/auth.js'
import { useBaseFetch } from '@/composables/fetch.js'
import { type PayoutMethod, useWithdrawContext } from '@/providers/creator-withdraw.ts'

const debug = useDebugLogger('TremendousDetailsStage')
const {
	withdrawData,
	maxWithdrawAmount,
	availableMethods,
	paymentOptions,
	calculateFees,
	setStage,
	paymentMethodsCache,
} = useWithdrawContext()
const { formatMessage } = useVIntl()
const auth = await useAuth()

const userEmail = computed(() => {
	return (auth.value.user as any)?.email || ''
})

const providerData = withdrawData.value.providerData
const initialDeliveryEmail =
	providerData.type === 'tremendous'
		? providerData.deliveryEmail || userEmail.value || ''
		: userEmail.value || ''
const deliveryEmail = ref<string>(initialDeliveryEmail)

const showGiftCardSelector = computed(() => {
	const method = withdrawData.value.selection.method
	return method === 'merchant_card' || method === 'charity'
})

const showPayPalCurrencySelector = computed(() => {
	const method = withdrawData.value.selection.method
	return method === 'paypal'
})

const shouldShowUsdWarning = computed(() => {
	const method = withdrawData.value.selection.method
	const currency = selectedCurrency.value
	return method === 'paypal' && currency === 'USD'
})

const selectedMethodDisplay = computed(() => {
	const method = withdrawData.value.selection.method
	if (!method) return null
	return paymentOptions.value.find((m) => m.value === method) || null
})

const categoryLabel = computed(() => {
	const method = withdrawData.value.selection.method
	switch (method) {
		case 'visa_card':
			return formatMessage(paymentMethodMessages.virtualVisa)
		case 'merchant_card':
			return formatMessage(paymentMethodMessages.giftCard)
		case 'charity':
			return formatMessage(paymentMethodMessages.charity)
		default:
			return formatMessage(messages.reward)
	}
})

const categoryLabelPlural = computed(() => {
	const method = withdrawData.value.selection.method
	switch (method) {
		case 'visa_card':
			return formatMessage(paymentMethodMessages.virtualVisaPlural)
		case 'merchant_card':
			return formatMessage(paymentMethodMessages.giftCardPlural)
		case 'charity':
			return formatMessage(paymentMethodMessages.charityPlural)
		default:
			return formatMessage(messages.rewardPlural)
	}
})

const isUnverifiedEmail = computed(() => {
	if (!deliveryEmail.value || !userEmail.value) return false
	return deliveryEmail.value.toLowerCase() !== userEmail.value.toLowerCase()
})

const maxAmount = computed(() => maxWithdrawAmount.value)
const roundedMaxAmount = computed(() => Math.floor(maxAmount.value * 100) / 100)

const formData = ref<Record<string, any>>({
	amount: withdrawData.value.calculation.amount || undefined,
})

const selectedGiftCardId = ref<string | null>(withdrawData.value.selection.methodId || null)

const currencyOptions = [
	{ value: 'USD', label: 'USD' },
	{ value: 'AUD', label: 'AUD' },
	{ value: 'CAD', label: 'CAD' },
	{ value: 'CHF', label: 'CHF' },
	{ value: 'CZK', label: 'CZK' },
	{ value: 'DKK', label: 'DKK' },
	{ value: 'EUR', label: 'EUR' },
	{ value: 'GBP', label: 'GBP' },
	{ value: 'MXN', label: 'MXN' },
	{ value: 'NOK', label: 'NOK' },
	{ value: 'NZD', label: 'NZD' },
	{ value: 'PLN', label: 'PLN' },
	{ value: 'SEK', label: 'SEK' },
	{ value: 'SGD', label: 'SGD' },
]

function getCurrencyFromCountryCode(countryCode: string | undefined): string {
	if (!countryCode) return 'USD'

	const code = countryCode.toUpperCase()

	const countryToCurrency: Record<string, string> = {
		US: 'USD', // United States
		GB: 'GBP', // UK
		CA: 'CAD', // Canada
		AU: 'AUD', // Australia
		CH: 'CHF', // Switzerland
		CZ: 'CZK', // Czech Republic
		DK: 'DKK', // Denmark
		MX: 'MXN', // Mexico
		NO: 'NOK', // Norway
		NZ: 'NZD', // New Zealand
		PL: 'PLN', // Poland
		SE: 'SEK', // Sweden
		SG: 'SGD', // Singapore

		// Eurozone countries
		AT: 'EUR', // Austria
		BE: 'EUR', // Belgium
		CY: 'EUR', // Cyprus
		EE: 'EUR', // Estonia
		FI: 'EUR', // Finland
		FR: 'EUR', // France
		DE: 'EUR', // Germany
		GR: 'EUR', // Greece
		IE: 'EUR', // Ireland
		IT: 'EUR', // Italy
		LV: 'EUR', // Latvia
		LT: 'EUR', // Lithuania
		LU: 'EUR', // Luxembourg
		MT: 'EUR', // Malta
		NL: 'EUR', // Netherlands
		PT: 'EUR', // Portugal
		SK: 'EUR', // Slovakia
		SI: 'EUR', // Slovenia
		ES: 'EUR', // Spain
	}

	return countryToCurrency[code] || 'USD'
}

const initialCurrency = getCurrencyFromCountryCode(withdrawData.value.selection.country?.id)
const selectedCurrency = ref<string>(initialCurrency)

const agreedTerms = computed({
	get: () => withdrawData.value.agreedTerms,
	set: (value) => {
		withdrawData.value.agreedTerms = value
	},
})

const calculatedFee = ref<number>(0)
const exchangeRate = ref<number | null>(null)
const feeLoading = ref(false)

const rewardOptions = ref<
	Array<{
		value: string
		label: string
		imageUrl?: string
		methodDetails?: {
			id: string
			name: string
			interval?: {
				fixed?: { values: number[] }
				standard?: { min: number; max: number }
			}
		}
	}>
>([])

const selectedRewardOption = computed(() => {
	if (!selectedGiftCardId.value) return null
	return rewardOptions.value.find((opt) => opt.value === selectedGiftCardId.value) || null
})

const selectedMethodDetails = computed(() => {
	console.log(rewardOptions.value, selectedGiftCardId.value)
	if (!selectedGiftCardId.value) return null
	const option = rewardOptions.value.find((opt) => opt.value === selectedGiftCardId.value)
	debug('Selected method details:', option?.methodDetails)
	return option?.methodDetails || null
})

const useFixedDenominations = computed(() => {
	const hasFixed = !!selectedMethodDetails.value?.interval?.fixed?.values
	debug('Use fixed denominations:', hasFixed, selectedMethodDetails.value?.interval)
	return hasFixed
})

const denominationOptions = computed(() => {
	const fixedValues = selectedMethodDetails.value?.interval?.fixed?.values
	if (!fixedValues) return []

	const filtered = fixedValues
		.filter((amount) => amount <= roundedMaxAmount.value)
		.sort((a, b) => a - b)
	debug(
		'Denomination options (filtered by max):',
		filtered,
		'from',
		fixedValues,
		'max:',
		roundedMaxAmount.value,
	)
	return filtered
})

const effectiveMinAmount = computed(() => {
	return selectedMethodDetails.value?.interval?.standard?.min || 0.01
})

const effectiveMaxAmount = computed(() => {
	const methodMax = selectedMethodDetails.value?.interval?.standard?.max
	if (methodMax !== undefined && methodMax !== null) {
		return Math.min(roundedMaxAmount.value, methodMax)
	}
	return roundedMaxAmount.value
})

const selectedDenomination = computed({
	get: () => formData.value.amount,
	set: (value) => {
		formData.value.amount = value
	},
})

const allRequiredFieldsFilled = computed(() => {
	const amount = formData.value.amount
	if (!amount || amount <= 0) return false

	if (!deliveryEmail.value) return false

	if (showGiftCardSelector.value && !selectedGiftCardId.value) return false

	return true
})

const calculateFeesDebounced = useDebounceFn(async () => {
	const amount = formData.value.amount
	if (!amount || amount <= 0) {
		calculatedFee.value = 0
		exchangeRate.value = null
		return
	}

	const methodId = showGiftCardSelector.value
		? selectedGiftCardId.value
		: withdrawData.value.selection.methodId

	if (!methodId) {
		calculatedFee.value = 0
		exchangeRate.value = null
		return
	}

	feeLoading.value = true
	try {
		await calculateFees()
		calculatedFee.value = withdrawData.value.calculation.fee ?? 0
		exchangeRate.value = withdrawData.value.calculation.exchangeRate
	} catch (error) {
		console.error('Failed to calculate fees:', error)
		calculatedFee.value = 0
		exchangeRate.value = null
	} finally {
		feeLoading.value = false
	}
}, 500)

watch(deliveryEmail, (newEmail) => {
	if (withdrawData.value.providerData.type === 'tremendous') {
		withdrawData.value.providerData.deliveryEmail = newEmail
	}
})

watch(
	selectedCurrency,
	(newCurrency) => {
		if (withdrawData.value.providerData.type === 'tremendous') {
			;(withdrawData.value.providerData as any).currency = newCurrency
		}
	},
	{ immediate: true },
)

watch(
	() => withdrawData.value.selection.country?.id,
	(newCountryId) => {
		if (showPayPalCurrencySelector.value && newCountryId) {
			const detectedCurrency = getCurrencyFromCountryCode(newCountryId)
			selectedCurrency.value = detectedCurrency
		}
	},
)

watch(
	[() => formData.value.amount, selectedGiftCardId, deliveryEmail, selectedCurrency],
	() => {
		withdrawData.value.calculation.amount = formData.value.amount ?? 0

		if (showGiftCardSelector.value && selectedGiftCardId.value) {
			withdrawData.value.selection.methodId = selectedGiftCardId.value
		}

		if (allRequiredFieldsFilled.value) {
			feeLoading.value = true
			calculateFeesDebounced()
		} else {
			calculatedFee.value = 0
			exchangeRate.value = null
			feeLoading.value = false
		}
	},
	{ deep: true },
)

onMounted(async () => {
	const methods = availableMethods.value
	const selectedMethod = withdrawData.value.selection.method

	rewardOptions.value = methods
		.filter((m) => m.type === 'tremendous')
		.filter(
			(m) =>
				m.category === selectedMethod ||
				(selectedMethod === 'merchant_card' && m.category === 'visa_card'),
		)
		.map((m) => ({
			value: m.id,
			label: m.name,
			imageUrl: m.image_url || m.image_logo_url || undefined,
			methodDetails: {
				id: m.id,
				name: m.name,
				interval: m.interval,
			},
		}))

	debug('Loaded reward options:', rewardOptions.value.length, 'methods')
	debug('Sample method with interval:', rewardOptions.value[0]?.methodDetails)

	if (allRequiredFieldsFilled.value) {
		feeLoading.value = true
		calculateFeesDebounced()
	}
})

watch(
	() => withdrawData.value.selection.method,
	(newMethod, oldMethod) => {
		if (oldMethod && newMethod !== oldMethod) {
			formData.value = {
				amount: undefined,
			}
			selectedGiftCardId.value = null
			calculatedFee.value = 0
			exchangeRate.value = null

			// Clear currency when switching away from PayPal International
			if (newMethod !== 'paypal' && withdrawData.value.providerData.type === 'tremendous') {
				;(withdrawData.value.providerData as any).currency = undefined
			}
		}
	},
)

async function switchToDirectPaypal() {
	withdrawData.value.selection.country = {
		id: 'US',
		name: 'United States',
	}

	let usMethods = paymentMethodsCache.value['US']

	if (!usMethods) {
		try {
			usMethods = (await useBaseFetch('payout/methods', {
				apiVersion: 3,
				query: { country: 'US' },
			})) as PayoutMethod[]

			paymentMethodsCache.value['US'] = usMethods
		} catch (error) {
			console.error('Failed to fetch US payment methods:', error)
			return
		}
	}

	availableMethods.value = usMethods

	const directPaypal = usMethods.find((m) => m.type === 'paypal')

	if (directPaypal) {
		withdrawData.value.selection.provider = 'paypal'
		withdrawData.value.selection.method = directPaypal.id
		withdrawData.value.selection.methodId = directPaypal.id

		withdrawData.value.providerData = {
			type: 'paypal',
		}

		await setStage('paypal-details', true)
	} else {
		console.error('An error occured - no paypal in US region??')
	}
}

const messages = defineMessages({
	unverifiedEmailHeader: {
		id: 'dashboard.creator-withdraw-modal.tremendous-details.unverified-email-header',
		defaultMessage: 'Unverified email',
	},
	unverifiedEmailMessage: {
		id: 'dashboard.creator-withdraw-modal.tremendous-details.unverified-email-message',
		defaultMessage:
			'The delivery email you have entered is not associated with your Modrinth account. Modrinth cannot recover rewards sent to an incorrect email address.',
	},
	paymentMethod: {
		id: 'dashboard.creator-withdraw-modal.tremendous-details.payment-method',
		defaultMessage: 'Payment method',
	},
	reward: {
		id: 'dashboard.creator-withdraw-modal.tremendous-details.reward',
		defaultMessage: 'Reward',
	},
	rewardPlaceholder: {
		id: 'dashboard.creator-withdraw-modal.tremendous-details.reward-placeholder',
		defaultMessage: 'Select reward',
	},
	rewardPlural: {
		id: 'dashboard.creator-withdraw-modal.tremendous-details.reward-plural',
		defaultMessage: 'Rewards',
	},
	usdPaypalWarningHeader: {
		id: 'dashboard.creator-withdraw-modal.tremendous-details.usd-paypal-warning-header',
		defaultMessage: 'Lower fees available',
	},
	usdPaypalWarningMessage: {
		id: 'dashboard.creator-withdraw-modal.tremendous-details.usd-paypal-warning-message',
		defaultMessage:
			'You selected USD for PayPal International. <direct-paypal-link>Switch to direct PayPal</direct-paypal-link> for better fees (≈2% instead of ≈6%).',
	},
})
</script>
