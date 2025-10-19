<template>
	<div class="mb-6 flex flex-col gap-4 p-12 py-0">
		<div class="flex flex-row justify-between">
			<span class="text-2xl font-semibold text-contrast">{{
				formatMessage(messages.transactionsHeader)
			}}</span>
		</div>

		<div class="flex flex-wrap items-center gap-2">
			<DropdownSelect
				v-model="selectedYear"
				:options="years"
				:display-name="(x) => (x === 'all' ? 'All years' : x)"
				name="Year filter"
			/>
			<DropdownSelect
				v-model="selectedMethod"
				:options="methods"
				:display-name="
					(x) => (x === 'all' ? 'Any method' : x === 'paypal' ? 'PayPal' : capitalizeString(x))
				"
				name="Method filter"
			/>
		</div>

		<p class="text-secondary">
			{{
				selectedYear !== 'all'
					? selectedMethod !== 'all'
						? formatMessage(messages.transfersTotalYearMethod, {
								amount: $formatMoney(totalAmount),
								year: selectedYear,
								method: selectedMethod,
							})
						: formatMessage(messages.transfersTotalYear, {
								amount: $formatMoney(totalAmount),
								year: selectedYear,
							})
					: selectedMethod !== 'all'
						? formatMessage(messages.transfersTotalMethod, {
								amount: $formatMoney(totalAmount),
								method: selectedMethod,
							})
						: formatMessage(messages.transfersTotal, {
								amount: $formatMoney(totalAmount),
							})
			}}
		</p>

		<div v-if="filteredPayouts.length > 0" class="flex flex-col gap-3">
			<RevenueTransaction
				v-for="payout in filteredPayouts"
				:key="payout.id"
				:transaction="payout"
				@cancelled="refresh"
			/>
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
import { DropdownSelect } from '@modrinth/ui'
import { capitalizeString } from '@modrinth/utils'
import { defineMessages, useVIntl } from '@vintl/vintl'
import dayjs from 'dayjs'

import RevenueTransaction from '~/components/ui/dashboard/RevenueTransaction.vue'

const { formatMessage } = useVIntl()

useHead({
	title: 'Transfer history - Modrinth',
})

const { data: payouts, refresh } = await useAsyncData(`payout`, () =>
	useBaseFetch(`payout`, {
		apiVersion: 3,
	}),
)

const sortedPayouts = computed(() =>
	(payouts.value ? [...payouts.value] : []).sort((a, b) => dayjs(b.created) - dayjs(a.created)),
)

const years = computed(() => {
	const values = sortedPayouts.value.map((x) => dayjs(x.created).year())
	return ['all', ...new Set(values)]
})

const selectedYear = ref('all')

const methods = computed(() => {
	const values = sortedPayouts.value.filter((x) => x.method).map((x) => x.method)
	return ['all', ...new Set(values)]
})

const selectedMethod = ref('all')

const filteredPayouts = computed(() =>
	sortedPayouts.value
		.filter((x) => selectedYear.value === 'all' || dayjs(x.created).year() === selectedYear.value)
		.filter((x) => selectedMethod.value === 'all' || x.method === selectedMethod.value),
)

const totalAmount = computed(() =>
	filteredPayouts.value.reduce((sum, payout) => sum + payout.amount, 0),
)

const messages = defineMessages({
	transactionsHeader: {
		id: 'dashboard.revenue.transactions.header',
		defaultMessage: 'Transactions',
	},
	transfersTotal: {
		id: 'revenue.transfers.total',
		defaultMessage: 'You have withdrawn {amount} in total.',
	},
	transfersTotalYear: {
		id: 'revenue.transfers.total.year',
		defaultMessage: 'You have withdrawn {amount} in {year}.',
	},
	transfersTotalMethod: {
		id: 'revenue.transfers.total.method',
		defaultMessage: 'You have withdrawn {amount} through {method}.',
	},
	transfersTotalYearMethod: {
		id: 'revenue.transfers.total.year_method',
		defaultMessage: 'You have withdrawn {amount} in {year} through {method}.',
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
