<template>
	<BaseEvent>
		<IntlFormatted :message-id="messages.commandExecuted">
			<template #command>
				<EventInlineText :text="command" class="align-middle font-mono font-medium text-contrast" />
			</template>
		</IntlFormatted>
	</BaseEvent>
</template>

<script setup lang="ts">
import { defineMessages } from '../../../../composables/i18n'
import IntlFormatted from '../../../base/IntlFormatted.vue'
import BaseEvent from './BaseEvent.vue'
import EventInlineText from './EventInlineText.vue'

defineProps<{
	command: string
}>()

const messages = defineMessages({
	commandExecuted: {
		id: 'servers.audit-log.event.console-command-executed',
		defaultMessage: 'Ran console command <command></command>',
	},
})
</script>
