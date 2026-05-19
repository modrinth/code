<template>
	<BaseEvent>
		<template v-if="kind === 'created'">Created backup <EventEntityLink :entity="backup" /></template>
		<template v-else-if="kind === 'restored'">
			Restored backup <EventEntityLink :entity="backup" />
		</template>
		<template v-else-if="kind === 'renamed'">
			Renamed backup
			<span class="font-semibold text-contrast">{{ from }}</span>
			to <EventEntityLink :entity="renamedBackup" />
		</template>
		<template v-else>Deleted backup <EventEntityLink :entity="backup" /></template>
	</BaseEvent>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import BaseEvent from './BaseEvent.vue'
import EventEntityLink from './EventEntityLink.vue'
import type { AuditBackupEventItem } from './types'

const props = defineProps<{
	kind: string
	backup: AuditBackupEventItem
	backupId?: string
	from?: string
	to?: string
}>()

const renamedBackup = computed(() => ({
	...props.backup,
	label: props.to ?? props.backup.label,
}))
</script>
