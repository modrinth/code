<template>
	<div class="flex flex-col gap-6">
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

		<div class="flex flex-col gap-2.5">
			<label>
				<span class="text-md font-semibold text-contrast"
					>{{ formatMessage(messages.email) }} <span class="text-red">*</span></span
				>
			</label>
			<input
				v-model="deliveryEmail"
				type="email"
				:placeholder="formatMessage(messages.emailPlaceholder)"
				class="w-full rounded-[14px] bg-surface-4 px-4 py-2.5 text-contrast placeholder:text-secondary"
			/>
		</div>

		<div v-if="showGiftCardSelector" class="flex flex-col gap-2.5">
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

		<div class="flex flex-col gap-2.5">
			<label>
				<span class="text-md font-semibold text-contrast"
					>{{ formatMessage(messages.amount) }} <span class="text-red">*</span></span
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
				<div class="flex items-center gap-2">
					<div class="relative flex-1">
						<input
							v-model.number="formData.amount"
							type="number"
							step="0.01"
							:min="selectedMethodDetails?.interval?.standard?.min || 0.01"
							:max="roundedMaxAmount"
							:placeholder="formatMessage(messages.amountPlaceholder)"
							class="w-full rounded-[14px] bg-surface-4 py-2.5 pl-4 pr-4 text-contrast placeholder:text-secondary"
							@input="enforceDecimalPlaces"
						/>
					</div>
					<ButtonStyled>
						<button class="px-4 py-2" @click="setMaxAmount">
							{{ formatMessage(messages.maxButton) }}
						</button>
					</ButtonStyled>
				</div>
				<span class="text-secondary">
					{{ formatMoney(roundedMaxAmount) }} {{ formatMessage(messages.available) }}
				</span>
			</div>

			<WithdrawFeeBreakdown
				:amount="formData.amount || 0"
				:fee="calculatedFee"
				:fee-loading="feeLoading"
			/>

			<Checkbox v-model="agreedTerms" class="rewards-checkbox">
				<span
					>I agree to the
					<nuxt-link to="/legal/cmp" class="text-link">Rewards Program Terms</nuxt-link></span
				>
			</Checkbox>
		</div>
	</div>
</template>

<script setup lang="ts">
import { Admonition, ButtonStyled, Checkbox, Chips, Combobox, useDebugLogger } from '@modrinth/ui'
import { formatMoney } from '@modrinth/utils'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { useDebounceFn } from '@vueuse/core'
import { computed, onMounted, ref, watch } from 'vue'

import WithdrawFeeBreakdown from '@/components/ui/dashboard/WithdrawFeeBreakdown.vue'
import { useAuth } from '@/composables/auth.js'
import { useWithdrawContext } from '@/providers/creator-withdraw.ts'

const debug = useDebugLogger('TremendousDetailsStage')
const withdrawContext = useWithdrawContext()
const { formatMessage } = useVIntl()
const auth = await useAuth()

const userEmail = computed(() => {
	return (auth.value.user as any)?.email || ''
})

const deliveryEmail = ref<string>(
	withdrawContext.withdrawData.value.deliveryEmail || userEmail.value || '',
)

const showGiftCardSelector = computed(() => {
	const method = withdrawContext.withdrawData.value.selectedMethod
	return method === 'merchant_card' || method === 'charity'
})

const categoryLabel = computed(() => {
	const method = withdrawContext.withdrawData.value.selectedMethod
	switch (method) {
		case 'visa_card':
			return 'Virtual Visa'
		case 'merchant_card':
			return 'Gift card'
		case 'charity':
			return 'Charity'
		default:
			return 'Reward'
	}
})

const categoryLabelPlural = computed(() => {
	const method = withdrawContext.withdrawData.value.selectedMethod
	switch (method) {
		case 'visa_card':
			return 'Virtual Visas'
		case 'merchant_card':
			return 'Gift cards'
		case 'charity':
			return 'Charities'
		default:
			return 'Rewards'
	}
})

const isUnverifiedEmail = computed(() => {
	if (!deliveryEmail.value || !userEmail.value) return false
	return deliveryEmail.value.toLowerCase() !== userEmail.value.toLowerCase()
})

const maxAmount = computed(() => withdrawContext.maxWithdrawAmount.value)
const roundedMaxAmount = computed(() => Math.floor(maxAmount.value * 100) / 100)

const formData = ref<Record<string, any>>({
	amount: withdrawContext.withdrawData.value.amount || undefined,
})

const selectedGiftCardId = ref<string | null>(
	withdrawContext.withdrawData.value.selectedMethodId || null,
)

const agreedTerms = ref<boolean>(false)

const calculatedFee = ref<number>(0)
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

const selectedDenomination = computed({
	get: () => formData.value.amount,
	set: (value) => {
		formData.value.amount = value
	},
})

function setMaxAmount() {
	formData.value.amount = roundedMaxAmount.value
}

