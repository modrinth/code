<template>
	<div class="flex h-full min-h-0 flex-col gap-4 overflow-hidden">
		<ReviewPanel
			mode="inline"
			:target="{ kind: 'description' }"
			:label="formatMessage(messages.description)"
			:disabled="isLoading || !!error"
			class="mb-2"
		/>
		<div class="min-h-0 flex-1 overflow-auto">
			<p v-if="!selection" class="m-0 text-secondary">
				{{ formatMessage(messages.empty) }}
			</p>
			<p v-else-if="isLoading" role="status" class="m-0">
				{{ formatMessage(messages.loading) }}
			</p>
			<div v-else-if="error" role="alert">
				<p class="m-0">{{ formatMessage(messages.loadError) }}</p>
				<Button @click="refresh">{{ formatMessage(messages.retry) }}</Button>
			</div>
			<ProjectPageDescription
				v-else-if="project?.description?.trim()"
				class="text-sm leading-tight"
				:description="project.description"
			/>
			<p v-else class="m-0 text-secondary">
				{{ formatMessage(messages.emptyDescription) }}
			</p>
		</div>
	</div>
</template>

<script setup lang="ts">
import { Button, ProjectPageDescription, useVIntl } from '@modrinth/ui'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import { projectReviewMessages as messages } from '../messages'
import ReviewPanel from '../review-panel/index.vue'

const { formatMessage } = useVIntl()
const { selection, project, isLoading, error, refresh } = injectProjectReviewPageContext()
</script>
