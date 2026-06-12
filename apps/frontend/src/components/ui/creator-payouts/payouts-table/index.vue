<template>
	<Table
		:columns="columns"
		:data="rows"
		row-key="rowKey"
		:body-cell-class="bodyCellClass"
		row-transition-name="payout-day-row"
	>
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
				<span v-tooltip="feesTooltip" class="inline-flex" tabindex="0" :aria-label="feesTooltip">
					<InfoIcon class="size-4" aria-hidden="true" />
				</span>
			</span>
		</template>
		<template #header-variance="{ column }">
			<span class="inline-flex items-center gap-1 text-sm font-normal">
				{{ column.label }}
				<span
					v-tooltip="varianceTooltip"
					class="inline-flex"
					tabindex="0"
					:aria-label="varianceTooltip"
				>
					<InfoIcon class="size-4" aria-hidden="true" />
				</span>
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
				<span
					v-tooltip="externalTooltip"
					class="inline-flex"
					tabindex="0"
					:aria-label="externalTooltip"
				>
					<InfoIcon class="size-4" aria-hidden="true" />
				</span>
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
			<button
				v-if="row.rowKind === 'period'"
				type="button"
				class="flex h-full w-full cursor-pointer items-center gap-3 border-0 bg-transparent p-0 text-left font-medium text-contrast"
				:class="{ 'opacity-50': isDim(row) }"
				:aria-expanded="row.isExpanded"
				@click="toggleExpanded(row.payouts_date)"
			>
				<ChevronRightIcon
					class="size-5 shrink-0 text-secondary transition-transform"
					:class="{ 'rotate-90': row.isExpanded }"
					aria-hidden="true"
				/>
				{{ row.period }}
			</button>
			<div v-else class="flex items-center gap-3 pl-8 text-sm text-secondary">
				{{ row.period }}
			</div>
		</template>

		<template #cell-status="{ row }">
			<span
				v-if="row.rowKind === 'period'"
				v-tooltip="statusTooltip(row.status)"
				class="inline-flex rounded-full border border-solid px-3 py-0.5 text-sm font-medium"
				:class="statusClass(row.status)"
				tabindex="0"
				:aria-label="statusTooltip(row.status)"
			>
				{{ statusLabel(row.status) }}
			</span>
			<span v-else :class="emptyValueClass">{{ row.status }}</span>
		</template>

		<template #cell-fees="{ row }">
			<span :class="row.fees === emptyValue ? emptyValueClass : 'text-red'">{{ row.fees }}</span>
		</template>
		<template #cell-variance="{ row }">
			<span :class="row.variance === emptyValue ? emptyValueClass : 'text-red'">{{
				row.variance
			}}</span>
		</template>
		<template #cell-estimated="{ row }">
			<span :class="valueClass(row.estimated)">{{ row.estimated }}</span>
		</template>
		<template #cell-netEstimated="{ row }">
			<span :class="valueClass(row.netEstimated)">{{ row.netEstimated }}</span>
		</template>
		<template #cell-actual="{ row }">
			<span :class="valueClass(row.actual)">{{ row.actual }}</span>
		</template>
		<template #cell-external="{ row }">
			<span :class="valueClass(row.external)">{{ row.external }}</span>
		</template>
		<template #cell-netActual="{ row }">
			<span :class="row.netActual === emptyValue ? emptyValueClass : 'font-medium text-contrast'">
				{{ row.netActual }}
			</span>
		</template>
		<template #cell-creator="{ row }">
			<span
				v-tooltip="row.creatorIsEstimated ? pendingEstimatedRevenueTooltip : undefined"
				:class="valueClass(row.creator)"
				:tabindex="row.creatorIsEstimated ? 0 : undefined"
				:aria-label="row.creatorIsEstimated ? pendingEstimatedRevenueTooltip : undefined"
			>
				{{ row.creator }}
			</span>
		</template>
		<template #cell-modrinth="{ row }">
			<span
				v-tooltip="row.modrinthIsEstimated ? pendingEstimatedRevenueTooltip : undefined"
				:class="valueClass(row.modrinth)"
				:tabindex="row.modrinthIsEstimated ? 0 : undefined"
				:aria-label="row.modrinthIsEstimated ? pendingEstimatedRevenueTooltip : undefined"
			>
				{{ row.modrinth }}
			</span>
		</template>
	</Table>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ChevronRightIcon, InfoIcon } from '@modrinth/assets'
