<template>
	<div class="mb-20 flex flex-col gap-4 lg:pl-8">
		<div class="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
			<span class="text-xl font-semibold text-contrast md:text-2xl">{{
				formatMessage(messages.transactionsHeader)
			}}</span>
			<div class="flex w-full flex-row gap-2 sm:max-w-[250px] md:items-center">
				<Combobox
					v-model="selectedYear"
					:options="yearOptions"
					:display-value="selectedYear === 'all' ? 'All years' : String(selectedYear)"
					listbox
				/>
				<ButtonStyled circular>
					<button
						v-tooltip="formatMessage(messages.downloadCsv)"
						:disabled="buildingCsv"
						@click="onDownloadCSV"
					>
						<SpinnerIcon v-if="buildingCsv" class="animate-spin" />
						<DownloadIcon v-else />
					</button>
				</ButtonStyled>
			</div>
		</div>
		<div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
			<div class="flex flex-col gap-3 rounded-2xl bg-surface-3 p-5">
				<div class="flex w-full justify-between">
					<span class="my-auto font-medium">{{ formatMessage(messages.received) }}</span>
					<ArrowDownLeftIcon class="size-6" />
				</div>
				<span class="text-2xl font-semibold text-contrast md:text-4xl">{{
					formatMoney(totalReceived)
				}}</span>
			</div>
			<div class="flex flex-col gap-3 rounded-2xl bg-surface-3 p-5">
				<div class="flex w-full justify-between">
					<span class="my-auto font-medium">{{ formatMessage(messages.withdrawn) }}</span>
					<ArrowUpRightIcon class="size-6" />
				</div>
				<span class="text-2xl font-semibold text-contrast md:text-4xl">{{
					formatMoney(totalWithdrawn)
				}}</span>
			</div>
			<div class="flex flex-col gap-3 rounded-2xl bg-surface-3 p-5">
				<div class="flex w-full justify-between">
					<span class="my-auto font-medium">{{ formatMessage(messages.transactions) }}</span>
					<GenericListIcon class="size-6" />
				</div>
				<span class="text-2xl font-semibold text-contrast md:text-4xl">{{
					filteredTransactions.length
				}}</span>
			</div>
		</div>
		<div
			v-if="Object.keys(groupedTransactions).length > 0"
			class="-mt-2 flex flex-col gap-5 md:gap-6"
		>
			<div
				v-for="(transactions, period) in groupedTransactions"
				:key="period"
				class="flex flex-col"
			>
				<h3 class="text-base font-medium text-primary md:text-lg">{{ period }}</h3>
				<div class="flex flex-col gap-3 md:gap-4">
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
			<span class="text-lg text-contrast md:text-xl">{{
				formatMessage(messages.noTransactions)
			}}</span>
			<span class="max-w-[256px] text-base text-secondary md:text-lg">{{
				formatMessage(messages.noTransactionsDesc)
			}}</span>
		</div>
	</div>
</template>
<script setup>
import {
	ArrowDownLeftIcon,
	ArrowUpRightIcon,
	DownloadIcon,
	GenericListIcon,
	SpinnerIcon,
} from '@modrinth/assets'
import { ButtonStyled, Combobox } from '@modrinth/ui'
import { formatMoney } from '@modrinth/utils'
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
const buildingCsv = ref(false)

const filteredTransactions = computed(() =>
	sortedTransactions.value.filter(
		(x) => selectedYear.value === 'all' || dayjs(x.created).year() === selectedYear.value,
	),
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

const totalReceived = computed(() => {
	return filteredTransactions.value
		.filter((txn) => txn.type === 'payout_available')
		.reduce((sum, txn) => sum + Number(txn.amount), 0)
})

const totalWithdrawn = computed(() => {
	return filteredTransactions.value
		.filter((txn) => txn.type === 'withdrawal')
		.reduce((sum, txn) => sum + Number(txn.amount), 0)
})

function escapeCSVField(field) {
	if (field === null || field === undefined) return ''
	const stringField = String(field)
	if (stringField.includes(',') || stringField.includes('"') || stringField.includes('\n')) {
		return `"${stringField.replace(/"/g, '""')}"`
	}
	return stringField
}

function transactionsToCSV() {
	if (!filteredTransactions.value || filteredTransactions.value.length === 0) {
		return ''
	}

	const newline = '\n'
	const header = ['Date', 'Type', 'Source', 'Status', 'Amount', 'Fee'].join(',')

	const rows = filteredTransactions.value.map((txn) => {
		const date = dayjs(txn.created).format('YYYY-MM-DD HH:mm:ss')
		const type = txn.type === 'withdrawal' ? 'Withdrawal' : 'Payout'

		let methodOrSource = ''
		let status = ''
		let fee = ''

		if (txn.type === 'withdrawal') {
			const method = txn.method_type || txn.method || 'Unknown'
			switch (method) {
				case 'paypal':
					methodOrSource = 'PayPal'
					break
				case 'venmo':
					methodOrSource = 'Venmo'
					break
				case 'tremendous':
					methodOrSource = 'Tremendous'
					break
				case 'muralpay':
					methodOrSource = 'Muralpay'
					break
				default:
					methodOrSource = method.charAt(0).toUpperCase() + method.slice(1)
			}

			status = txn.status
				? txn.status.replace(/-/g, ' ').replace(/\b\w/g, (l) => l.toUpperCase())
				: 'Unknown'

			fee = txn.fee ? Number(txn.fee).toFixed(2) : '0.00'
		} else {
			methodOrSource = txn.payout_source
				? txn.payout_source
						.split('_')
						.map((word) => word.charAt(0).toUpperCase() + word.slice(1))
						.join(' ')
				: 'Unknown'
			status = 'N/A'
			fee = 'N/A'
		}

		const amount = Number(txn.amount).toFixed(2)

		return [
			escapeCSVField(date),
			escapeCSVField(type),
			escapeCSVField(methodOrSource),
			escapeCSVField(status),
			escapeCSVField(amount),
			escapeCSVField(fee),
		].join(',')
	})

	return [header, ...rows].join(newline)
}

const downloadTransactionsCSV = () => {
	buildingCsv.value = true

	try {
		const csv = transactionsToCSV()

		if (!csv) {
			return
		}

		const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' })

		const link = document.createElement('a')
		const url = URL.createObjectURL(blob)
		const yearSuffix = selectedYear.value === 'all' ? 'all' : selectedYear.value
		const filename = `modrinth-transactions-${yearSuffix}.csv`

		link.setAttribute('href', url)
		link.setAttribute('download', filename)
		link.style.visibility = 'hidden'
		document.body.appendChild(link)

		link.click()
		document.body.removeChild(link)
	} finally {
		buildingCsv.value = false
	}
}

const onDownloadCSV = useClientTry(async () => await downloadTransactionsCSV())

const messages = defineMessages({
	transactionsHeader: {
		id: 'dashboard.revenue.transactions.header',
		defaultMessage: 'Transactions',
	},
	received: {
		id: 'dashboard.revenue.stats.received',
		defaultMessage: 'Received',
	},
	withdrawn: {
		id: 'dashboard.revenue.stats.withdrawn',
		defaultMessage: 'Withdrawn',
	},
	transactions: {
		id: 'dashboard.revenue.stats.transactions',
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
	downloadCsv: {
		id: 'dashboard.revenue.transactions.btn.download-csv',
		defaultMessage: 'Download as CSV',
	},
})
</script>
