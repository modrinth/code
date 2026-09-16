<template>
	<p v-if="!selection" class="text-secondary">
		{{ formatMessage(messages.empty) }}
	</p>
	<p v-else-if="isLoading" role="status">
		{{ formatMessage(messages.loading) }}
	</p>
	<div v-else-if="error" role="alert">
		<p>{{ formatMessage(messages.loadError) }}</p>
		<Button @click="refresh">{{ formatMessage(messages.retry) }}</Button>
	</div>
	<ProjectPageDescription
		v-else-if="project?.description?.trim()"
		:description="project.description"
	/>
	<p v-else class="text-secondary">
		{{ formatMessage(messages.emptyDescription) }}
	</p>
</template>

<script setup lang="ts">
import { Button, ProjectPageDescription, useVIntl } from '@modrinth/ui'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import { projectReviewMessages as messages } from '../messages'

const { formatMessage } = useVIntl()
const { selection, project, isLoading, error, refresh } = injectProjectReviewPageContext()
</script>
