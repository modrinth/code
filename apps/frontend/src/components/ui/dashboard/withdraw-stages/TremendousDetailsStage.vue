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
				{{ formatMoney(fixedDenominationMin ?? effectiveMinAmount)
				}}<template v-if="selectedMethodCurrencyCode && selectedMethodCurrencyCode !== 'USD'">
					({{
						formatAmountForDisplay(
							fixedDenominationMin ?? effectiveMinAmount,
							selectedMethodCurrencyCode,
							selectedMethodExchangeRate,
						)
					}})</template
				>
				min,
				{{
					formatMoney(
						fixedDenominationMax ??
							selectedMethodDetails.interval?.standard?.max ??
							effectiveMaxAmount,
					)
				}}<template v-if="selectedMethodCurrencyCode && selectedMethodCurrencyCode !== 'USD'">
					({{
						formatAmountForDisplay(
							fixedDenominationMax ??
								selectedMethodDetails.interval?.standard?.max ??
								effectiveMaxAmount,
							selectedMethodCurrencyCode,
							selectedMethodExchangeRate,
						)
					}})</template
				>
				max withdrawal amount.
			</span>
			<span
				v-if="selectedMethodDetails && effectiveMinAmount > roundedMaxAmount"
				class="text-sm text-red"
			>
				You need at least {{ formatMoney(effectiveMinAmount)
				}}<template v-if="selectedMethodCurrencyCode && selectedMethodCurrencyCode !== 'USD'">
					({{
						formatAmountForDisplay(
							effectiveMinAmount,
							selectedMethodCurrencyCode,
							selectedMethodExchangeRate,
						)
					}})</template
				>
				to use this gift card.
			</span>
		</div>

		<div class="flex flex-col gap-2.5">
			<label>
				<span class="text-md font-semibold text-contrast">
					<template v-if="useDenominationSuggestions">
						{{ formatMessage(messages.searchAmountLabel) }} ({{ selectedMethodCurrencyCode }})
					</template>
					<template v-else>
						{{ formatMessage(formFieldLabels.amount) }}
					</template>
					<span class="text-red">*</span>
				</span>
			</label>

			<div v-if="showGiftCardSelector && useFixedDenominations" class="flex flex-col gap-2.5">
				<template v-if="useDenominationSuggestions">
					<div class="iconified-input w-full">
						<SearchIcon aria-hidden="true" />
						<input
							v-model.number="denominationSearchInput"
							type="number"
							step="0.01"
							:min="0"
							:disabled="effectiveMinAmount > roundedMaxAmount"
							:placeholder="formatMessage(messages.enterDenominationPlaceholder)"
							class="!bg-surface-4"
							@input="hasTouchedSuggestions = true"
						/>
					</div>
					<Transition
						enter-active-class="transition-opacity duration-200 ease-out"
						enter-from-class="opacity-0"
						enter-to-class="opacity-100"
						leave-active-class="transition-opacity duration-150 ease-in"
						leave-from-class="opacity-100"
						leave-to-class="opacity-0"
					>
						<span
							v-if="
								selectedMethodCurrencyCode &&
								selectedMethodCurrencyCode !== 'USD' &&
								selectedMethodExchangeRate
							"
							class="text-sm text-secondary"
						>
							{{
								formatMessage(messages.balanceWorthHint, {
									usdBalance: formatMoney(roundedMaxAmount),
									localBalance: formatAmountForDisplay(
										roundedMaxAmount,
										selectedMethodCurrencyCode,
										selectedMethodExchangeRate,
									),
								})
							}}
						</span>
					</Transition>
				</template>

				<Transition
					enter-active-class="transition-all duration-300 ease-out"
					enter-from-class="opacity-0 max-h-0"
					enter-to-class="opacity-100 max-h-96"
					leave-active-class="transition-all duration-200 ease-in"
					leave-from-class="opacity-100 max-h-96"
					leave-to-class="opacity-0 max-h-0"
				>
					<div
						v-if="
							!useDenominationSuggestions ||
							(denominationSearchInput && displayedSuggestions.length > 0)
						"
						class="overflow-hidden pt-0"
					>
						<span
							v-if="useDenominationSuggestions"
							class="mb-1 block text-sm font-medium text-secondary"
						>
							{{ formatMessage(messages.availableDenominationsLabel) }}
						</span>
						<div class="p-[2px]">
							<Chips
								v-model="selectedDenomination"
								:items="useDenominationSuggestions ? displayedSuggestions : denominationOptions"
								:format-label="
									(amt: number) =>
										formatAmountForDisplay(
											amt,
											selectedMethodCurrencyCode,
											selectedMethodExchangeRate,
										)
								"
								:never-empty="false"
								:capitalize="false"
							/>
						</div>
						<span
							v-if="useDenominationSuggestions && hasTouchedSuggestions && !hasSelectedDenomination"
							class="mt-2.5 block text-sm text-orange"
						>
							{{ formatMessage(messages.selectDenominationRequired) }}
						</span>
						<span
							v-if="
								!useDenominationSuggestions &&
								selectedMethodCurrencyCode &&
								selectedMethodCurrencyCode !== 'USD' &&
								selectedMethodExchangeRate
							"
							class="mt-2 block text-sm text-secondary"
						>
							{{
								formatMessage(messages.balanceWorthHint, {
									usdBalance: formatMoney(roundedMaxAmount),
									localBalance: formatAmountForDisplay(
										roundedMaxAmount,
										selectedMethodCurrencyCode,
										selectedMethodExchangeRate,
									),
								})
							}}
						</span>
					</div>
				</Transition>

				<Transition
					enter-active-class="transition-opacity duration-200 ease-out"
					enter-from-class="opacity-0"
					enter-to-class="opacity-100"
					leave-active-class="transition-opacity duration-150 ease-in"
					leave-from-class="opacity-100"
					leave-to-class="opacity-0"
				>
					<span
						v-if="
							useDenominationSuggestions &&
							denominationSearchInput &&
							displayedSuggestions.length === 0
						"
						class="text-sm text-secondary"
					>
						{{ noSuggestionsMessage }}
					</span>
				</Transition>

				<span
					v-if="!useDenominationSuggestions && denominationOptions.length === 0"
					class="text-error text-sm"
				>
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
				v-if="allRequiredFieldsFilled && formData.amount && formData.amount > 0"
				:amount="formData.amount || 0"
				:fee="calculatedFee"
				:fee-loading="feeLoading"
				:exchange-rate="showGiftCardSelector ? selectedMethodExchangeRate : giftCardExchangeRate"
				:local-currency="
					showGiftCardSelector ? (selectedMethodCurrencyCode ?? undefined) : giftCardCurrencyCode
				"
				:is-gift-card="showGiftCardSelector"
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
import { SearchIcon } from '@modrinth/assets'
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

