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
	<TagItem :style="`--_color: var(--color-green)`">
		{{ formatMessage(pingMessage, { ping }) }}
	</TagItem>
</template>
