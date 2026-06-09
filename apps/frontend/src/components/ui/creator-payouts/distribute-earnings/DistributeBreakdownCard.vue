<template>
	<div class="rounded-2xl border border-solid border-surface-4 bg-surface-3 p-6">
		<h2 class="m-0 text-lg font-semibold text-contrast">Distribution Breakdown</h2>

		<div class="mt-5 border-0 border-t border-solid border-surface-4 pt-5">
			<DistributeBreakdownRow
				label="Estimated Revenue"
				:value="formatCurrency(estimatedRevenue, { cents: true })"
			/>
			<DistributeBreakdownRow
				label="Clean.io Fee"
				:value="formatSignedCurrency(-Math.abs(payout.fees_deducted_usd))"
				negative
			/>
			<DistributeBreakdownRow
				label="Variance Deduction"
				:value="formatSignedCurrency(payout.variance_adjustment_usd)"
				negative
			/>
		</div>

		<div class="mt-4 border-0 border-t border-solid border-surface-4 pt-4">
			<DistributeBreakdownRow
				label="Net Estimated Revenue"
				:value="formatCurrency(payout.net_estimated_revenue_usd, { cents: true })"
				strong
			/>
		</div>

		<div class="mt-4 border-0 border-t border-dashed border-surface-4 pt-4">
			<DistributeBreakdownRow label="Actual Revenue" :value="actualRevenueLabel" />
			<DistributeBreakdownRow label="Variance Resolution" :value="varianceResolutionLabel" />
		</div>

		<div class="mt-4 border-0 border-t border-solid border-surface-4 pt-4">
			<DistributeBreakdownRow label="Net Actual Revenue" :value="netActualLabel" strong />
		</div>

		<div class="mt-4 border-0 border-t border-solid border-surface-4 pt-4">
			<DistributeBreakdownRow label="Creator Revenue (75%)" :value="creatorRevenueLabel" />
			<DistributeBreakdownRow label="Modrinth Revenue (25%)" :value="modrinthRevenueLabel" />
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { computed } from 'vue'

import {
	formatCurrency,
	formatSignedCurrency,
	getCreatorShare,
	getModrinthShare,
	getNetActualRevenue,
	getTotalAdjustments,
	type DistributionAdjustment,
} from '../utils'
import DistributeBreakdownRow from './DistributeBreakdownRow.vue'

const props = defineProps<{
	payout: Labrinth.Payouts.Internal.HistoryItem
	amountReceived: number | undefined
	adjustments: DistributionAdjustment[]
}>()

const emptyValue = '—'

const estimatedRevenue = computed(() =>
	props.payout.days.reduce((total, day) => total + (day.estimated_revenue_usd ?? 0), 0),
)
const hasActualAmount = computed(() => (props.amountReceived ?? 0) > 0)
const actualRevenue = computed(() => props.amountReceived ?? 0)
const totalAdjustments = computed(() => getTotalAdjustments(props.adjustments))
const netActualRevenue = computed(() => getNetActualRevenue(actualRevenue.value, props.adjustments))
const actualRevenueLabel = computed(() =>
	hasActualAmount.value ? formatCurrency(actualRevenue.value, { cents: true }) : emptyValue,
)
const varianceResolutionLabel = computed(() =>
	hasActualAmount.value ? formatSignedCurrency(totalAdjustments.value) : emptyValue,
)
const netActualLabel = computed(() =>
	hasActualAmount.value ? formatCurrency(netActualRevenue.value, { cents: true }) : emptyValue,
)
const creatorRevenueLabel = computed(() =>
	hasActualAmount.value
		? formatCurrency(getCreatorShare(netActualRevenue.value), { cents: true })
		: emptyValue,
)
const modrinthRevenueLabel = computed(() =>
	hasActualAmount.value
		? formatCurrency(getModrinthShare(netActualRevenue.value), { cents: true })
		: emptyValue,
)
</script>
