<template>
	<div
		class="flex flex-col gap-2.5 rounded-2xl border border-solid p-5"
		:class="
			payout.status === 'review'
				? 'border-surface-5 bg-surface-2'
				: 'border-dashed border-surface-5 bg-surface-1.5 opacity-75'
		"
	>
		<div class="flex flex-wrap items-center gap-2 pb-1">
			<div
				class="inline-flex items-center gap-2 rounded-full border border-solid border-surface-5 bg-surface-2 px-3 py-1 text-sm font-semibold text-secondary"
			>
				<CalendarIcon v-if="payout.status === 'review'" class="size-4" aria-hidden="true" />
				<ClockIcon v-else class="size-4" aria-hidden="true" />
				{{ availabilityLabel }}
			</div>
			<div
				v-if="payout.status === 'review'"
				class="inline-flex items-center gap-1 text-sm font-bold text-red"
			>
				<CircleAlertIcon class="size-4" aria-hidden="true" />
				{{ remainingLabel }}
			</div>
		</div>

		<h2 class="m-0 text-xl font-extrabold text-contrast">
			{{ title }}
		</h2>
		<p class="m-0 max-w-[25rem] text-base font-medium text-secondary">
			{{ description }}
		</p>

		<ButtonStyled v-if="payout.status === 'review'" type="outlined" class="mt-auto w-fit">
			<NuxtLink :to="`/admin/creator-payouts/distribute?payouts_date=${payout.payouts_date}`">
				Reconcile Earnings
				<ChevronRightIcon aria-hidden="true" />
			</NuxtLink>
		</ButtonStyled>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { CalendarIcon, ChevronRightIcon, CircleAlertIcon, ClockIcon } from '@modrinth/assets'
import { ButtonStyled } from '@modrinth/ui'
import { computed } from 'vue'

import {
	formatMonthYear,
	formatShortDate,
	getDaysRemaining,
	getPendingAvailableDate,
	getReviewDueDate,
} from '../utils'

const props = defineProps<{
	payout: Labrinth.Payouts.Internal.HistoryItem
}>()

const title = computed(() =>
	props.payout.status === 'review'
		? `Distribute ${formatMonthYear(props.payout.payouts_date)} Earnings`
		: `${formatMonthYear(props.payout.payouts_date)} Earnings`,
)
const availabilityLabel = computed(() => {
	if (props.payout.status === 'review') {
		return `Due ${formatShortDate(getReviewDueDate(props.payout.payouts_date))}`
	}

	return `Available ~${formatShortDate(getPendingAvailableDate(props.payout.payouts_date))}`
})
const remainingLabel = computed(() => {
	const days = getDaysRemaining(getReviewDueDate(props.payout.payouts_date))
	return days === 1 ? '1 day remaining' : `${days} days remaining`
})
const description = computed(() =>
	props.payout.status === 'review'
		? `Reconcile advertisement revenue and distribute ${formatMonthYear(
				props.payout.payouts_date,
			)} creator earnings.`
		: 'Awaiting advertisement revenue from Aditude.',
)
</script>
