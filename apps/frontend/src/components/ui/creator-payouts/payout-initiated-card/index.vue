<template>
	<div
		class="bg-blue-highlight flex items-center justify-between gap-4 rounded-2xl border border-solid border-blue p-7"
	>
		<div class="flex min-w-0 items-center gap-5">
			<div
				class="bg-blue-highlight grid size-11 shrink-0 place-items-center rounded-full text-blue"
			>
				<div class="size-3 rounded-full bg-blue" />
			</div>
			<div class="min-w-0">
				<h2 class="m-0 text-lg font-semibold text-contrast">
					{{ formatMonthYear(distribution.payouts_date) }} payout initiated
				</h2>
				<p class="m-0 text-base font-medium text-secondary">
					{{ formatCurrency(creatorAmount, { cents: true }) }} to creators — Processing in
					<span class="font-semibold text-contrast">{{ countdownLabel }}</span>
				</p>
			</div>
		</div>

		<ButtonStyled type="outlined">
			<button :disabled="cancelling" @click="$emit('cancel')">Cancel</button>
		</ButtonStyled>
	</div>
</template>

<script setup lang="ts">
import { ButtonStyled } from '@modrinth/ui'
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'

import {
	formatCurrency,
	formatMonthYear,
	getDistributionCreatorAmount,
	type DistributionRun,
} from '../utils'

const props = defineProps<{
	distribution: DistributionRun
	cancelling?: boolean
}>()

defineEmits<{
	cancel: []
}>()

const now = ref(Date.now())
let interval: ReturnType<typeof setInterval> | undefined

const creatorAmount = computed(() => getDistributionCreatorAmount(props.distribution))
const countdownLabel = computed(() => {
	const remainingSeconds = Math.max(
		0,
		Math.floor((new Date(props.distribution.distributes_at).getTime() - now.value) / 1000),
	)
	const minutes = Math.floor(remainingSeconds / 60)
	const seconds = remainingSeconds % 60
	return `${minutes}:${seconds.toString().padStart(2, '0')}`
})

onMounted(() => {
	interval = setInterval(() => {
		now.value = Date.now()
	}, 1000)
})

onBeforeUnmount(() => {
	if (interval) {
		clearInterval(interval)
	}
})
</script>
