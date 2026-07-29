<template>
	<BaseEvent>
		<IntlFormatted :message-id="message">
			<template #port>
				<span class="font-mono font-medium text-contrast">{{ port }}</span>
			</template>
		</IntlFormatted>
	</BaseEvent>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import { defineMessages } from '../../../../composables/i18n'
import IntlFormatted from '../../../base/IntlFormatted.vue'
import BaseEvent from './BaseEvent.vue'

const props = defineProps<{
	kind: 'added' | 'removed'
	port: number
}>()

const messages = defineMessages({
	added: {
		id: 'servers.audit-log.event.port-allocation-added',
		defaultMessage: 'Added port allocation <port></port>',
	},
	removed: {
		id: 'servers.audit-log.event.port-allocation-removed',
		defaultMessage: 'Removed port allocation <port></port>',
	},
})

const message = computed(() => (props.kind === 'added' ? messages.added : messages.removed))
</script>
