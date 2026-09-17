<template>
	<div v-if="project" class="flex flex-wrap gap-1">
		<TagItem
			v-for="category in project.categories"
			:key="category"
			class="!border-brand !bg-transparent !text-brand opacity-70"
			>{{ formatCategory(formatMessage, category) }}</TagItem
		>
		<TagItem v-for="category in project.additional_categories" :key="category">
			{{ formatCategory(formatMessage, category) }}
		</TagItem>
	</div>
</template>

<script setup lang="ts">
import { formatCategory, TagItem, useVIntl } from '@modrinth/ui'

import { injectProjectReviewPageContext } from '~/providers/project-review'

const { project } = injectProjectReviewPageContext()
const { formatMessage } = useVIntl()
</script>
