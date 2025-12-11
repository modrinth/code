<template>
	<div class="flex flex-col items-center gap-4">
		<div class="flex w-full items-center justify-center gap-2.5">
			<span class="text-xl font-semibold text-contrast sm:text-2xl">
				{{ formatMessage(messages.title) }}
			</span>
		</div>
		<div class="flex w-full flex-col gap-3">
			<div class="span-4 flex w-full flex-col gap-2.5 rounded-2xl bg-surface-2 p-4">
				<div
					class="flex w-full flex-col gap-1 sm:flex-row sm:items-center sm:justify-between sm:gap-0"
				>
					<span class="text-sm font-normal text-primary sm:text-[1rem]">
						{{ formatMessage(messages.method) }}
					</span>
					<span class="break-words text-sm font-semibold text-contrast sm:text-[1rem]">
						{{ result?.methodType || 'N/A' }}
					</span>
				</div>
				<div
					class="flex w-full flex-col gap-1 sm:flex-row sm:items-center sm:justify-between sm:gap-0"
				>
					<span class="text-sm font-normal text-primary sm:text-[1rem]">
						{{ formatMessage(messages.recipient) }}
					</span>
					<span class="break-words text-sm font-semibold text-contrast sm:text-[1rem]">
						{{ result?.recipientDisplay || 'N/A' }}
					</span>
				</div>
				<div
					v-if="destinationLabel && destinationValue"
					class="flex w-full flex-col gap-1 sm:flex-row sm:items-center sm:justify-between sm:gap-0"
				>
					<span class="text-sm font-normal text-primary sm:text-[1rem]">
						{{ destinationLabel }}
					</span>
					<span class="break-words font-mono text-sm font-semibold text-contrast sm:text-[1rem]">
						{{ destinationValue }}
					</span>
				</div>
				<div
					class="flex w-full flex-col gap-1 sm:flex-row sm:items-center sm:justify-between sm:gap-0"
				>
					<span class="text-sm font-normal text-primary sm:text-[1rem]">
						{{ formatMessage(messages.date) }}
					</span>
					<span class="break-words text-sm font-semibold text-contrast sm:text-[1rem]">
						{{ formattedDate }}
					</span>
				</div>
				<div
					class="flex w-full flex-col gap-1 sm:flex-row sm:items-center sm:justify-between sm:gap-0"
				>
					<span class="text-sm font-normal text-primary sm:text-[1rem]">
						{{ formatMessage(messages.amount) }}
					</span>
					<span class="break-words text-sm font-semibold text-contrast sm:text-[1rem]">
						{{ formatMoney(result?.amount || 0) }}
					</span>
				</div>
				<div
					class="flex w-full flex-col gap-1 sm:flex-row sm:items-center sm:justify-between sm:gap-0"
				>
					<span class="text-sm font-normal text-primary sm:text-[1rem]">
						{{ formatMessage(messages.fee) }}
					</span>
					<span class="break-words text-sm font-semibold text-contrast sm:text-[1rem]">
						{{ formatMoney(result?.fee || 0) }}
					</span>
				</div>
				<div class="border-b-1 h-0 w-full rounded-full border-b border-solid border-divider" />
				<div
					class="flex w-full flex-col gap-1 sm:flex-row sm:items-center sm:justify-between sm:gap-0"
				>
					<span class="text-sm font-normal text-primary sm:text-[1rem]">
						{{ formatMessage(messages.netAmount) }}
					</span>
					<span class="break-words text-sm font-semibold text-contrast sm:text-[1rem]">
						{{ formatMoney(result?.netAmount || 0) }}
						<template v-if="shouldShowExchangeRate">
							<span> ({{ formattedLocalCurrency }})</span>
						</template>
					</span>
				</div>
				<template v-if="shouldShowExchangeRate">
					<div
						class="flex w-full flex-col gap-1 sm:flex-row sm:items-center sm:justify-between sm:gap-0"
					>
						<span class="text-sm font-normal text-primary sm:text-[1rem]">
							{{ formatMessage(messages.exchangeRate) }}
						</span>
						<span class="break-words text-sm font-normal text-secondary sm:text-[1rem]">
							1 USD = {{ withdrawData.calculation.exchangeRate?.toFixed(4) }}
							{{ localCurrency }}
						</span>
					</div>
				</template>
			</div>
			<span
				v-if="withdrawData.providerData.type === 'tremendous'"
				class="w-full break-words text-center text-sm font-normal text-primary sm:text-[1rem]"
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
		</div>
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
import { normalizeChildren } from '@modrinth/ui'
import { formatMoney } from '@modrinth/utils'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { IntlFormatted } from '@vintl/vintl/components'
import dayjs from 'dayjs'
import { computed, onMounted, ref } from 'vue'
import ConfettiExplosion from 'vue-confetti-explosion'

import { type TremendousProviderData, useWithdrawContext } from '@/providers/creator-withdraw.ts'
import { getRailConfig } from '@/utils/muralpay-rails'

const { withdrawData } = useWithdrawContext()
const { formatMessage } = useVIntl()

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
	// Check if it's Tremendous withdrawal with currency
	if (withdrawData.value.providerData.type === 'tremendous') {
		return (withdrawData.value.providerData as TremendousProviderData).currency
	}

	// Fall back to MuralPay rail currency
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

const isMuralPayWithdrawal = computed(() => {
	return withdrawData.value.providerData.type === 'muralpay'
})

const destinationLabel = computed(() => {
	if (!isMuralPayWithdrawal.value) return null

	const rail = selectedRail.value
	if (!rail) return null

	if (rail.type === 'crypto') {
		return formatMessage(messages.wallet)
	} else if (rail.type === 'fiat') {
		return formatMessage(messages.account)
	}
	return null
})

const destinationValue = computed(() => {
	if (!isMuralPayWithdrawal.value || withdrawData.value.providerData.type !== 'muralpay') {
		return null
	}

	const accountDetails = withdrawData.value.providerData.accountDetails
	const rail = selectedRail.value

	if (rail?.type === 'crypto' && accountDetails.walletAddress) {
		const addr = accountDetails.walletAddress
		if (addr.length > 10) {
			return `${addr.slice(0, 6)}...${addr.slice(-4)}`
		}
		return addr
	} else if (rail?.type === 'fiat' && accountDetails.bankAccountNumber) {
		const accountType = accountDetails.accountType || 'Account'
		const last4 = accountDetails.bankAccountNumber.slice(-4)

		const formattedType = accountType.charAt(0) + accountType.slice(1).toLowerCase()

		return `${formattedType} (${last4})`
	}

	return null
})

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
	wallet: {
		id: 'dashboard.withdraw.completion.wallet',
		defaultMessage: 'Wallet',
	},
	account: {
		id: 'dashboard.withdraw.completion.account',
		defaultMessage: 'Account',
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
			"You'll receive an email at <b>{email}</b> with instructions to redeem your withdrawal.",
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
