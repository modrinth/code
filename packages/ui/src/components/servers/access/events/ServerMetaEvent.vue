<template>
	<BaseEvent>
		<template v-if="kind === 'name'">
			Changed server name to <EventEntityLink :entity="{ id: name ?? '', label: name ?? '' }" />
		</template>
		<template v-else-if="kind === 'subdomain'">
			Changed server subdomain to
			<EventEntityLink :entity="{ id: subdomain ?? '', label: subdomain ?? '', mono: true }" />
		</template>
		<template v-else>
			Changed plan to <span class="font-semibold text-contrast">{{ specsLabel }}</span>
		</template>
	</BaseEvent>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import BaseEvent from './BaseEvent.vue'
import EventEntityLink from './EventEntityLink.vue'

const props = defineProps<{
	kind: 'name' | 'subdomain' | 'plan'
	name?: string
	subdomain?: string
	newSpecs?: Record<string, unknown>
}>()

const specsLabel = computed(() => {
	const cpu = numberValue(props.newSpecs?.cpu)
	const memory = numberValue(props.newSpecs?.memory_mb)
	const storage = numberValue(props.newSpecs?.storage_mb)
	const parts = []
	if (cpu != null) parts.push(`${cpu} CPU`)
	if (memory != null) parts.push(`${formatMb(memory)} RAM`)
	if (storage != null) parts.push(`${formatMb(storage)} storage`)
	return parts.length > 0 ? parts.join(' / ') : 'new plan'
})

function numberValue(value: unknown): number | null {
	return typeof value === 'number' && Number.isFinite(value) ? value : null
}

function formatMb(value: number): string {
	if (value >= 1024) return `${Math.round(value / 1024)} GB`
	return `${value} MB`
}
</script>
