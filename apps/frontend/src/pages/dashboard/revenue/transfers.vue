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
			<div v-for="payout in filteredPayouts" :key="payout.id" class="flex flex-row gap-3">
				<div
					class="flex h-12 min-h-12 w-12 min-w-12 justify-center rounded-full border-[1px] border-solid border-button-bg bg-bg-raised !p-0 shadow-md"
				>
					<ArrowUpIcon class="my-auto size-8 text-secondary" />
				</div>
				<div class="flex w-full flex-row justify-between">
					<div class="flex flex-col">
						<span class="text-lg font-semibold text-contrast">{{
							formatMethodName(payout.method)
						}}</span>
						<span class="text-secondary">
							{{ formatTransactionStatus(payout.status) }} |
							{{ $dayjs(payout.created).format('MMM DD YYYY') }}
							<template v-if="payout.fee"> | Fee {{ $formatMoney(payout.fee) }}</template>
							<template v-if="payout.method_address">
								| {{ formatWallet(payout.method) }} ({{ payout.method_address }})
							</template>
						</span>
					</div>
					<div class="my-auto flex flex-col items-end">
						<span class="text-lg font-semibold text-contrast">{{
							$formatMoney(payout.amount)
						}}</span>
						<button
							v-if="payout.status === 'in-transit'"
							class="mt-1 inline-flex items-center gap-1 rounded-md border border-button-bg bg-button-bg px-3 py-1 text-sm text-contrast hover:brightness-110"
							@click="cancelPayout(payout.id)"
						>
							<XIcon class="size-4" />
							Cancel payment
						</button>
					</div>
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
import { ArrowUpIcon, XIcon } from '@modrinth/assets'
import { DropdownSelect, injectNotificationManager } from '@modrinth/ui'
import { capitalizeString, formatWallet } from '@modrinth/utils'
import { defineMessages, useVIntl } from '@vintl/vintl'
import dayjs from 'dayjs'

const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

useHead({
	title: 'Transfer history - Modrinth',
})

const auth = await useAuth()

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

async function cancelPayout(id) {
	startLoading()
	try {
		await useBaseFetch(`payout/${id}`, {
			method: 'DELETE',
			apiVersion: 3,
		})
		await refresh()
		await useAuth(auth.value.token)
	} catch (err) {
		addNotification({
			title: 'An error occurred',
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
	stopLoading()
}

function formatTransactionStatus(status) {
	if (status === 'in-transit') return 'In Transit'
	return capitalizeString(status)
}

function formatMethodName(method) {
	if (!method) return 'Unknown'
	switch (method) {
		case 'paypal':
			return 'PayPal'
		case 'venmo':
			return 'Venmo'
		case 'tremendous':
			return 'Tremendous'
		default:
			return capitalizeString(method)
	}
}

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
