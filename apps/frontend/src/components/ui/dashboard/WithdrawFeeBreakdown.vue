<template>
	<Transition
		enter-active-class="transition-all duration-300 ease-out"
		enter-from-class="opacity-0 max-h-0"
		enter-to-class="opacity-100 max-h-40"
		leave-active-class="transition-all duration-200 ease-in"
		leave-from-class="opacity-100 max-h-40"
		leave-to-class="opacity-0 max-h-0"
	>
		<div v-if="amount > 0" class="flex flex-col gap-2.5 rounded-[20px] bg-surface-2 p-4">
			<template v-if="isGiftCard && shouldShowExchangeRate">
				<div class="flex items-center justify-between">
					<span class="text-primary">{{ formatMessage(messages.feeBreakdownGiftCardValue) }}</span>
					<span class="font-semibold text-contrast"
						>{{ formatMoney(amount || 0) }} ({{ formattedLocalCurrency }})</span
					>
				</div>
			</template>
			<template v-else>
				<div class="flex items-center justify-between">
					<span class="text-primary">{{ formatMessage(messages.feeBreakdownAmount) }}</span>
					<span class="font-semibold text-contrast">{{ formatMoney(amount || 0) }}</span>
				</div>
			</template>

			<div class="flex items-center justify-between">
				<span class="text-primary">{{ formatMessage(messages.feeBreakdownFee) }}</span>
				<span class="h-4 font-semibold text-contrast">
					<template v-if="feeLoading">
						<LoaderCircleIcon class="size-5 animate-spin !text-secondary" />
					</template>
					<template v-else>-{{ formatMoney(fee || 0) }}</template>
				</span>
			</div>

			<div class="h-px bg-surface-5" />
			<div class="flex items-center justify-between">
				<span class="text-primary">{{ formatMessage(messages.feeBreakdownNetAmount) }}</span>
				<span class="font-semibold text-contrast">
					{{ formatMoney(netAmount) }}
					<template v-if="shouldShowExchangeRate">
						<span> ({{ formattedLocalCurrency }})</span>
					</template>
				</span>
			</div>
			<template v-if="shouldShowExchangeRate && !isGiftCard">
				<div class="flex items-center justify-between text-sm">
					<span class="text-primary">{{ formatMessage(messages.feeBreakdownExchangeRate) }}</span>
					<span class="text-secondary"
						>1 USD = {{ exchangeRate?.toFixed(4) }} {{ localCurrency }}</span
					>
				</div>
			</template>
		</div>
	</Transition>
</template>

<script setup lang="ts">
import { LoaderCircleIcon } from '@modrinth/assets'
import { formatMoney } from '@modrinth/utils'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { computed } from 'vue'

const props = withDefaults(
	defineProps<{
		amount: number
		fee: number | null
		feeLoading: boolean
		exchangeRate?: number | null
		localCurrency?: string
		isGiftCard?: boolean
	}>(),
	{
		exchangeRate: null,
		localCurrency: undefined,
		isGiftCard: false,
	},
)

const { formatMessage } = useVIntl()

const netAmount = computed(() => {
	const amount = props.amount || 0
	const fee = props.fee || 0
	return Math.max(0, amount - fee)
})

const shouldShowExchangeRate = computed(() => {
	if (!props.localCurrency) return false
	if (props.localCurrency === 'USD') return false
	return props.exchangeRate !== null && props.exchangeRate !== undefined && props.exchangeRate > 0
})

const netAmountInLocalCurrency = computed(() => {
	if (!shouldShowExchangeRate.value) return null
	return netAmount.value * (props.exchangeRate || 0)
})

const formattedLocalCurrency = computed(() => {
	if (!shouldShowExchangeRate.value || !netAmountInLocalCurrency.value || !props.localCurrency)
		return ''

	try {
		return new Intl.NumberFormat('en-US', {
			style: 'currency',
			currency: props.localCurrency,
			minimumFractionDigits: 2,
			maximumFractionDigits: 2,
		}).format(netAmountInLocalCurrency.value)
	} catch {
		return `${props.localCurrency} ${netAmountInLocalCurrency.value.toFixed(2)}`
	}
})

const messages = defineMessages({
	feeBreakdownAmount: {
		id: 'dashboard.creator-withdraw-modal.fee-breakdown-amount',
		defaultMessage: 'Amount',
	},
	feeBreakdownFee: {
		id: 'dashboard.creator-withdraw-modal.fee-breakdown-fee',
		defaultMessage: 'Fee',
	},
	feeBreakdownNetAmount: {
		id: 'dashboard.creator-withdraw-modal.fee-breakdown-net-amount',
		defaultMessage: 'Net amount',
	},
	feeBreakdownExchangeRate: {
		id: 'dashboard.creator-withdraw-modal.fee-breakdown-exchange-rate',
		defaultMessage: 'FX rate',
	},
	feeBreakdownGiftCardValue: {
		id: 'dashboard.creator-withdraw-modal.fee-breakdown-gift-card-value',
		defaultMessage: 'Gift card value',
	},
	feeBreakdownUsdEquivalent: {
		id: 'dashboard.creator-withdraw-modal.fee-breakdown-usd-equivalent',
		defaultMessage: 'USD equivalent',
	},
})
</script>