const denominationSearchInput = ref<number | undefined>(undefined)
const hasTouchedSuggestions = ref(false)

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
			currencyCode?: string | null
			exchangeRate?: number | null
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

const selectedMethodCurrencyCode = computed(() => selectedMethodDetails.value?.currencyCode || null)
const selectedMethodExchangeRate = computed(() => selectedMethodDetails.value?.exchangeRate || null)

const giftCardCurrencyCode = computed(() => {
	if (showPayPalCurrencySelector.value) {
		return selectedCurrency.value !== 'USD' ? selectedCurrency.value : undefined
	}

	if (
		showGiftCardSelector.value &&
		selectedMethodCurrencyCode.value &&
		selectedMethodCurrencyCode.value !== 'USD'
	) {
		return selectedMethodCurrencyCode.value
	}
	return undefined
})

const giftCardExchangeRate = computed(() => {
	if (showPayPalCurrencySelector.value) {
		return exchangeRate.value
	}

	if (
		showGiftCardSelector.value &&
		selectedMethodCurrencyCode.value &&
		selectedMethodCurrencyCode.value !== 'USD'
	) {
		return selectedMethodExchangeRate.value
	}
	return exchangeRate.value
})

function formatAmountForDisplay(
	usdAmount: number,
	currencyCode: string | null | undefined,
	rate: number | null | undefined,
): string {
	if (!currencyCode || currencyCode === 'USD' || !rate) {
		return formatMoney(usdAmount)
	}
	const localAmount = usdAmount * rate
	try {
		return new Intl.NumberFormat('en-US', {
			style: 'currency',
			currency: currencyCode,
			minimumFractionDigits: 2,
			maximumFractionDigits: 2,
		}).format(localAmount)
	} catch {
		return `${currencyCode} ${localAmount.toFixed(2)}`
	}
}

