<template>
	<div class="review-details min-w-0 text-sm text-secondary">
		<p v-if="isLoading" role="status">{{ formatMessage(messages.loading) }}</p>
		<div v-else-if="error" role="alert">
			<p>{{ formatMessage(messages.loadError) }}</p>
			<Button @click="refresh">{{ formatMessage(messages.retry) }}</Button>
		</div>
		<p v-else-if="!project">{{ formatMessage(messages.empty) }}</p>
		<template v-else>
			<div class="flex flex-col gap-6 px-2 pb-6">
				<Slug />
				<Icon />
				<Summary />
			</div>
			<License class="review-section" />
			<Tags class="review-section" />
			<div class="review-section !px-0"><Links /></div>
			<Compatibility class="review-section" />
			<Members class="review-section" />
			<Details class="review-section" />
		</template>
	</div>
</template>

<script setup lang="ts">
import { Button, useVIntl } from '@modrinth/ui'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import { projectReviewMessages as messages } from '../messages'
import Compatibility from './project/compatibility.vue'
import Details from './project/details.vue'
import Icon from './project/icon.vue'
import License from './project/license.vue'
import Links from './project/links.vue'
import Members from './project/members.vue'
import Slug from './project/slug.vue'
import Summary from './project/summary.vue'
import Tags from './project/tags.vue'

const { project, isLoading, error, refresh } = injectProjectReviewPageContext()
const { formatMessage } = useVIntl()
</script>

<style scoped>
.review-section {
	border-top: 1px solid var(--color-divider);
	padding: 1rem 0.5rem;
}
.review-details :deep(h2) {
	margin: 0 0 0.75rem;
	font-size: 0.6875rem;
	font-weight: 700;
	letter-spacing: 0.1em;
	text-transform: uppercase;
}
.review-details :deep(.review-badge) {
	border-radius: 0.25rem;
	background: var(--color-button-bg);
	padding: 0.125rem 0.375rem;
	font-size: 0.6875rem;
}
</style>
