<template>
	<ProjectReview />
	<ConfirmLeaveModal ref="confirmLeaveModal" />
</template>

<script setup lang="ts">
import { ConfirmLeaveModal, useVIntl } from '@modrinth/ui'

import ProjectReview from '~/components/project-review/index.vue'
import { projectReviewMessages } from '~/components/project-review/messages'
import {
	createProjectReviewPageContext,
	provideProjectReviewPageContext,
} from '~/providers/project-review'

const { formatMessage } = useVIntl()
const { confirmLeaveModal, project } = provideProjectReviewPageContext(
	createProjectReviewPageContext(),
)

definePageMeta({
	layout: 'empty',
	middleware: ['auth', 'staff'],
})

useHead({
	htmlAttrs: { style: 'scrollbar-gutter: auto; overflow: hidden;' },
	bodyAttrs: { style: 'overflow: hidden;' },
	title: () =>
		project.value?.name
			? `${project.value.name} - ${formatMessage(projectReviewMessages.title)}`
			: formatMessage(projectReviewMessages.title),
})
useFavicon('review')
</script>
