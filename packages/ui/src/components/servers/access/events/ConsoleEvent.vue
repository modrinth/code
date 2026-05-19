<template>
	<BaseEvent>
		<IntlFormatted :message-id="messages.commandExecuted">
			<template #command>
				<span
					v-tooltip="command"
					class="inline-block max-w-full truncate align-bottom font-mono text-contrast"
				>
					{{ command }}
				</span>
			</template>
		</IntlFormatted>
	</BaseEvent>
</template>

<script setup lang="ts">
import { defineMessages } from '../../../../composables/i18n'
import IntlFormatted from '../../../base/IntlFormatted.vue'
import BaseEvent from './BaseEvent.vue'

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
