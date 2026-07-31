<template>
	<Admonition
		type="info"
		inline-actions
		:header="formatMessage(messages.sharedInstanceUpdateAvailableHeader)"
	>
		{{ formatMessage(messages.sharedInstanceUpdateAvailableBody, { name: instanceName }) }}
		<template #actions>
			<Button color="blue" type="colored" class="!h-10" @click="emit('review', $event)">
					<DownloadIcon aria-hidden="true" />
					{{ formatMessage(messages.sharedInstanceReviewUpdateButton) }}
				</Button>
		</template>
	</Admonition>
</template>

<script setup lang="ts">
import { DownloadIcon } from '@modrinth/assets'
import Button from '@modrinth/ui/src/components/base/buttons/Button.vue'
import { Admonition, useVIntl } from '@modrinth/ui'

import { instanceAdmonitionsMessages as messages } from './instance-admonitions-messages'

defineProps<{
	instanceName: string
}>()

const emit = defineEmits<{
	review: [event: MouseEvent]
}>()

const { formatMessage } = useVIntl()
</script>
