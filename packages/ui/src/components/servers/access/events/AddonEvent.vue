<template>
	<BaseEvent>
		<template v-if="kind === 'uploaded'">
			Uploaded <EventEntityList :entities="fileNames ?? []" />
		</template>
		<template v-else>
			{{ label }} <EventEntityList :entities="addonEntities" />
		</template>
	</BaseEvent>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import BaseEvent from './BaseEvent.vue'
import EventEntityList from './EventEntityList.vue'
import type { AuditAddonEventItem, EventEntity } from './types'

const props = defineProps<{
	kind: string
	addons?: AuditAddonEventItem[]
	fileNames?: EventEntity[]
}>()

const label = computed(() => {
	switch (props.kind) {
		case 'added':
			return 'Added content'
		case 'disabled':
			return 'Disabled content'
		case 'enabled':
			return 'Enabled content'
		case 'deleted':
			return 'Deleted content'
		case 'updated':
			return 'Updated content'
		default:
			return 'Changed content'
	}
})

const addonEntities = computed(() => props.addons?.map((addon) => addon.project) ?? [])
</script>
