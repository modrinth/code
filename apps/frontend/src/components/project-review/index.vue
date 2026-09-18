<template>
	<ClientOnly>
		<ProjectReviewLayout :tabs="visibleTabs">
			<template #left><ProjectInfo /></template>
			<template #right><Conversation :key="projectId" /></template>
			<template #footer><QueueBar /></template>
			<template #description><Description :key="projectId" /></template>
			<template #gallery><Gallery :key="projectId" /></template>
			<template #disclosures><Disclosures :key="projectId" /></template>
			<template #permissions><Permissions :key="projectId" /></template>
			<template #versions><Versions :key="projectId" /></template>
			<template #history><History :key="projectId" /></template>
			<template #tech-review><TechReview :key="projectId" /></template>
		</ProjectReviewLayout>
		<StageProvider v-if="project" :key="project.id" :project="project" />
		<template #fallback>
			<p class="m-0 p-4 text-secondary" role="status">
				{{ formatMessage(projectReviewMessages.loading) }}
			</p>
		</template>
	</ClientOnly>
</template>

<script setup lang="ts">
import { useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'
import {
	createReviewSession,
	provideReviewSession,
} from '~/providers/project-review/review-session'
import { createReviewStages, provideReviewStages } from '~/providers/project-review/review-stages'
import StageProvider from '~/providers/project-review/stage-provider'

import Conversation from './conversation.vue'
import Description from './description/index.vue'
import Disclosures from './disclosures/index.vue'
import Gallery from './gallery/index.vue'
import History from './history/index.vue'
import ProjectReviewLayout from './layout/index.client.vue'
import { projectReviewTabs } from './layout/types'
import { projectReviewMessages } from './messages'
import Permissions from './permissions/index.vue'
import ProjectInfo from './project-info/index.vue'
import QueueBar from './queue-bar.vue'
import { createReviewContext, provideReviewContext } from './review-panel/context'
import TechReview from './tech-review/index.vue'
import Versions from './versions/index.vue'

const { formatMessage } = useVIntl()
const { projectId, project } = injectProjectReviewPageContext()
const visibleTabs = computed(() =>
	projectReviewTabs.filter(
		(tab) => tab !== 'permissions' || project.value?.project_types.includes('modpack'),
	),
)
const reviewProjectId = computed(() => project.value?.id)
const stages = provideReviewStages(createReviewStages(reviewProjectId))
provideReviewSession(createReviewSession())
provideReviewContext(createReviewContext(reviewProjectId, (target) => !!stages.resolve(target)))
</script>
