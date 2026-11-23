<script setup lang="ts">
import { BoxIcon, PlusIcon } from '@modrinth/assets'
import { Button, defineMessages, EmptyState, useVIntl } from '@modrinth/ui'

import { versionActionMessages } from './messages'

const emit = defineEmits<{ create: [] }>()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'project.settings.versions.empty.title',
		defaultMessage: 'No versions created',
	},
	description: {
		id: 'project.settings.versions.empty.description',
		defaultMessage: 'Create your first project version.',
	},
})
</script>

<template>
	<EmptyState
		:icon="BoxIcon"
		:heading="formatMessage(messages.title)"
		:description="formatMessage(messages.description)"
	>
		<template #actions>
			<Button type="colored" color="green" size="lg" @click="emit('create')">
				<PlusIcon /> {{ formatMessage(versionActionMessages.create) }}
			</Button>
		</template>
	</EmptyState>
</template>