import { Table, type TableColumn } from '@modrinth/ui'
import { computed, ref } from 'vue'

import {
	formatCurrency,
	formatMonthYear,
	formatSignedCurrency,
	getCreatorShare,
	getModrinthShare,
} from '../utils'

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

type PayoutRowBase = Record<string, unknown> &
	Record<PayoutColumnKey, string> & {
		rowKey: string
		rowKind: 'period' | 'day'
		payouts_date: string
		isDimmed: boolean
		isExpanded: boolean
		creatorIsEstimated: boolean
		modrinthIsEstimated: boolean
	}

type PeriodRow = PayoutRowBase & {
	rowKind: 'period'
	status: Labrinth.Payouts.Internal.PayoutStatus
}

type DayRow = PayoutRowBase & {
	rowKind: 'day'
}

type PayoutRow = PeriodRow | DayRow

const props = defineProps<{
	payouts: Labrinth.Payouts.Internal.HistoryItem[]
}>()

const emptyValue = '—'
const emptyValueClass = 'text-primary opacity-60'
const feesTooltip = 'Deduction to cover Clean.io fees'
const varianceTooltip = 'Deduction to account for variance between estimated and actual revenue.'
const externalTooltip =
	'Manual adjustments for direct campaigns, overreported revenue, or other corrections.'
const pendingEstimatedRevenueTooltip = 'Pending estimated revenue'
const expandedPayoutDates = ref<Set<string>>(new Set())

const columns: TableColumn<PayoutColumnKey>[] = [
	{ key: 'period', label: 'Period', width: '16%' },
	{ key: 'status', label: 'Status', width: '8.75%' },
	{ key: 'estimated', label: 'Est Rev', align: 'right', width: '10%' },
	{ key: 'fees', label: 'Fees', align: 'right', width: '10%' },
	{ key: 'variance', label: 'Variance Adj', align: 'right', width: '12%' },
	{ key: 'netEstimated', label: 'Net Est Rev', align: 'right', width: '11%' },
	{ key: 'actual', label: 'Actual Rev', align: 'right', width: '11%' },
	{ key: 'external', label: 'External Adj', align: 'right', width: '12%' },
	{ key: 'netActual', label: 'Net Actual Rev', align: 'right', width: '13%' },
	{ key: 'creator', label: 'Creator (75%)', align: 'right', width: '12%' },
	{ key: 'modrinth', label: 'Modrinth (25%)', align: 'right', width: '14%' },
]

const rows = computed<PayoutRow[]>(() => {
	const tableRows: PayoutRow[] = []

	for (const payout of [...props.payouts].sort((left, right) =>
		right.payouts_date.localeCompare(left.payouts_date),
	)) {
		const isExpanded = expandedPayoutDates.value.has(payout.payouts_date)

		tableRows.push({
			rowKey: `period-${payout.payouts_date}`,
			rowKind: 'period',
			payouts_date: payout.payouts_date,
			isDimmed: payout.status === 'paid',
			isExpanded,
			period: formatMonthYear(payout.payouts_date),
			status: payout.status,
			estimated: formatCurrency(getEstimatedRevenue(payout)),
			fees: formatSignedCurrency(-Math.abs(payout.fees_deducted_usd)),
			variance: formatSignedCurrency(payout.variance_adjustment_usd),
			netEstimated: formatCurrency(payout.net_estimated_revenue_usd),
			actual: formatCurrency(payout.actual_revenue_usd),
			external: formatCurrency(payout.total_external_adjustment_usd),
			netActual: formatCurrency(payout.net_actual_revenue_usd),
			creator: formatPayoutSplitCurrency(
				payout.creator_net_actual_revenue_usd,
				payout.creator_net_estimated_revenue_usd,
			),
			modrinth: formatPayoutSplitCurrency(
				payout.modrinth_net_actual_revenue_usd,
				payout.modrinth_net_estimated_revenue_usd,
			),
			creatorIsEstimated: payout.creator_net_actual_revenue_usd === undefined,
			modrinthIsEstimated: payout.modrinth_net_actual_revenue_usd === undefined,
		})

		if (isExpanded) {
			payout.days.forEach((day, dayIndex) => {
				const dailyEstimatedRevenue = day.estimated_revenue_usd
				const dailyFeesDeducted = getDailyFeesDeducted(payout)
				const dailyNetEstimatedRevenue = getDailyNetEstimatedRevenue(
					dailyEstimatedRevenue,
					dailyFeesDeducted,
				)
				const dailyCreatorRevenue = getDailyCreatorRevenue(dailyNetEstimatedRevenue)
				const dailyModrinthRevenue = getDailyModrinthRevenue(dailyNetEstimatedRevenue)

				tableRows.push({
					rowKey: `day-${payout.payouts_date}-${dayIndex}`,
					rowKind: 'day',
					payouts_date: payout.payouts_date,
					isDimmed: payout.status === 'paid',
					isExpanded: false,
					period: formatDayLabel(payout.payouts_date, dayIndex),
					status: emptyValue,
					estimated: formatCurrency(dailyEstimatedRevenue),
					fees: formatSignedCurrency(-dailyFeesDeducted),
					variance: emptyValue,
					netEstimated: formatCurrency(dailyNetEstimatedRevenue),
					actual: emptyValue,
					external: emptyValue,
					netActual: emptyValue,
					creator: formatEstimatedCurrency(dailyCreatorRevenue),
					modrinth: formatEstimatedCurrency(dailyModrinthRevenue),
					creatorIsEstimated: dailyCreatorRevenue !== undefined,
					modrinthIsEstimated: dailyModrinthRevenue !== undefined,
				})
			})
		}
	}

	return tableRows
})

