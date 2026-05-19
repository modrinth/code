<template>
	<BaseEvent>
		<IntlFormatted :message-id="message">
			<template #content>
				<EventEntityList :entities="addonEntities" />
			</template>
			<template #files>
				<EventEntityList :entities="fileNames ?? []" />
			</template>
		</IntlFormatted>
	</BaseEvent>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import { defineMessages, type MessageDescriptor } from '../../../../composables/i18n'
import IntlFormatted from '../../../base/IntlFormatted.vue'
import BaseEvent from './BaseEvent.vue'
import EventEntityList from './EventEntityList.vue'
import type { AuditAddonEventItem, EventEntity } from './types'

const props = defineProps<{
	kind: string
	addons?: AuditAddonEventItem[]
	fileNames?: EventEntity[]
}>()

const messages = defineMessages({
	added: {
		id: 'servers.audit-log.event.addon-added',
		defaultMessage: 'Added content <content></content>',
	},
	uploaded: {
		id: 'servers.audit-log.event.addon-uploaded',
		defaultMessage: 'Uploaded <files></files>',
	},
	disabled: {
		id: 'servers.audit-log.event.addon-disabled',
		defaultMessage: 'Disabled content <content></content>',
	},
	enabled: {
		id: 'servers.audit-log.event.addon-enabled',
		defaultMessage: 'Enabled content <content></content>',
	},
	deleted: {
		id: 'servers.audit-log.event.addon-deleted',
		defaultMessage: 'Deleted content <content></content>',
	},
	updated: {
		id: 'servers.audit-log.event.addon-updated',
		defaultMessage: 'Updated content <content></content>',
	},
	changed: {
		id: 'servers.audit-log.event.addon-changed',
		defaultMessage: 'Changed content <content></content>',
	},
})

const kindMessages: Record<string, MessageDescriptor> = {
	added: messages.added,
	uploaded: messages.uploaded,
	disabled: messages.disabled,
	enabled: messages.enabled,
	deleted: messages.deleted,
	updated: messages.updated,
}

const message = computed(() => kindMessages[props.kind] ?? messages.changed)
const addonEntities = computed(() => props.addons?.map((addon) => addon.project) ?? [])
</script>
