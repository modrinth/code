<template>
	<div class="flex flex-col items-center gap-6">
		<div class="flex w-full items-center justify-center gap-2.5">
			<span class="text-nowrap text-2xl font-semibold text-contrast">
				{{ formatMessage(messages.title) }}
			</span>
		</div>
		<div class="flex w-full flex-col gap-3">
			<div class="span-4 flex w-full flex-col gap-2.5 rounded-2xl bg-surface-2 p-4">
				<div class="flex w-full items-center justify-between">
					<span class="text-nowrap text-base font-normal text-primary">
						{{ formatMessage(messages.method) }}
					</span>
					<span class="text-nowrap text-base font-semibold text-contrast">
						{{ result?.methodType || 'N/A' }}
					</span>
				</div>
				<div class="flex w-full items-center justify-between">
					<span class="text-nowrap text-base font-normal text-primary">
						{{ formatMessage(messages.recipient) }}
					</span>
					<span class="text-nowrap text-base font-semibold text-contrast">
						{{ result?.recipientDisplay || 'N/A' }}
					</span>
				</div>
				<div class="flex w-full items-center justify-between">
					<span class="text-nowrap text-base font-normal text-primary">
						{{ formatMessage(messages.date) }}
					</span>
					<span class="text-nowrap text-base font-semibold text-contrast">
						{{ formattedDate }}
					</span>
				</div>
				<div class="flex w-full items-center justify-between">
					<span class="text-nowrap text-base font-normal text-primary">
						{{ formatMessage(messages.amount) }}
					</span>
					<span class="text-nowrap text-base font-semibold text-contrast">
						{{ formatMoney(result?.amount || 0) }}
					</span>
				</div>
				<div class="flex w-full items-center justify-between">
					<span class="text-nowrap text-base font-normal text-primary">
						{{ formatMessage(messages.fee) }}
					</span>
					<span class="text-nowrap text-base font-semibold text-contrast">
						{{ formatMoney(result?.fee || 0) }}
					</span>
				</div>
				<div class="border-b-1 h-0 w-full rounded-full border-b border-solid border-divider" />
				<div class="flex w-full items-center justify-between">
					<span class="text-nowrap text-base font-normal text-primary">
						{{ formatMessage(messages.netAmount) }}
					</span>
					<span class="text-nowrap text-base font-semibold text-contrast">
						{{ formatMoney(result?.netAmount || 0) }}
						<template v-if="shouldShowExchangeRate">
							<span class="text-secondary"> ({{ formattedLocalCurrency }})</span>
						</template>
					</span>
				</div>
				<template v-if="shouldShowExchangeRate">
					<div class="border-b-1 h-0 w-full rounded-full border-b border-solid border-divider" />
					<div class="flex w-full items-center justify-between">
						<span class="text-nowrap text-base font-normal text-primary">
							{{ formatMessage(messages.exchangeRate) }}
						</span>
						<span class="text-nowrap text-base font-normal text-secondary">
							1 USD = {{ withdrawData.calculation.exchangeRate?.toFixed(4) }}
							{{ localCurrency }}
						</span>
					</div>
				</template>
			</div>
		</div>
		<span
			v-if="withdrawData.providerData.type === 'tremendous'"
			class="w-full text-center text-base font-normal text-primary"
		>
			<IntlFormatted
				:message-id="messages.emailConfirmation"
				:values="{ email: withdrawData.result?.recipientDisplay }"
			>
				<template #b="{ children }">
					<strong>
						<component :is="() => normalizeChildren(children)" />
					</strong>
				</template>
			</IntlFormatted>
		</span>
		<Teleport to="body">
			<div
				v-if="showConfetti"
				class="pointer-events-none fixed inset-0 z-[9999] flex items-center justify-center"
			>
				<ConfettiExplosion />
			</div>
		</Teleport>
	</div>
</template>

