<template>
	<Admonition
		type="info"
		:header="formatMessage(messages.sharedInstanceRestoredHeader)"
		dismissible
		@dismiss="emit('dismiss')"
	>
		{{ formatMessage(messages.sharedInstanceRestoredBody) }}
	</Admonition>
</template>

<script setup lang="ts">
import { Admonition, useVIntl } from '@modrinth/ui'

import { instanceAdmonitionsMessages as messages } from './messages'

const emit = defineEmits<{
	dismiss: []
}>()

const { formatMessage } = useVIntl()
</script>
