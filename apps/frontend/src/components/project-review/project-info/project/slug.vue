<template>
	<div v-if="project" class="min-w-0">
		<div class="break-all font-mono text-xs">
			<span>/{{ projectV2?.project_type ?? project.project_types[0] }}/ </span>
			<span class="text-contrast">{{ project.slug ?? project.id }}</span>
		</div>
		<p class="mb-0 mt-2 flex items-center gap-1.5 text-xs">
			<CheckIcon v-if="matchesName" class="size-3 shrink-0" />
			{{ formatMessage(matchesName ? messages.matchesName : messages.customSlug) }}
		</p>
	</div>
</template>

<script setup lang="ts">
import { CheckIcon } from '@modrinth/assets'
import { useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import { generateUrlSlug } from '~/composables/project-slug-suggestions'
import { injectProjectReviewPageContext } from '~/providers/project-review'

import { projectReviewMessages as messages } from '../../messages'

const { project, projectV2 } = injectProjectReviewPageContext()
const { formatMessage } = useVIntl()
const matchesName = computed(
	() => project.value && generateUrlSlug(project.value.name) === project.value.slug,
)
</script>