const useFixedDenominations = computed(() => {
	const interval = selectedMethodDetails.value?.interval
	if (!interval) return false

	if (interval.fixed?.values?.length) {
		debug('Use fixed denominations: true (has fixed values)')
		return true
	}

	// treat min=max as single fixed value
	if (interval.standard) {
		const { min, max } = interval.standard
		const isSingleValue = min === max
		debug('Use fixed denominations:', isSingleValue, '(min=max:', min, '=', max, ')')
		return isSingleValue
	}
	return false
})

const useDenominationSuggestions = computed(() => {
	if (!useFixedDenominations.value) return false
	const interval = selectedMethodDetails.value?.interval
	if (!interval?.fixed?.values) return false
	return interval.fixed.values.length > 10
})

const denominationSuggestions = computed(() => {
	const allDenominations = denominationOptions.value
	if (allDenominations.length === 0) return []

	const input = denominationSearchInput.value

	// When no search input, use the user's balance as the target
	const exchangeRate = selectedMethodExchangeRate.value
	const targetInUsd =
		input && input > 0 ? (exchangeRate ? input / exchangeRate : input) : roundedMaxAmount.value

	const rangeSize = targetInUsd * 0.2
	let lowerBound = targetInUsd - rangeSize / 2
	let upperBound = targetInUsd + rangeSize / 2

	const minAvailable = allDenominations[0]
	const maxAvailable = allDenominations[allDenominations.length - 1]

	// shift range when hitting boundaries to maintain ~20% total range
	if (upperBound > maxAvailable) {
		const overflow = upperBound - maxAvailable
		upperBound = maxAvailable
		lowerBound = Math.max(minAvailable, lowerBound - overflow)
	} else if (lowerBound < minAvailable) {
		const underflow = minAvailable - lowerBound
		lowerBound = minAvailable
		upperBound = Math.min(maxAvailable, upperBound + underflow)
	}

	return allDenominations
		.filter((amt) => amt >= lowerBound && amt <= upperBound)
		.sort((a, b) => a - b)
})

const maxDisplayedSuggestions = 10
const displayedSuggestions = computed(() => {
	const all = denominationSuggestions.value
	if (all.length <= maxDisplayedSuggestions) return all

	const input = denominationSearchInput.value
	const exchangeRate = selectedMethodExchangeRate.value

	// Use balance as target when no search input
	const targetInUsd =
		input && input > 0 ? (exchangeRate ? input / exchangeRate : input) : roundedMaxAmount.value

	// select values closest to target, then sort ascending for display
	const closest = [...all]
		.sort((a, b) => Math.abs(a - targetInUsd) - Math.abs(b - targetInUsd))
		.slice(0, maxDisplayedSuggestions)

	return closest.sort((a, b) => a - b)
})

const noSuggestionsMessage = computed(() => {
	if (!denominationSearchInput.value || denominationSearchInput.value <= 0) {
		return null
	}
	if (denominationSuggestions.value.length === 0) {
		const maxDenom = fixedDenominationMax.value
		if (maxDenom) {
			const maxInLocal = formatAmountForDisplay(
				maxDenom,
				selectedMethodCurrencyCode.value,
				selectedMethodExchangeRate.value,
			)
			return `No denominations near this amount. The highest available is ${maxInLocal}.`
		}
		return 'No denominations near this amount'
	}
	return null
})

