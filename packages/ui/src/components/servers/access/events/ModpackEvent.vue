<template>
	<BaseEvent>
		<IntlFormatted :message-id="message">
			<template #version>
				<span class="font-mono text-secondary">{{ shortVersionId }}</span>
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
	newVersionId?: string | null
}>()

const messages = defineMessages({
	changed: {
		id: 'servers.audit-log.event.modpack-changed',
		defaultMessage: 'Changed modpack',
	},
	changedToVersion: {
		id: 'servers.audit-log.event.modpack-changed-to-version',
		defaultMessage: 'Changed modpack to version <version></version>',
	},
})

const message = computed(() =>
	props.newVersionId ? messages.changedToVersion : messages.changed,
)

const shortVersionId = computed(() =>
	props.newVersionId && props.newVersionId.length > 12
		? props.newVersionId.slice(0, 8)
		: props.newVersionId,
)
</script>
