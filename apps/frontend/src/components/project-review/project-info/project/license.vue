<template>
	<div v-if="project" class="flex flex-col gap-1">
		<div class="flex items-start gap-1">
			<span v-tooltip="project.license.id" class="flex min-h-6 min-w-0 items-center">
				{{ licenseName }}
			</span>
			<TagItem v-if="isCustomLicense" class="!border-surface-4 !bg-surface-3 !text-secondary">
				Custom
			</TagItem>
		</div>
		<div>
			<a
				v-if="url"
				v-tooltip="url"
				:href="url"
				target="_blank"
				rel="noopener noreferrer"
				class="mt-1 flex w-fit min-w-0 max-w-full items-center gap-1 font-mono text-xs !transition-colors hover:text-contrast"
			>
				<span class="min-w-0 truncate">{{ url.replace(/^https?:\/\//, '') }}</span>
				<ExternalIcon class="size-3 shrink-0" />
			</a>
			<span v-else class="text-secondary">{{ formatMessage(messages.noLicenseUrl) }}</span>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ExternalIcon } from '@modrinth/assets'
import { TagItem, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'
import { reviewExternalUrl } from '~/providers/project-review/project-links'

import { projectReviewMessages as messages } from '../../messages'

const { project, projectV2 } = injectProjectReviewPageContext()
const { formatMessage } = useVIntl()
const licenseName = computed(() => {
	const license = project.value?.license
	const name = license?.name || projectV2.value?.license.name
	if (name) return name

	const id = license?.id ?? ''
	return id.startsWith('LicenseRef-') ? id.replace(/^LicenseRef-/, '').replaceAll('-', ' ') : id
})
const isCustomLicense = computed(() => {
	const id = project.value?.license.id
	return (
		!!id &&
		id.startsWith('LicenseRef-') &&
		id !== 'LicenseRef-All-Rights-Reserved' &&
		id !== 'LicenseRef-Unknown' &&
		id !== 'LicenseRef-NOASSERTION'
	)
})
const url = computed(() => reviewExternalUrl(project.value?.license.url))
</script>