function enforceDecimalPlaces(event: Event) {
	const input = event.target as HTMLInputElement
	const value = input.value

	if (value && value.includes('.')) {
		const parts = value.split('.')
		if (parts[1] && parts[1].length > 2) {
			const rounded = Math.floor(parseFloat(value) * 100) / 100
			formData.value.amount = rounded
			input.value = rounded.toString()
		}
	}
}

const calculateFees = useDebounceFn(async () => {
	const amount = formData.value.amount
	if (!amount || amount <= 0) {
		calculatedFee.value = 0
		return
	}

	const methodId = showGiftCardSelector.value
		? selectedGiftCardId.value
		: withdrawContext.withdrawData.value.selectedMethodId

	if (!methodId) {
		calculatedFee.value = 0
		return
	}

	feeLoading.value = true
	try {
		// mostly useless apart from PayPal/Venmo
		const response = (await useBaseFetch('payout/fees', {
			apiVersion: 3,
			method: 'POST',
			body: {
				amount,
				method: 'tremendous',
				method_id: methodId,
			},
		})) as { fee: number | null; exchange_rate: number | null }

		calculatedFee.value = response.fee || 0
	} catch (error) {
		console.error('Failed to calculate fees:', error)
		calculatedFee.value = 0
	} finally {
		feeLoading.value = false
	}
}, 500)

watch(
	() => formData.value.amount,
	(newAmount) => {
		if (newAmount !== undefined && newAmount !== null) {
			if (newAmount > roundedMaxAmount.value) {
				formData.value.amount = roundedMaxAmount.value
				return
			}
			if (newAmount < 0) {
				formData.value.amount = 0
				return
			}
		}
	},
)

watch(deliveryEmail, (newEmail) => {
	withdrawContext.withdrawData.value.deliveryEmail = newEmail
})

watch(
	[() => formData.value.amount, selectedGiftCardId],
	() => {
		withdrawContext.withdrawData.value.amount = formData.value.amount ?? 0

		if (showGiftCardSelector.value && selectedGiftCardId.value) {
			withdrawContext.withdrawData.value.selectedMethodId = selectedGiftCardId.value
		}

		if (formData.value.amount) {
			calculateFees()
		}
	},
	{ deep: true },
)

onMounted(async () => {
	const country = withdrawContext.withdrawData.value.selectedCountry
	if (!country) return

	debug('Fetching payout methods for country:', country.id)

	try {
		// todo: when we do @modrinth/api deduplicate types across all these stages.
		const methods = (await useBaseFetch('payout/methods', {
			apiVersion: 3,
			query: { country: country.id },
		})) as Array<{
			id: string
			type: string
			name: string
			category?: string
			image_url: string | null
			image_logo_url: string | null
			interval?: {
				fixed?: {
					values: number[]
				}
				standard?: {
					min: number
					max: number
				}
			}
		}>

		const selectedMethod = withdrawContext.withdrawData.value.selectedMethod

		rewardOptions.value = methods
			.filter((m) => m.type === 'tremendous')
			.filter((m) => m.category === selectedMethod)
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
	} catch (error) {
		console.error('Failed to fetch gift card options:', error)
	}

	if (formData.value.amount) {
		calculateFees()
	}
})

watch(
	() => withdrawContext.withdrawData.value.selectedMethod,
	(newMethod, oldMethod) => {
		if (oldMethod && newMethod !== oldMethod) {
			formData.value = {
				amount: undefined,
			}
			selectedGiftCardId.value = null
			calculatedFee.value = 0
		}
	},
)

const messages = defineMessages({
	email: {
		id: 'dashboard.creator-withdraw-modal.tremendous-details.email',
		defaultMessage: 'Email',
	},
	emailPlaceholder: {
		id: 'dashboard.creator-withdraw-modal.tremendous-details.email-placeholder',
		defaultMessage: 'Enter email address',
	},
	unverifiedEmailHeader: {
		id: 'dashboard.creator-withdraw-modal.tremendous-details.unverified-email-header',
		defaultMessage: 'Unverified email',
	},
	unverifiedEmailMessage: {
		id: 'dashboard.creator-withdraw-modal.tremendous-details.unverified-email-message',
		defaultMessage:
			'The delivery email you have entered is not associated with your Modrinth account. Modrinth cannot recover rewards sent to an incorrect email address.',
	},
	reward: {
		id: 'dashboard.creator-withdraw-modal.tremendous-details.reward',
		defaultMessage: 'Reward',
	},
	rewardPlaceholder: {
		id: 'dashboard.creator-withdraw-modal.tremendous-details.reward-placeholder',
		defaultMessage: 'Select reward',
	},
	amount: {
		id: 'dashboard.creator-withdraw-modal.tremendous-details.amount',
		defaultMessage: 'Amount',
	},
	amountPlaceholder: {
		id: 'dashboard.creator-withdraw-modal.tremendous-details.amount-placeholder',
		defaultMessage: 'Enter amount',
	},
	maxButton: {
		id: 'dashboard.creator-withdraw-modal.tremendous-details.max-button',
		defaultMessage: 'Max',
	},
	available: {
		id: 'dashboard.creator-withdraw-modal.tremendous-details.available',
		defaultMessage: 'available.',
	},
})
</script>
