<template>
	<BaseEvent>
		{{ formatMessage(messages.powerEvent) }}
	</BaseEvent>
</template>

<script setup lang="ts">
import { defineMessages, useVIntl } from '../../../../composables/i18n'
import BaseEvent from './BaseEvent.vue'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	powerEvent: {
		id: 'servers.audit-log.event.power-event',
		defaultMessage: 'Server power event',
	},
})
</script>
