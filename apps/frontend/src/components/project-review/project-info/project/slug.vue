<template>
	<div v-if="project" class="min-w-0">
		<div class="break-all font-mono text-xs">
			<span>/{{ projectTypeForUrl }}/ </span>
			<span class="text-contrast">{{ project.slug ?? project.id }}</span>
		</div>
		<p class="m-0 flex items-center gap-1.5 text-xs">
			<CheckIcon v-if="matchesName" class="size-3 shrink-0" />
			{{ formatMessage(matchesName ? messages.matchesName : messages.customSlug) }}
		</p>
	</div>
</template>

<script setup lang="ts">
import { CheckIcon } from '@modrinth/assets'
import { generateUrlSlug } from '@modrinth/moderation/src/utils'
import { useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import { getProjectTypeForUrl } from '~/helpers/projects.js'
import { injectProjectReviewPageContext } from '~/providers/project-review'

import { projectReviewMessages as messages } from '../../messages'

const { project } = injectProjectReviewPageContext()
const { formatMessage } = useVIntl()
const projectTypeForUrl = computed(() => {
	if (project.value?.minecraft_server != null) return 'server'
	const type = project.value?.project_types[0] ?? 'mod'
	return getProjectTypeForUrl(type, project.value?.loaders ?? [])
})
const matchesName = computed(
	() => project.value && generateUrlSlug(project.value.name) === project.value.slug,
)
</script>
