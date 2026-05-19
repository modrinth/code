<template>
	<BaseEvent>
		<IntlFormatted :message-id="message">
			<template #backup>
				<EventEntityLink :entity="backup" />
			</template>
			<template #renamed-backup>
				<EventEntityLink :entity="renamedBackup" />
			</template>
			<template #from>
				<span class="font-semibold text-contrast">{{ from }}</span>
			</template>
		</IntlFormatted>
	</BaseEvent>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import { defineMessages, type MessageDescriptor } from '../../../../composables/i18n'
import IntlFormatted from '../../../base/IntlFormatted.vue'
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

const messages = defineMessages({
	created: {
		id: 'servers.audit-log.event.backup-created',
		defaultMessage: 'Created backup <backup></backup>',
	},
	restored: {
		id: 'servers.audit-log.event.backup-restored',
		defaultMessage: 'Restored backup <backup></backup>',
	},
	renamed: {
		id: 'servers.audit-log.event.backup-renamed',
		defaultMessage: 'Renamed backup <from></from> to <renamed-backup></renamed-backup>',
	},
	deleted: {
		id: 'servers.audit-log.event.backup-deleted',
		defaultMessage: 'Deleted backup <backup></backup>',
	},
	changed: {
		id: 'servers.audit-log.event.backup-changed',
		defaultMessage: 'Changed backup <backup></backup>',
	},
})

const kindMessages: Record<string, MessageDescriptor> = {
	created: messages.created,
	restored: messages.restored,
	renamed: messages.renamed,
	deleted: messages.deleted,
}

const message = computed(() => kindMessages[props.kind] ?? messages.changed)
const renamedBackup = computed(() => ({
	...props.backup,
	label: props.to ?? props.backup.label,
}))
</script>
