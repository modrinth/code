<template>
	<div
		ref="eventRef"
		v-tooltip="truncatedTooltip(eventRef, eventTooltipText)"
		class="audit-log-table-event line-clamp-1 min-w-0"
	>
		<component
			:is="event.component"
			v-bind="event.props"
			class="audit-log-table-event-component"
		/>
	</div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'

import { truncatedTooltip } from '#ui/utils/truncate'

import type { ParsedAuditEvent } from './events/types'

defineProps<{
	event: ParsedAuditEvent
}>()

const eventRef = ref<HTMLElement | null>(null)
const eventTooltipText = computed(
	() => eventRef.value?.textContent?.replace(/\s+/g, ' ').trim() ?? '',
)
</script>
