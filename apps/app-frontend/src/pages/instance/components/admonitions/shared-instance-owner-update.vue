<template>
	<Admonition
		type="info"
		inline-actions
		:header="formatMessage(messages.sharedInstanceOwnerUpdateHeader)"
	>
		{{ formatMessage(messages.sharedInstanceOwnerUpdateBody) }}
		<template #actions>
			<Button type="colored" color="blue" size="lg" @click="emit('review-update', $event)">
				<DownloadIcon aria-hidden="true" />
				{{ formatMessage(messages.sharedInstanceOwnerUpdateButton) }}
			</Button>
		</template>
	</Admonition>
</template>

<script setup lang="ts">
import { DownloadIcon } from '@modrinth/assets'
import { Admonition, Button, useVIntl } from '@modrinth/ui'

import { instanceAdmonitionsMessages as messages } from './messages'

const emit = defineEmits<{
	'review-update': [event: MouseEvent]
}>()

const { formatMessage } = useVIntl()
</script>
