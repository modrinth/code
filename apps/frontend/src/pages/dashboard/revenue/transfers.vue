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
					:display-value="
						selectedYear === 'all' ? formatMessage(messages.allYears) : String(selectedYear)
					"
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
						@cancelled="refetch"
					/>
				</div>
			</div>
		</div>
		<EmptyState
			v-else
			:heading="formatMessage(messages.noTransactions)"
			:description="formatMessage(messages.noTransactionsDesc)"
		/>
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
import {
	ButtonStyled,
	Combobox,
	commonMessages,
	defineMessages,
	EmptyState,
	injectModrinthClient,
	useFormatDateTime,
	useFormatMoney,
	useVIntl,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import dayjs from 'dayjs'

import RevenueTransaction from '~/components/ui/dashboard/RevenueTransaction.vue'
import { useGeneratedState } from '~/composables/generated'
import { findRail } from '~/utils/muralpay-rails'

const { formatMessage } = useVIntl()
const formatMoney = useFormatMoney()
const formatMonth = useFormatDateTime({
	year: 'numeric',
	month: 'long',
})

const client = injectModrinthClient()
const generatedState = useGeneratedState()

useHead({
	title: () => `${formatMessage(messages.headTitle)} - Modrinth`,
})

const { data: transactions, refetch } = useQuery({
	queryKey: ['payout', 'history'],
	queryFn: () => client.labrinth.payout_v3.getHistory(),
})

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
		label: year === 'all' ? formatMessage(messages.allYears) : String(year),
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
		return formatMessage(messages.thisMonth)
	} else if (txnDate.isSame(now.subtract(1, 'month'), 'month')) {
		return formatMessage(messages.lastMonth)
	} else {
		return formatMonth(txnDate.toDate())
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
	const header = [
		formatMessage(messages.csvDateHeader),
		formatMessage(messages.csvTypeHeader),
		formatMessage(messages.csvSourceHeader),
		formatMessage(messages.csvStatusHeader),
		formatMessage(messages.csvAmountHeader),
		formatMessage(messages.csvFeeHeader),
	].join(',')

	const rows = filteredTransactions.value.map((txn) => {
		const date = dayjs(txn.created).format('YYYY-MM-DD HH:mm:ss')
		const type =
			txn.type === 'withdrawal'
				? formatMessage(messages.csvTypeWithdrawal)
				: formatMessage(messages.csvTypePayout)

		let methodOrSource = ''
		let status = ''
		let fee = ''

		if (txn.type === 'withdrawal') {
			const method = txn.method_type || txn.method || 'unknown'
			switch (method) {
				case 'paypal':
					methodOrSource = 'PayPal'
					break
				case 'venmo':
					methodOrSource = 'Venmo'
					break
				case 'tremendous':
					if (txn.method_id) {
						const info = generatedState.value.tremendousIdMap?.[txn.method_id]
						if (info) {
							methodOrSource = `Tremendous (${info.name})`
							break
						}
					}
					methodOrSource = 'Tremendous'
					break
				case 'muralpay':
					if (txn.method_id) {
						const rail = findRail(txn.method_id)
						if (rail) {
							methodOrSource = `${rail.name.defaultMessage}`
							break
						}
					}
					methodOrSource = formatMessage(messages.muralPayUnknown, {
						unknown: formatMessage(commonMessages.unknownLabel),
					})
					break
				default:
					methodOrSource =
						method === 'unknown'
							? formatMessage(commonMessages.unknownLabel)
							: method.charAt(0).toUpperCase() + method.slice(1)
			}

			status = txn.status
				? txn.status.replace(/-/g, ' ').replace(/\b\w/g, (l) => l.toUpperCase())
				: formatMessage(commonMessages.unknownLabel)

			fee = txn.fee ? Number(txn.fee).toFixed(2) : '0.00'
		} else {
			methodOrSource = txn.payout_source
				? txn.payout_source
						.split('_')
						.map((word) => word.charAt(0).toUpperCase() + word.slice(1))
						.join(' ')
				: formatMessage(commonMessages.unknownLabel)
			status = formatMessage(messages.notApplicable)
			fee = formatMessage(messages.notApplicable)
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
	headTitle: {
		id: 'dashboard.revenue.transactions.head-title',
		defaultMessage: 'Transaction history',
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
	allYears: {
		id: 'dashboard.revenue.transactions.year.all',
		defaultMessage: 'All years',
	},
	thisMonth: {
		id: 'dashboard.revenue.transactions.period.this-month',
		defaultMessage: 'This month',
	},
	lastMonth: {
		id: 'dashboard.revenue.transactions.period.last-month',
		defaultMessage: 'Last month',
	},
	notApplicable: {
		id: 'dashboard.revenue.transactions.not-applicable',
		defaultMessage: 'N/A',
	},
	csvDateHeader: {
		id: 'dashboard.revenue.transactions.csv.header.date',
		defaultMessage: 'Date',
	},
	csvTypeHeader: {
		id: 'dashboard.revenue.transactions.csv.header.type',
		defaultMessage: 'Type',
	},
	csvSourceHeader: {
		id: 'dashboard.revenue.transactions.csv.header.source',
		defaultMessage: 'Source',
	},
	csvStatusHeader: {
		id: 'dashboard.revenue.transactions.csv.header.status',
		defaultMessage: 'Status',
	},
	csvAmountHeader: {
		id: 'dashboard.revenue.transactions.csv.header.amount',
		defaultMessage: 'Amount',
	},
	csvFeeHeader: {
		id: 'dashboard.revenue.transactions.csv.header.fee',
		defaultMessage: 'Fee',
	},
	csvTypeWithdrawal: {
		id: 'dashboard.revenue.transactions.csv.type.withdrawal',
		defaultMessage: 'Withdrawal',
	},
	csvTypePayout: {
		id: 'dashboard.revenue.transactions.csv.type.payout',
		defaultMessage: 'Payout',
	},
	muralPayUnknown: {
		id: 'dashboard.revenue.transactions.csv.source.mural-pay-unknown',
		defaultMessage: 'Mural Pay ({unknown})',
	},
})
</script>
