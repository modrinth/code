<script setup lang="ts">
import { SignalIcon } from '@modrinth/assets'
import { computed } from 'vue'

import { defineMessage, useVIntl } from '../../../composables'
import { TagItem } from '../../base'

const props = defineProps<{
	ping?: number
	statusOnline?: boolean
}>()

const pingMessage = defineMessage({
	id: 'project.server.ping.ms',
	defaultMessage: '{ping} ms',
})

const { formatMessage } = useVIntl()

const pingClass = computed(() => {
	if (props.ping === undefined) {
		return 'border-brand bg-highlight-green text-brand'
	}
	if (props.ping < 100) {
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
			{{ formatMessage(pingMessage, { ping }) }}
		</template>
		<template v-else>
			<SignalIcon />
			Online
		</template>
	</TagItem>
	<TagItem
		v-else
		v-tooltip="'Server is offline'"
		class="border !border-solid border-red bg-highlight-red text-red smart-clickable:allow-pointer-events w-max"
	>
		<SignalIcon />
		Offline
	</TagItem>
</template>
