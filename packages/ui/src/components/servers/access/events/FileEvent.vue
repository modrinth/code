<template>
	<BaseEvent>
		<template v-if="kind === 'renamed' && from && to">
			Renamed <EventEntityLink :entity="from" /> to <EventEntityLink :entity="to" />
		</template>
		<template v-else-if="file">
			{{ label }} <EventEntityLink :entity="file" />
		</template>
	</BaseEvent>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import BaseEvent from './BaseEvent.vue'
import EventEntityLink from './EventEntityLink.vue'
import type { EventEntity } from './types'

const props = defineProps<{
	kind: string
	file?: EventEntity
	from?: EventEntity
	to?: EventEntity
}>()

const label = computed(() => {
	switch (props.kind) {
		case 'uploaded':
			return 'Uploaded file'
		case 'deleted':
			return 'Deleted file'
		case 'edited':
			return 'Edited file'
		default:
			return 'Changed file'
	}
})
</script>
