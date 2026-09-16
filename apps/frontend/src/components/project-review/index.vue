<template>
	<ClientOnly>
		<ProjectReviewWorkspace>
			<template #left><ProjectInfo /></template>
			<template #footer><QueueBar /></template>
			<template #description>
				<div v-if="project" class="markdown-body" v-html="renderString(project.description)" />
				<p v-else>{{ formatMessage(projectReviewMessages.empty) }}</p>
			</template>
		</ProjectReviewWorkspace>
		<template #fallback>
			<p class="p-4 text-secondary" role="status">
				{{ formatMessage(projectReviewMessages.loading) }}
			</p>
		</template>
	</ClientOnly>
</template>

<script setup lang="ts">
import { useVIntl } from '@modrinth/ui'
import { renderString } from '@modrinth/utils'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import { projectReviewMessages } from './messages'
import ProjectInfo from './project-info/index.vue'
import QueueBar from './workspace/queue-bar.vue'
import ProjectReviewWorkspace from './workspace/workspace.client.vue'

const { formatMessage } = useVIntl()
const { project } = injectProjectReviewPageContext()
</script>
