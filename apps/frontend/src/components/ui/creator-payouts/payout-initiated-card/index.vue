<template>
	<Admonition
		type="info"
		:header="`${formatMonthYear(distribution.payouts_date)} payout initiated`"
	>
		<div class="text-primary">
			{{ formatCurrency(creatorAmount, { cents: true }) }} to creators. Processing in
			<span class="text-contrast">{{ countdownLabel }}</span>
		</div>

		<template #top-right-actions>
			<ButtonStyled type="outlined">
				<button class="!border" type="button" :disabled="cancelling" @click="$emit('cancel')">
					Cancel
				</button>
			</ButtonStyled>
		</template>
	</Admonition>
</template>

<script setup lang="ts">
import { Admonition, ButtonStyled } from '@modrinth/ui'
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'

import {
	type DistributionRun,
	formatCurrency,
	formatMonthYear,
	getDistributionCreatorAmount,
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
