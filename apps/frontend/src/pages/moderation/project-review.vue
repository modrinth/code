<template>
	<ProjectReview />
	<ConfirmLeaveModal ref="confirmLeaveModal" />
</template>

<script setup lang="ts">
import { ConfirmLeaveModal, useVIntl } from '@modrinth/ui'
import { onBeforeUnmount, onMounted } from 'vue'

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
	title: () =>
		project.value?.name
			? `${project.value.name} - ${formatMessage(projectReviewMessages.title)}`
			: formatMessage(projectReviewMessages.title),
})
useFavicon('review')

onMounted(() => {
	document.documentElement.classList.add('project-review-page')
	document.body.classList.add('project-review-page')
})

onBeforeUnmount(() => {
	document.documentElement.classList.remove('project-review-page')
	document.body.classList.remove('project-review-page')
})
</script>

<style scoped>
:global(html.project-review-page) {
	scrollbar-gutter: auto;
	overflow: hidden;
}

:global(body.project-review-page) {
	overflow: hidden;
}
</style>
