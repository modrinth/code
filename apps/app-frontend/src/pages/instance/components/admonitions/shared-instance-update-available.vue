<template>
	<Admonition
		type="info"
		inline-actions
		:header="formatMessage(messages.sharedInstanceUpdateAvailableHeader)"
	>
		{{ formatMessage(messages.sharedInstanceUpdateAvailableBody, { name: instanceName }) }}
		<template #actions>
			<Button type="colored" color="blue" size="lg" @click="emit('review', $event)">
				<DownloadIcon aria-hidden="true" />
				{{ formatMessage(messages.sharedInstanceReviewUpdateButton) }}
			</Button>
		</template>
	</Admonition>
</template>

<script setup lang="ts">
import { DownloadIcon } from '@modrinth/assets'
import { Admonition, Button, useVIntl } from '@modrinth/ui'

import { instanceAdmonitionsMessages as messages } from './messages'

defineProps<{
	instanceName: string
}>()

const emit = defineEmits<{
	review: [event: MouseEvent]
}>()

const { formatMessage } = useVIntl()
</script>
