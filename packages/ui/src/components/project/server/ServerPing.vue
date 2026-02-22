<script setup lang="ts">
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
		class="border !border-solid border-brand bg-brand-highlight !font-medium"
		style="--_color: var(--color-brand)"
	>
		{{ formatMessage(pingMessage, { ping }) }}
	</TagItem>
</template>
