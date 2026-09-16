<template>
	<section v-if="project" class="flex flex-wrap gap-1" :aria-label="formatMessage(messages.tags)">
		<span
			v-for="category in project.categories"
			:key="category"
			class="rounded-full border border-solid border-brand px-2 py-0.5 text-xs text-brand"
			>{{ formatCategory(formatMessage, category) }}</span
		>
		<span
			v-for="category in project.additional_categories"
			:key="category"
			class="rounded-full bg-button-bg px-2 py-0.5 text-xs"
			>{{ formatCategory(formatMessage, category) }}</span
		>
	</section>
</template>

<script setup lang="ts">
import { formatCategory, useVIntl } from '@modrinth/ui'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import { projectReviewMessages as messages } from '../../messages'

const { project } = injectProjectReviewPageContext()
const { formatMessage } = useVIntl()
</script>
