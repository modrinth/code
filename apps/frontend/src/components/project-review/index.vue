<template>
	<ClientOnly>
		<ProjectReviewLayout>
			<template #left><ProjectInfo /></template>
			<template #footer><QueueBar /></template>
			<template #description><Description :key="projectId" /></template>
			<template #gallery><Gallery :key="projectId" /></template>
			<template #disclosures><Disclosures :key="projectId" /></template>
			<template #versions><Versions :key="projectId" /></template>
			<template #history><History :key="projectId" /></template>
			<template #tech-review><TechReview :key="projectId" /></template>
		</ProjectReviewLayout>
		<template #fallback>
			<p class="m-0 p-4 text-secondary" role="status">
				{{ formatMessage(projectReviewMessages.loading) }}
			</p>
		</template>
	</ClientOnly>
</template>

<script setup lang="ts">
import { useVIntl } from '@modrinth/ui'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import Description from './description/index.vue'
import Disclosures from './disclosures/index.vue'
import Gallery from './gallery/index.vue'
import History from './history/index.vue'
import ProjectReviewLayout from './layout/index.client.vue'
import { projectReviewMessages } from './messages'
import ProjectInfo from './project-info/index.vue'
import QueueBar from './queue-bar.vue'
import TechReview from './tech-review/index.vue'
import Versions from './versions/index.vue'

const { formatMessage } = useVIntl()
const { projectId } = injectProjectReviewPageContext()
</script>
