<script setup lang="ts">
import { Button, commonMessages, Slider, useVIntl } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'

import { maxMemoryQueryOptions } from './queries'

const MIN_MEMORY_MB = 512
const FIRST_SNAP_POINT_MB = 2048

defineProps<{
	id?: string
	ariaLabel?: string
}>()

const memory = defineModel<number>({ required: true })

const { formatMessage } = useVIntl()

const memoryQuery = useQuery(maxMemoryQueryOptions())
const maxMemory = computed(() => Math.floor((memoryQuery.data.value ?? 0) / 1024))

const snapPoints = computed(() => {
	const points: number[] = []
	for (let point = FIRST_SNAP_POINT_MB; point <= maxMemory.value; point *= 2) {
		points.push(point)
	}
	return points
})
</script>

<template>
	<Slider
		v-if="maxMemory"
		:id="id"
		v-model="memory"
		:aria-label="ariaLabel"
		:min="MIN_MEMORY_MB"
		:max="maxMemory"
		:step="64"
		:snap-points="snapPoints"
		:snap-range="MIN_MEMORY_MB"
		:min-label="`${MIN_MEMORY_MB} MB`"
		:max-label="`${Number((maxMemory / 1024).toFixed(1))} GB`"
		unit="MB"
	/>
	<Button v-else-if="memoryQuery.isError.value" @click="memoryQuery.refetch()">
		{{ formatMessage(commonMessages.refreshButton) }}
	</Button>
	<div
		v-else
		role="status"
		:aria-label="formatMessage(commonMessages.loadingLabel)"
		class="h-10 animate-pulse rounded-lg bg-surface-3"
	/>
</template>
