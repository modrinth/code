<template>
	<BaseEvent>
		<span class="mr-1 text-secondary">{{ formatMessage(messages.unknownEvent) }}</span>
		<EventInlineText :text="rawAction" class="font-mono text-secondary" />
	</BaseEvent>
</template>

<script setup lang="ts">
import { defineMessages, useVIntl } from '../../../../composables/i18n'
import BaseEvent from './BaseEvent.vue'
import EventInlineText from './EventInlineText.vue'

defineProps<{
	rawAction: string
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	unknownEvent: {
		id: 'servers.audit-log.event.unknown',
		defaultMessage: 'Unknown event',
	},
})
</script>
