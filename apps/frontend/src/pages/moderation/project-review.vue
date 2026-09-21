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
const { confirmLeaveModal } = provideProjectReviewPageContext(createProjectReviewPageContext())

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
