<script setup lang="ts">
import { SignalIcon } from '@modrinth/assets'
import { computed } from 'vue'

import { defineMessages, useVIntl } from '../../../composables'
import { TagItem } from '../../base'

const props = defineProps<{
	ping?: number
	statusOnline?: boolean
}>()

const messages = defineMessages({
	ping: {
		id: 'project.server.ping.ms',
		defaultMessage: '{ping} ms',
	},
	online: {
		id: 'project.server.status.online',
		defaultMessage: 'Online',
	},
	offline: {
		id: 'project.server.status.offline',
		defaultMessage: 'Offline',
	},
	offlineTooltip: {
		id: 'project.server.status.offline.tooltip',
		defaultMessage: 'Server is offline',
	},
})

const { formatMessage } = useVIntl()

const pingClass = computed(() => {
	if (props.ping === undefined) {
		return 'border-brand bg-highlight-green text-brand'
	}
	if (props.ping < 150) {
		return 'border-brand bg-highlight-green text-brand'
	}
	if (props.ping < 250) {
		return 'border-brand-orange bg-highlight-orange text-orange'
	}
	return 'border-red bg-highlight-red text-red'
})
</script>
<template>
	<TagItem
		v-if="ping || statusOnline"
		class="border !border-solid !font-medium w-max"
		:class="pingClass"
	>
		<template v-if="ping !== undefined">
			{{ formatMessage(messages.ping, { ping }) }}
		</template>
		<template v-else>
			<SignalIcon />
			{{ formatMessage(messages.online) }}
		</template>
	</TagItem>
	<TagItem
		v-else
		v-tooltip="formatMessage(messages.offlineTooltip)"
		class="border !border-solid border-red bg-highlight-red text-red smart-clickable:allow-pointer-events w-max"
	>
		<SignalIcon />
		{{ formatMessage(messages.offline) }}
	</TagItem>
</template>
