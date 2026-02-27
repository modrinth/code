<script setup lang="ts">
import { SignalIcon } from '@modrinth/assets'

import { defineMessage, useVIntl } from '../../../composables'
import { TagItem } from '../../base'

defineProps<{
	ping?: number
	statusOnline?: boolean
}>()

const pingMessage = defineMessage({
	id: 'project.server.ping.ms',
	defaultMessage: '{ping} ms',
})

const { formatMessage } = useVIntl()
</script>
<template>
	<TagItem
		v-if="ping || statusOnline"
		class="border !border-solid border-brand bg-brand-highlight !font-medium w-max"
		style="--_color: var(--color-brand)"
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
