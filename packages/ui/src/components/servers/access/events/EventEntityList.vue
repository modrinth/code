<template>
	<span class="inline-flex max-w-full flex-wrap items-center gap-x-1 gap-y-0.5 align-baseline">
		<template v-for="(entity, index) in visibleEntities" :key="entity.id">
			<EventEntityLink :entity="entity" />
			<span v-if="index < visibleEntities.length - 1" class="text-secondary">,</span>
		</template>
		<span
			v-if="hiddenCount > 0"
			v-tooltip="hiddenTooltip"
			class="inline-flex rounded-full border border-solid border-surface-5 bg-surface-4 px-1.5 text-xs font-semibold text-secondary"
		>
			+{{ hiddenCount }}
		</span>
	</span>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import EventEntityLink from './EventEntityLink.vue'
import type { EventEntity } from './types'

const props = withDefaults(
	defineProps<{
		entities: EventEntity[]
		limit?: number
	}>(),
	{
		limit: 3,
	},
)

const visibleEntities = computed(() => props.entities.slice(0, props.limit))
const hiddenEntities = computed(() => props.entities.slice(props.limit))
const hiddenCount = computed(() => hiddenEntities.value.length)
const hiddenTooltip = computed(() => hiddenEntities.value.map((entity) => entity.label).join(', '))
</script>
