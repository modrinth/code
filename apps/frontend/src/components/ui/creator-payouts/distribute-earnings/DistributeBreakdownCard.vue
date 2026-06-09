<template>
	<div class="rounded-2xl border border-solid border-surface-4 bg-surface-2 p-7">
		<h2 class="m-0 text-lg font-semibold text-contrast">Distribution Breakdown</h2>

		<div class="mt-5 border-0 border-t border-solid border-surface-4 pt-5">
			<BreakdownRow label="Estimated Revenue" :value="formatCurrency(estimatedRevenue, { cents: true })" />
			<BreakdownRow
				label="Clean.io Fee"
				:value="formatSignedCurrency(-Math.abs(payout.fees_deducted_usd))"
				negative
			/>
			<BreakdownRow
				label="Variance Deduction"
				:value="formatSignedCurrency(payout.variance_adjustment_usd)"
				negative
			/>
		</div>

		<div class="mt-4 border-0 border-t border-solid border-surface-4 pt-4">
			<BreakdownRow
				label="Net Estimated Revenue"
				:value="formatCurrency(payout.net_estimated_revenue_usd, { cents: true })"
				strong
			/>
		</div>

		<div class="mt-4 border-0 border-t border-dashed border-surface-4 pt-4">
			<BreakdownRow label="Actual Revenue" :value="actualRevenueLabel" />
			<BreakdownRow label="Variance Resolution" :value="varianceResolutionLabel" />
		</div>

		<div class="mt-4 border-0 border-t border-solid border-surface-4 pt-4">
			<BreakdownRow label="Net Actual Revenue" :value="netActualLabel" strong />
		</div>

		<div class="mt-4 border-0 border-t border-solid border-surface-4 pt-4">
			<BreakdownRow label="Creator Revenue (75%)" :value="creatorRevenueLabel" />
			<BreakdownRow label="Modrinth Revenue (25%)" :value="modrinthRevenueLabel" />
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { computed, defineComponent, h } from 'vue'

import {
	formatCurrency,
	formatSignedCurrency,
	getCreatorShare,
	getModrinthShare,
	getNetActualRevenue,
	getTotalAdjustments,
	type DistributionAdjustment,
} from '../utils'

const props = defineProps<{
	payout: Labrinth.Payouts.Internal.HistoryItem
	amountReceived: number | undefined
	adjustments: DistributionAdjustment[]
}>()

const emptyValue = '—'

const BreakdownRow = defineComponent({
	props: {
		label: { type: String, required: true },
		value: { type: String, required: true },
		negative: { type: Boolean, default: false },
		strong: { type: Boolean, default: false },
	},
	setup(rowProps) {
		return () =>
			h('div', { class: 'mb-4 flex items-center justify-between gap-6 last:mb-0' }, [
				h(
					'span',
					{
						class: [
							'text-base font-medium',
							rowProps.strong ? 'text-contrast font-semibold' : 'text-secondary',
						],
					},
					rowProps.label,
				),
				h(
					'span',
					{
						class: [
							'text-right text-base font-semibold',
							rowProps.value === emptyValue
								? 'text-primary'
								: rowProps.strong
									? 'text-contrast text-xl font-semibold'
									: 'text-primary',
							rowProps.negative ? 'text-red' : '',
						],
					},
					rowProps.value,
				),
			])
	},
})

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
