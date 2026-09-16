<template>
	<ClientOnly>
		<ProjectReviewLayout>
			<template #left><ProjectInfo /></template>
			<template #footer><QueueBar /></template>
			<template #description><Description /></template>
		</ProjectReviewLayout>
		<template #fallback>
			<p class="p-4 text-secondary" role="status">
				{{ formatMessage(projectReviewMessages.loading) }}
			</p>
		</template>
	</ClientOnly>
</template>

<script setup lang="ts">
import { useVIntl } from '@modrinth/ui'

import Description from './description.vue'
import ProjectReviewLayout from './layout/index.client.vue'
import { projectReviewMessages } from './messages'
import ProjectInfo from './project-info/index.vue'
import QueueBar from './queue-bar.vue'

const { formatMessage } = useVIntl()
</script>
