<template>
	<BaseEvent>
		<span
			v-if="isDeleted"
			class="inline-flex min-w-0 max-w-full flex-wrap items-center gap-1 whitespace-normal align-middle @[800px]:flex-nowrap @[800px]:whitespace-nowrap"
		>
			<span class="shrink-0">{{ formatMessage(messages.deletedLabel) }}</span>
			<EventEntityList
				class="min-w-0"
				:entities="addonEntities"
				:limit="1"
				single-line
				entity-text-weight="semibold"
			/>
		</span>
		<IntlFormatted v-else :message-id="message">
			<template #content>
				<EventEntityList
					:entities="addonEntities"
					:single-line="true"
					:limit="contentLimit"
					entity-text-weight="semibold"
				/>
			</template>
			<template #files>
				<EventEntityList :entities="fileNames ?? []" :single-line="true" :limit="1" />
			</template>
		</IntlFormatted>
	</BaseEvent>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import { defineMessages, type MessageDescriptor, useVIntl } from '../../../../composables/i18n'
import IntlFormatted from '../../../base/IntlFormatted.vue'
import BaseEvent from './BaseEvent.vue'
import EventEntityList from './EventEntityList.vue'
import type { AuditAddonEventItem, EventEntity } from './types'

const props = defineProps<{
	kind: string
	addons?: AuditAddonEventItem[]
	fileNames?: EventEntity[]
}>()

const { formatMessage } = useVIntl()

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
	deletedLabel: {
		id: 'servers.audit-log.event.addon-deleted-label',
		defaultMessage: 'Deleted content',
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
const isDeleted = computed(() => props.kind === 'deleted')
const shouldShowSingleItem = computed(() =>
	['added', 'disabled', 'enabled', 'updated'].includes(props.kind),
)
const contentLimit = computed(() => (shouldShowSingleItem.value ? 1 : undefined))
</script>
