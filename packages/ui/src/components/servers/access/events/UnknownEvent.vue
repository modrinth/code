<template>
	<BaseEvent>
		<span class="mr-1 text-secondary">{{ formatMessage(messages.unknownEvent) }}</span>
		<span
			ref="rawActionRef"
			v-tooltip="truncatedTooltip(rawActionRef, rawAction)"
			class="min-w-0 truncate font-mono text-secondary"
		>
			{{ rawAction }}
		</span>
	</BaseEvent>
</template>

<script setup lang="ts">
import { ref } from 'vue'

import { truncatedTooltip } from '#ui/utils/truncate'

import { defineMessages, useVIntl } from '../../../../composables/i18n'
import BaseEvent from './BaseEvent.vue'

defineProps<{
	rawAction: string
}>()

const { formatMessage } = useVIntl()
const rawActionRef = ref<HTMLElement | null>(null)

const messages = defineMessages({
	unknownEvent: {
		id: 'servers.audit-log.event.unknown',
		defaultMessage: 'Unknown event',
	},
})
</script>
