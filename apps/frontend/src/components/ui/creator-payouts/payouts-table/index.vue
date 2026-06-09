<template>
	<Table :columns="columns" :data="rows" row-key="payouts_date" table-min-width="1180px">
		<template #header-period="{ column }">
			<span class="text-sm font-normal">{{ column.label }}</span>
		</template>
		<template #header-status="{ column }">
			<span class="text-sm font-normal">{{ column.label }}</span>
		</template>
		<template #header-estimated="{ column }">
			<span class="text-sm font-normal">{{ column.label }}</span>
		</template>
		<template #header-fees="{ column }">
			<span class="inline-flex items-center gap-1 text-sm font-normal">
				{{ column.label }}
				<InfoIcon class="size-4" aria-hidden="true" />
			</span>
		</template>
		<template #header-variance="{ column }">
			<span class="inline-flex items-center gap-1 text-sm font-normal">
				{{ column.label }}
				<InfoIcon class="size-4" aria-hidden="true" />
			</span>
		</template>
		<template #header-netEstimated="{ column }">
			<span class="text-sm font-normal">{{ column.label }}</span>
		</template>
		<template #header-actual="{ column }">
			<span class="text-sm font-normal">{{ column.label }}</span>
		</template>
		<template #header-external="{ column }">
			<span class="inline-flex items-center gap-1 text-sm font-normal">
				{{ column.label }}
				<InfoIcon class="size-4" aria-hidden="true" />
			</span>
		</template>
		<template #header-netActual="{ column }">
			<span class="text-sm font-normal">{{ column.label }}</span>
		</template>
		<template #header-creator="{ column }">
			<span class="text-sm font-normal">{{ column.label }}</span>
		</template>
		<template #header-modrinth="{ column }">
			<span class="text-sm font-normal">{{ column.label }}</span>
		</template>

		<template #cell-period="{ row }">
			<div
				class="flex items-center gap-3 font-medium text-contrast"
				:class="{ 'opacity-50': isDim(row) }"
			>
				<ChevronRightIcon class="size-5 text-secondary" aria-hidden="true" />
				{{ formatMonthYear(row.payouts_date) }}
			</div>
		</template>

		<template #cell-status="{ row }">
			<span
				class="inline-flex rounded-full border border-solid px-3 py-0.5 text-sm font-medium"
				:class="statusClass(row.status)"
			>
				{{ statusLabel(row.status) }}
			</span>
		</template>

		<template #cell-fees="{ row }">
			<span class="text-red">{{ formatSignedCurrency(-Math.abs(row.fees_deducted_usd)) }}</span>
		</template>
		<template #cell-variance="{ row }">
			<span class="text-red">{{ formatSignedCurrency(row.variance_adjustment_usd) }}</span>
		</template>
		<template #cell-netActual="{ row }">
			<span class="font-medium text-contrast">{{
				formatCurrency(row.net_actual_revenue_usd)
			}}</span>
		</template>
	</Table>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ChevronRightIcon, InfoIcon } from '@modrinth/assets'
import { Table, type TableColumn } from '@modrinth/ui'
import { computed } from 'vue'

import { formatCurrency, formatMonthYear, formatSignedCurrency } from '../utils'

type PayoutColumnKey =
	| 'period'
	| 'status'
	| 'estimated'
	| 'fees'
	| 'variance'
	| 'netEstimated'
	| 'actual'
	| 'external'
	| 'netActual'
	| 'creator'
	| 'modrinth'

type PayoutRow = Labrinth.Payouts.Internal.HistoryItem & Record<PayoutColumnKey, string>

const props = defineProps<{
	payouts: Labrinth.Payouts.Internal.HistoryItem[]
}>()

const columns: TableColumn<PayoutColumnKey>[] = [
	{ key: 'period', label: 'Period', width: '16%' },
	{ key: 'status', label: 'Status', width: '8.5%' },
	{ key: 'estimated', label: 'Est Rev', align: 'right', width: '10%' },
	{ key: 'fees', label: 'Fees', align: 'right', width: '10%' },
	{ key: 'variance', label: 'Variance Adj', align: 'right', width: '12%' },
	{ key: 'netEstimated', label: 'Net Est Rev', align: 'right', width: '11%' },
	{ key: 'actual', label: 'Actual Rev', align: 'right', width: '11%' },
	{ key: 'external', label: 'External Adj', align: 'right', width: '12%' },
	{ key: 'netActual', label: 'Net Actual Rev', align: 'right', width: '13%' },
	{ key: 'creator', label: 'Creator (75%)', align: 'right', width: '12%' },
	{ key: 'modrinth', label: 'Modrinth (25%)', align: 'right', width: '12%' },
]

const rows = computed<PayoutRow[]>(() =>
	[...props.payouts]
		.sort((left, right) => right.payouts_date.localeCompare(left.payouts_date))
		.map((payout) => ({
			...payout,
			period: formatMonthYear(payout.payouts_date),
			status: payout.status,
			estimated: formatCurrency(getEstimatedRevenue(payout)),
			fees: formatSignedCurrency(-Math.abs(payout.fees_deducted_usd)),
			variance: formatSignedCurrency(payout.variance_adjustment_usd),
			netEstimated: formatCurrency(payout.net_estimated_revenue_usd),
			actual: formatCurrency(payout.actual_revenue_usd),
			external: formatCurrency(payout.total_external_adjustment_usd),
			netActual: formatCurrency(payout.net_actual_revenue_usd),
			creator: formatCurrency(
				payout.creator_net_actual_revenue_usd ?? payout.creator_net_estimated_revenue_usd,
			),
			modrinth: formatCurrency(
				payout.modrinth_net_actual_revenue_usd ?? payout.modrinth_net_estimated_revenue_usd,
			),
		})),
)

function getEstimatedRevenue(payout: Labrinth.Payouts.Internal.HistoryItem): number | undefined {
	const total = payout.days.reduce((sum, day) => sum + (day.estimated_revenue_usd ?? 0), 0)
	if (total !== 0) {
		return total
	}

	return (
		payout.net_estimated_revenue_usd + payout.fees_deducted_usd - payout.variance_adjustment_usd
	)
}

function statusLabel(status: Labrinth.Payouts.Internal.PayoutStatus): string {
	return status[0].toUpperCase() + status.slice(1)
}

function statusClass(status: Labrinth.Payouts.Internal.PayoutStatus): string {
	switch (status) {
		case 'open':
			return 'border-blue bg-blue-highlight text-blue'
		case 'pending':
			return 'border-orange bg-orange-highlight text-orange'
		case 'review':
			return 'border-green bg-green-highlight text-green'
		case 'paid':
			return 'border-surface-4 bg-surface-3 text-secondary'
	}
}

function isDim(row: PayoutRow): boolean {
	return row.status === 'paid'
}
</script>
