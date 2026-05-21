<template>
	<BaseEvent>
		<IntlFormatted :message-id="message">
			<template #backup>
				<EventEntityLink v-if="backup" :entity="backup" />
			</template>
			<template #renamed-backup>
				<EventEntityLink v-if="renamedBackup" :entity="renamedBackup" />
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
	backup?: AuditBackupEventItem
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
	createdFallback: {
		id: 'servers.audit-log.event.backup-created-fallback',
		defaultMessage: 'Created backup',
	},
	restoredFallback: {
		id: 'servers.audit-log.event.backup-restored-fallback',
		defaultMessage: 'Restored backup',
	},
	renamedFallback: {
		id: 'servers.audit-log.event.backup-renamed-fallback',
		defaultMessage: 'Renamed backup',
	},
	deletedFallback: {
		id: 'servers.audit-log.event.backup-deleted-fallback',
		defaultMessage: 'Deleted backup',
	},
	changedFallback: {
		id: 'servers.audit-log.event.backup-changed-fallback',
		defaultMessage: 'Changed backup',
	},
})

const kindMessages: Record<string, MessageDescriptor> = {
	created: messages.created,
	restored: messages.restored,
	renamed: messages.renamed,
	deleted: messages.deleted,
}

const fallbackMessages: Record<string, MessageDescriptor> = {
	created: messages.createdFallback,
	restored: messages.restoredFallback,
	renamed: messages.renamedFallback,
	deleted: messages.deletedFallback,
}

const message = computed(() =>
	props.backup
		? (kindMessages[props.kind] ?? messages.changed)
		: (fallbackMessages[props.kind] ?? messages.changedFallback),
)
const renamedBackup = computed(() =>
	props.backup
		? {
				...props.backup,
				label: props.to ?? props.backup.label,
			}
		: undefined,
)
</script>