<script setup lang="ts">
import { ButtonStyled } from '@modrinth/ui'
import { formatMoney } from '@modrinth/utils'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { IntlFormatted } from '@vintl/vintl/components'
import dayjs from 'dayjs'
import { computed, onMounted, ref } from 'vue'
import ConfettiExplosion from 'vue-confetti-explosion'

import { useWithdrawContext } from '@/providers/creator-withdraw.ts'
import { getRailConfig } from '@/utils/muralpay-rails'
import { normalizeChildren } from '@/utils/vue-children.ts'

const { withdrawData } = useWithdrawContext()
const { formatMessage } = useVIntl()

const emit = defineEmits<{
	(e: 'close' | 'view-transactions'): void
}>()

const result = computed(() => withdrawData.value.result)

const showConfetti = ref(false)

onMounted(() => {
	showConfetti.value = true
	setTimeout(() => {
		showConfetti.value = false
	}, 5000)
})

const formattedDate = computed(() => {
	if (!result.value?.created) return 'N/A'
	return dayjs(result.value.created).format('MMMM D, YYYY')
})

const selectedRail = computed(() => {
	const railId = withdrawData.value.selection.method
	return railId ? getRailConfig(railId) : null
})

const localCurrency = computed(() => {
	return selectedRail.value?.currency
})

const shouldShowExchangeRate = computed(() => {
	if (!localCurrency.value) return false
	if (localCurrency.value === 'USD') return false
	const exchangeRate = withdrawData.value.calculation.exchangeRate
	return exchangeRate !== null && exchangeRate !== undefined && exchangeRate > 0
})

const netAmountInLocalCurrency = computed(() => {
	if (!shouldShowExchangeRate.value) return null
	const netAmount = result.value?.netAmount || 0
	const exchangeRate = withdrawData.value.calculation.exchangeRate || 0
	return netAmount * exchangeRate
})

const formattedLocalCurrency = computed(() => {
	if (!shouldShowExchangeRate.value || !netAmountInLocalCurrency.value || !localCurrency.value)
		return ''

	try {
		return new Intl.NumberFormat('en-US', {
			style: 'currency',
			currency: localCurrency.value,
			minimumFractionDigits: 2,
			maximumFractionDigits: 2,
		}).format(netAmountInLocalCurrency.value)
	} catch {
		return `${localCurrency.value} ${netAmountInLocalCurrency.value.toFixed(2)}`
	}
})

function handleClose() {
	emit('close')
}

function handleViewTransactions() {
	emit('view-transactions')
}

const messages = defineMessages({
	title: {
		id: 'dashboard.withdraw.completion.title',
		defaultMessage: 'Withdraw complete',
	},
	method: {
		id: 'dashboard.withdraw.completion.method',
		defaultMessage: 'Method',
	},
	recipient: {
		id: 'dashboard.withdraw.completion.recipient',
		defaultMessage: 'Recipient',
	},
	date: {
		id: 'dashboard.withdraw.completion.date',
		defaultMessage: 'Date',
	},
	amount: {
		id: 'dashboard.withdraw.completion.amount',
		defaultMessage: 'Amount',
	},
	fee: {
		id: 'dashboard.withdraw.completion.fee',
		defaultMessage: 'Fee',
	},
	netAmount: {
		id: 'dashboard.withdraw.completion.net-amount',
		defaultMessage: 'Net amount',
	},
	exchangeRate: {
		id: 'dashboard.withdraw.completion.exchange-rate',
		defaultMessage: 'Exchange rate',
	},
	emailConfirmation: {
		id: 'dashboard.withdraw.completion.email-confirmation',
		defaultMessage:
			'You will receive an email at <b>{email}</b> from our partner, Tremendous, with instructions to redeem your withdrawal.',
	},
	closeButton: {
		id: 'dashboard.withdraw.completion.close-button',
		defaultMessage: 'Close',
	},
	transactionsButton: {
		id: 'dashboard.withdraw.completion.transactions-button',
		defaultMessage: 'Transactions',
	},
})
</script>
