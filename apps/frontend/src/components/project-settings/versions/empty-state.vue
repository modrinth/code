<script setup lang="ts">
import { BoxIcon, PlusIcon } from '@modrinth/assets'
import { Button, EmptyState, useVIntl } from '@modrinth/ui'

import { messages, versionActionMessages } from './messages'

const emit = defineEmits<{ create: [] }>()
const { formatMessage } = useVIntl()
</script>

<template>
	<EmptyState
		:icon="BoxIcon"
		:heading="formatMessage(messages.emptyTitle)"
		:description="formatMessage(messages.emptyDescription)"
	>
		<template #actions>
			<Button type="colored" color="green" size="lg" @click="emit('create')">
				<PlusIcon /> {{ formatMessage(versionActionMessages.create) }}
			</Button>
		</template>
	</EmptyState>
</template>