const hasSelectedDenomination = computed(() => {
	return (
		formData.value.amount !== undefined &&
		formData.value.amount > 0 &&
		denominationOptions.value.includes(formData.value.amount)
	)
})

const denominationOptions = computed(() => {
	const interval = selectedMethodDetails.value?.interval
	if (!interval) return []

	let values: number[] = []

	if (interval.fixed?.values) {
		values = [...interval.fixed.values]
	} else if (interval.standard && interval.standard.min === interval.standard.max) {
		// min=max case: treat as single fixed value
		values = [interval.standard.min]
	}

	if (values.length === 0) return []

	const filtered = values.filter((amount) => amount <= roundedMaxAmount.value).sort((a, b) => a - b)
	debug(
		'Denomination options (filtered by max):',
		filtered,
		'from',
		values,
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

const fixedDenominationMin = computed(() => {
	if (!useFixedDenominations.value) return null
	const options = denominationOptions.value
	if (options.length === 0) return null
	return options[0]
})

const fixedDenominationMax = computed(() => {
	if (!useFixedDenominations.value) return null
	const options = denominationOptions.value
	if (options.length === 0) return null
	return options[options.length - 1]
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
				currencyCode: m.currency_code,
				exchangeRate: m.exchange_rate,
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
			denominationSearchInput.value = undefined
			hasTouchedSuggestions.value = false

			// Clear currency when switching away from PayPal International
			if (newMethod !== 'paypal' && withdrawData.value.providerData.type === 'tremendous') {
				;(withdrawData.value.providerData as any).currency = undefined
			}
		}
	},
)

watch(selectedGiftCardId, (newId, oldId) => {
	if (oldId && newId !== oldId) {
		// Reset state when gift card changes
		hasTouchedSuggestions.value = false
		formData.value.amount = undefined
		// denominationSearchInput will be prefilled by the watch below
		denominationSearchInput.value = undefined
	}
})

// Prefill denomination search with balance in local currency when suggestions mode is enabled
watch(
	[useDenominationSuggestions, selectedMethodExchangeRate],
	([showSuggestions, exchangeRate]) => {
		if (showSuggestions && denominationSearchInput.value === undefined) {
			const balanceInLocal = exchangeRate
				? roundedMaxAmount.value * exchangeRate
				: roundedMaxAmount.value
			denominationSearchInput.value = Math.floor(balanceInLocal * 100) / 100
			hasTouchedSuggestions.value = true
		}
	},
	{ immediate: true },
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
	enterDenominationPlaceholder: {
		id: 'dashboard.creator-withdraw-modal.tremendous-details.enter-denomination-placeholder',
		defaultMessage: 'Enter amount',
	},
	enterAmountHint: {
		id: 'dashboard.creator-withdraw-modal.tremendous-details.enter-amount-hint',
		defaultMessage: 'Find gift cards near this value.',
	},
	balanceWorthHint: {
		id: 'dashboard.creator-withdraw-modal.tremendous-details.balance-worth-hint',
		defaultMessage: 'Your balance of {usdBalance} is currently worth {localBalance}.',
	},
	searchAmountLabel: {
		id: 'dashboard.creator-withdraw-modal.tremendous-details.search-amount-label',
		defaultMessage: 'Search amount',
	},
	availableDenominationsLabel: {
		id: 'dashboard.creator-withdraw-modal.tremendous-details.available-denominations-label',
		defaultMessage: 'Available denominations',
	},
	selectDenominationHint: {
		id: 'dashboard.creator-withdraw-modal.tremendous-details.select-denomination-hint',
		defaultMessage: 'Select a denomination:',
	},
	selectDenominationRequired: {
		id: 'dashboard.creator-withdraw-modal.tremendous-details.select-denomination-required',
		defaultMessage: 'Please select a denomination to continue',
	},
})
</script>
