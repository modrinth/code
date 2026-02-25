<script setup lang="ts">
import { NoSignalIcon } from '../../../../../assets/generated-icons'
import { defineMessage, useVIntl } from '../../../composables'
import { TagItem } from '../../base'

defineProps<{
	ping: number
}>()

const pingMessage = defineMessage({
	id: 'project.server.ping.ms',
	defaultMessage: '{ping} ms',
})

const { formatMessage } = useVIntl()
</script>
<template>
	<TagItem
		v-if="ping"
		class="border !border-solid border-brand bg-brand-highlight !font-medium w-max"
		style="--_color: var(--color-brand)"
	>
		{{ formatMessage(pingMessage, { ping }) }}
	</TagItem>
	<TagItem
		v-else
		v-tooltip="'Server is offline'"
		class="border !border-solid border-surface-5 smart-clickable:allow-pointer-events w-max"
	>
		<NoSignalIcon />
	</TagItem>
</template>
