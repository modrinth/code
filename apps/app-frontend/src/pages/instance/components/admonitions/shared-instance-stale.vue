<template>
	<Admonition
		type="info"
		inline-actions
		:header="formatMessage(messages.sharedInstanceChangesHeader)"
	>
		{{ formatMessage(messages.sharedInstanceChangesBody) }}
		<template #actions>
			<InstancePushButtons :instance="instance" @published="emit('published')" />
		</template>
	</Admonition>
</template>

<script setup lang="ts">
import { Admonition, useVIntl } from '@modrinth/ui'

import InstancePushButtons from '@/components/ui/linked-servers/InstancePushButtons.vue'
import type { GameInstance } from '@/helpers/types'

import { instanceAdmonitionsMessages as messages } from './messages'

defineProps<{
	instance: GameInstance
}>()

const emit = defineEmits<{
	published: []
}>()

const { formatMessage } = useVIntl()
</script>
