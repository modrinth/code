<template>
	<ProjectReview />
</template>

<script setup lang="ts">
import { useVIntl } from '@modrinth/ui'

import ProjectReview from '~/components/project-review/index.vue'
import { projectReviewMessages } from '~/components/project-review/messages'
import {
	createProjectReviewPageContext,
	provideProjectReviewPageContext,
} from '~/providers/project-review'

const { formatMessage } = useVIntl()
provideProjectReviewPageContext(createProjectReviewPageContext())

definePageMeta({
	layout: 'empty',
	middleware: ['auth', 'staff'],
})

useHead({
	htmlAttrs: { style: 'scrollbar-gutter: auto; overflow: hidden;' },
	bodyAttrs: { style: 'overflow: hidden;' },
	title: () => `${formatMessage(projectReviewMessages.title)} - Modrinth`,
})
</script>
