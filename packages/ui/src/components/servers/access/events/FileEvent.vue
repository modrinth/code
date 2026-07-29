<template>
	<BaseEvent>
		<IntlFormatted :message-id="message">
			<template #file>
				<EventEntityLink v-if="file" :entity="file" />
			</template>
			<template #from>
				<EventEntityLink v-if="from" :entity="from" />
			</template>
			<template #to>
				<EventEntityLink v-if="to" :entity="to" />
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
import type { EventEntity } from './types'

const props = defineProps<{
	kind: string
	file?: EventEntity
	from?: EventEntity
	to?: EventEntity
}>()

const messages = defineMessages({
	uploaded: {
		id: 'servers.audit-log.event.file-uploaded',
		defaultMessage: 'Uploaded file <file></file>',
	},
	deleted: {
		id: 'servers.audit-log.event.file-deleted',
		defaultMessage: 'Deleted file <file></file>',
	},
	edited: {
		id: 'servers.audit-log.event.file-edited',
		defaultMessage: 'Edited file <file></file>',
	},
	renamed: {
		id: 'servers.audit-log.event.file-renamed',
		defaultMessage: 'Renamed <from></from> to <to></to>',
	},
	changed: {
		id: 'servers.audit-log.event.file-changed',
		defaultMessage: 'Changed file <file></file>',
	},
})

const kindMessages: Record<string, MessageDescriptor> = {
	uploaded: messages.uploaded,
	deleted: messages.deleted,
	edited: messages.edited,
	renamed: messages.renamed,
}

const message = computed(() => kindMessages[props.kind] ?? messages.changed)
</script>
