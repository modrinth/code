<template>
	<BaseEvent>
		<IntlFormatted :message-id="message">
			<template #modpack>
				<EventEntityLink v-if="modpack" :entity="modpack" />
			</template>
			<template #version>
				<EventInlineText :text="versionLabel" class="align-middle font-mono text-secondary" />
			</template>
		</IntlFormatted>
	</BaseEvent>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import { defineMessages } from '../../../../composables/i18n'
import IntlFormatted from '../../../base/IntlFormatted.vue'
import BaseEvent from './BaseEvent.vue'
import EventEntityLink from './EventEntityLink.vue'
import EventInlineText from './EventInlineText.vue'
import type { EventEntity } from './types'

const props = defineProps<{
	kind: 'changed' | 'unlinked'
	modpack?: EventEntity | null
	versionLabel?: string | null
}>()

const messages = defineMessages({
	changed: {
		id: 'servers.audit-log.event.modpack-changed',
		defaultMessage: 'Changed modpack',
	},
	changedToModpack: {
		id: 'servers.audit-log.event.modpack-changed-to-modpack',
		defaultMessage: 'Changed modpack to <modpack></modpack>',
	},
	changedToVersion: {
		id: 'servers.audit-log.event.modpack-changed-to-version',
		defaultMessage: 'Changed modpack to version <version></version>',
	},
	unlinked: {
		id: 'servers.audit-log.event.modpack-unlinked',
		defaultMessage: 'Unlinked modpack',
	},
	unlinkedModpack: {
		id: 'servers.audit-log.event.modpack-unlinked-modpack',
		defaultMessage: 'Unlinked modpack <modpack></modpack>',
	},
	unlinkedVersion: {
		id: 'servers.audit-log.event.modpack-unlinked-version',
		defaultMessage: 'Unlinked modpack version <version></version>',
	},
})

const message = computed(() => {
	if (props.kind === 'unlinked') {
		if (props.modpack) return messages.unlinkedModpack
		return props.versionLabel ? messages.unlinkedVersion : messages.unlinked
	}

	if (props.modpack) return messages.changedToModpack
	return props.versionLabel ? messages.changedToVersion : messages.changed
})

const versionLabel = computed(() => props.versionLabel ?? '')
</script>
