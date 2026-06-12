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
				:value="formatSignedCurrencyWithCents(-Math.abs(payout.fees_deducted_usd))"
				:tone="getAmountTone(-Math.abs(payout.fees_deducted_usd))"
			/>
			<DistributeBreakdownRow
				label="Variance Deduction"
				:value="formatSignedCurrencyWithCents(payout.variance_adjustment_usd)"
				:tone="getAmountTone(payout.variance_adjustment_usd)"
			/>
		</div>

		<div class="mt-4 border-0 border-t border-solid border-surface-4 pt-4">
			<DistributeBreakdownRow
				label="Net Estimated Revenue"
				:value="formatCurrency(payout.net_estimated_revenue_usd, { cents: true })"
				strong
			/>
		</div>

		<div class="mt-4 border-0 border-t border-solid border-surface-4 pt-4">
			<DistributeBreakdownRow label="Actual Revenue" :value="actualRevenueLabel" />
			<DistributeBreakdownRow
				label="Variance Resolution"
				:value="varianceResolutionLabel"
				:description="varianceResolutionDescription"
				:tone="getAmountTone(varianceResolution)"
			/>
			<DistributeBreakdownRow
				v-for="(adjustment, index) in adjustments"
				:key="`${index}-${adjustment.description}`"
				:label="adjustment.description"
				:value="formatSignedCurrencyWithCents(adjustment.amount)"
				:tone="getAmountTone(adjustment.amount)"
			/>
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
	getCreatorShare,
	getModrinthShare,
	getNetActualRevenue,
	roundCurrency,
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
const netActualRevenue = computed(() => getNetActualRevenue(actualRevenue.value, props.adjustments))
const varianceResolution = computed(() =>
	roundCurrency(actualRevenue.value - props.payout.net_estimated_revenue_usd),
)
const varianceDeduction = computed(() => Math.abs(props.payout.variance_adjustment_usd))
const returnedVariance = computed(() =>
	Math.min(Math.max(varianceResolution.value, 0), varianceDeduction.value),
)
const actualRevenueLabel = computed(() =>
	hasActualAmount.value ? formatCurrency(actualRevenue.value, { cents: true }) : emptyValue,
)
const varianceResolutionLabel = computed(() =>
	hasActualAmount.value ? formatSignedCurrencyWithCents(varianceResolution.value) : emptyValue,
)
const varianceResolutionDescription = computed(() => {
	if (!hasActualAmount.value) {
		return undefined
	}

	if (varianceResolution.value < 0) {
		return `Variance consumed + ${formatCurrency(Math.abs(varianceResolution.value), { cents: true })} additional shortfall`
	}

	if (varianceDeduction.value === 0) {
		return undefined
	}

	if (varianceResolution.value >= varianceDeduction.value) {
		return 'Variance deduction fully returned'
	}

	return `${formatCurrency(returnedVariance.value, { cents: true })} of ${formatCurrency(varianceDeduction.value, { cents: true })} returned`
})
const adjustments = computed(() =>
	props.adjustments.filter((adjustment) => adjustment.description || adjustment.amount !== 0),
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

function formatSignedCurrencyWithCents(amount: number): string {
	const formatted = formatCurrency(Math.abs(amount), { cents: true })

	if (amount < 0) {
		return `-${formatted}`
	}

	if (amount > 0) {
		return `+${formatted}`
	}

	return formatted
}

function getAmountTone(amount: number): 'positive' | 'negative' | 'neutral' {
	if (amount > 0) {
		return 'positive'
	}

	if (amount < 0) {
		return 'negative'
	}

	return 'neutral'
}
</script>