function toggleExpanded(payoutsDate: string) {
	const nextExpandedPayoutDates = new Set(expandedPayoutDates.value)
	if (nextExpandedPayoutDates.has(payoutsDate)) {
		nextExpandedPayoutDates.delete(payoutsDate)
	} else {
		nextExpandedPayoutDates.add(payoutsDate)
	}
	expandedPayoutDates.value = nextExpandedPayoutDates
}

function formatDayLabel(payoutsDate: string, dayIndex: number): string {
	const [year, month] = payoutsDate.split('-').map(Number)
	return new Intl.DateTimeFormat(undefined, {
		month: 'short',
		day: 'numeric',
	}).format(new Date(year, month - 1, dayIndex + 1, 12))
}

function getDailyFeesDeducted(payout: Labrinth.Payouts.Internal.HistoryItem): number {
	return payout.days.length > 0 ? payout.fees_deducted_usd / payout.days.length : 0
}

function getDailyNetEstimatedRevenue(
	estimatedRevenue: number | null,
	feesDeducted: number,
): number | undefined {
	if (estimatedRevenue === null) {
		return undefined
	}

	return estimatedRevenue - feesDeducted
}

function getDailyCreatorRevenue(netEstimatedRevenue: number | undefined): number | undefined {
	return netEstimatedRevenue === undefined ? undefined : getCreatorShare(netEstimatedRevenue)
}

function getDailyModrinthRevenue(netEstimatedRevenue: number | undefined): number | undefined {
	return netEstimatedRevenue === undefined ? undefined : getModrinthShare(netEstimatedRevenue)
}

function formatPayoutSplitCurrency(
	actualRevenue: number | undefined,
	estimatedRevenue: number | undefined,
): string {
	if (actualRevenue !== undefined) {
		return formatCurrency(actualRevenue)
	}

	return formatEstimatedCurrency(estimatedRevenue)
}

function formatEstimatedCurrency(amount: number | undefined): string {
	const formattedAmount = formatCurrency(amount)
	return formattedAmount === emptyValue ? formattedAmount : `~${formattedAmount}`
}

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

function statusTooltip(status: Labrinth.Payouts.Internal.PayoutStatus): string {
	switch (status) {
		case 'open':
			return 'Revenue is still being earned for this month.'
		case 'pending':
			return 'Month closed. Awaiting payout from ad provider on its NET 60 schedule (~60 days after month-end).'
		case 'review':
			return 'Distribution of this month is in review, awaiting finalization.'
		case 'paid':
			return 'Revenue paid and distributed.'
	}
}

function isDim(row: PayoutRow): boolean {
	return row.isDimmed
}

function bodyCellClass(row: PayoutRow): string {
	return row.rowKind === 'day' ? 'h-8' : 'h-14'
}

function valueClass(value: string): string {
	return value === emptyValue ? emptyValueClass : 'text-secondary'
}
</script>

<style scoped>
:deep(.payout-day-row-enter-active),
:deep(.payout-day-row-leave-active) {
	transition:
		opacity 150ms ease,
		transform 150ms ease;
}

:deep(.payout-day-row-enter-from),
:deep(.payout-day-row-leave-to) {
	opacity: 0;
	transform: translateY(-4px);
}
</style>
