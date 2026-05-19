<template>
	<BaseEvent>
		<span class="text-secondary">{{ formatMessage(messages.unknownEvent) }}</span>
		<span class="font-mono text-secondary"> {{ rawAction }}</span>
	</BaseEvent>
</template>

<script setup lang="ts">
import { defineMessages, useVIntl } from '../../../../composables/i18n'
import BaseEvent from './BaseEvent.vue'

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
