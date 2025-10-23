<template>
	<div class="mb-6 flex flex-col gap-4 p-12 py-0">
		<div class="flex flex-row items-center justify-between">
			<span class="text-2xl font-semibold text-contrast">{{
				formatMessage(messages.transactionsHeader)
			}}</span>
			<div class="flex w-[400px] flex-row items-center gap-2">
				<Combobox
					v-model="selectedYear"
					:options="yearOptions"
					:display-value="selectedYear === 'all' ? 'All years' : String(selectedYear)"
					listbox
				/>
				<Combobox
					v-model="selectedMethod"
					:options="methodOptions"
					:display-value="selectedMethod === 'all' ? 'All types' : formatTypeLabel(selectedMethod)"
					listbox
				/>
			</div>
		</div>

		<div v-if="Object.keys(groupedTransactions).length > 0" class="flex flex-col gap-6">
			<div
				v-for="(transactions, period) in groupedTransactions"
				:key="period"
				class="flex flex-col gap-4"
			>
				<h3 class="text-base font-medium leading-6 text-primary">{{ period }}</h3>
				<div class="flex flex-col gap-4">
					<RevenueTransaction
						v-for="transaction in transactions"
						:key="transaction.id || transaction.created"
						:transaction="transaction"
						@cancelled="refresh"
					/>
				</div>
			</div>
		</div>
		<div v-else class="mx-auto flex flex-col justify-center p-6 text-center">
			<span class="text-xl text-contrast">{{ formatMessage(messages.noTransactions) }}</span>
			<span class="max-w-[256px] text-lg text-secondary">{{
				formatMessage(messages.noTransactionsDesc)
			}}</span>
		</div>
	</div>
</template>
<script setup>
import { Combobox } from '@modrinth/ui'
import { capitalizeString } from '@modrinth/utils'
import { defineMessages, useVIntl } from '@vintl/vintl'
import dayjs from 'dayjs'

import RevenueTransaction from '~/components/ui/dashboard/RevenueTransaction.vue'

const { formatMessage } = useVIntl()

useHead({
	title: 'Transaction history - Modrinth',
})

const { data: transactions, refresh } = await useAsyncData(`payout-history`, () =>
	useBaseFetch(`payout/history`, {
		apiVersion: 3,
	}),
)

const allTransactions = computed(() => {
	if (!transactions.value) return []

	return transactions.value.map((txn) => ({
		...txn,
		type: txn.type || (txn.method_type || txn.method ? 'withdrawal' : 'payout_available'),
	}))
})

const sortedTransactions = computed(() =>
	[...allTransactions.value].sort((a, b) => dayjs(b.created).diff(dayjs(a.created))),
)

const yearOptions = computed(() => {
	const yearSet = new Set(sortedTransactions.value.map((x) => dayjs(x.created).year()))
	const yearValues = ['all', ...Array.from(yearSet).sort((a, b) => b - a)]

	return yearValues.map((year) => ({
		value: year,
		label: year === 'all' ? 'All years' : String(year),
	}))
})

const selectedYear = ref('all')

const methodOptions = computed(() => {
	const types = new Set()

	sortedTransactions.value.forEach((x) => {
		if (x.type === 'payout_available' && x.payout_source) {
			types.add(x.payout_source)
		} else if (x.type === 'withdrawal' && (x.method_type || x.method)) {
			types.add(x.method_type || x.method)
		}
	})

	const typeValues = ['all', ...Array.from(types)]

	return typeValues.map((type) => ({
		value: type,
		label: type === 'all' ? 'All types' : formatTypeLabel(type),
	}))
})

const selectedMethod = ref('all')

function formatMethodLabel(method) {
	switch (method) {
		case 'paypal':
			return 'PayPal'
		case 'venmo':
			return 'Venmo'
		case 'tremendous':
			return 'Tremendous'
		case 'muralpay':
			return 'Muralpay'
		default:
			return capitalizeString(method)
	}
}

function formatTypeLabel(type) {
	// Check if it's a payout method (withdrawal)
	const payoutMethods = ['paypal', 'venmo', 'tremendous', 'muralpay']
	if (payoutMethods.includes(type)) {
		return formatMethodLabel(type)
	}
	// Otherwise it's a payout_source (income), convert snake_case to Title Case
	return type
		.split('_')
		.map((word) => capitalizeString(word))
		.join(' ')
}

const filteredTransactions = computed(() =>
	sortedTransactions.value
		.filter((x) => selectedYear.value === 'all' || dayjs(x.created).year() === selectedYear.value)
		.filter((x) => {
			if (selectedMethod.value === 'all') return true
			// Check if it's an income source
			if (x.type === 'payout_available') {
				return x.payout_source === selectedMethod.value
			}
			// Check if it's a withdrawal method
			return x.type === 'withdrawal' && (x.method_type || x.method) === selectedMethod.value
		}),
)

function getPeriodLabel(date) {
	const txnDate = dayjs(date)
	const now = dayjs()

	if (txnDate.isSame(now, 'month')) {
		return 'This month'
	} else if (txnDate.isSame(now.subtract(1, 'month'), 'month')) {
		return 'Last month'
	} else {
		return txnDate.format('MMMM YYYY')
	}
}

const groupedTransactions = computed(() => {
	const groups = {}

	filteredTransactions.value.forEach((transaction) => {
		const period = getPeriodLabel(transaction.created)

		if (!groups[period]) {
			groups[period] = []
		}

		groups[period].push(transaction)
	})

	return groups
})

const messages = defineMessages({
	transactionsHeader: {
		id: 'dashboard.revenue.transactions.header',
		defaultMessage: 'Transactions',
	},
	noTransactions: {
		id: 'dashboard.revenue.transactions.none',
		defaultMessage: 'No transactions',
	},
	noTransactionsDesc: {
		id: 'dashboard.revenue.transactions.none.desc',
		defaultMessage: 'Your payouts and withdrawals will appear here.',
	},
})
</script>
<style scoped></style>
