<template>
	<div v-if="project" class="flex flex-col gap-1">
		<div class="flex items-center gap-3">
			<span
				class="min-w-0 flex-1 truncate"
				:title="project.license.name || projectV2?.license.name || project.license.id"
				>{{ project.license.name || projectV2?.license.name || project.license.id }}</span
			>
		</div>
		<div>
			<a
				v-if="url"
				:href="url"
				target="_blank"
				rel="noopener noreferrer"
				class="mt-1 flex min-w-0 items-center gap-1 font-mono text-xs !transition-colors hover:text-contrast"
			>
				<span class="truncate" :title="url">{{ url.replace(/^https?:\/\//, '') }}</span>
				<ExternalIcon class="size-3 shrink-0" />
			</a>
			<span v-else class="text-secondary">{{ formatMessage(messages.noLicenseUrl) }}</span>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ExternalIcon } from '@modrinth/assets'
import { useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'
import { reviewExternalUrl } from '~/providers/project-review/project-links'

import { projectReviewMessages as messages } from '../../messages'

const { project, projectV2 } = injectProjectReviewPageContext()
const { formatMessage } = useVIntl()
const url = computed(() => reviewExternalUrl(project.value?.license.url))
</script>
