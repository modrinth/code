<template>
	<dl class="m-0 grid grid-cols-[max-content_minmax(0,1fr)] gap-x-3 gap-y-2.5">
		<dt class="text-sm text-secondary">{{ formatMessage(messages.gameVersions) }}</dt>
		<dd class="m-0 flex flex-wrap items-start gap-1">
			<TagItem
				v-for="version in gameVersions"
				:key="version"
				class="!border-surface-4 !bg-surface-3 !text-secondary"
				>{{ version }}</TagItem
			>
			<span v-if="compatibilityError">{{ formatMessage(messages.unavailable) }}</span>
			<span v-else-if="gameVersions.length === 0">{{ formatMessage(messages.emptyVersions) }}</span>
		</dd>
		<dt class="text-sm text-secondary">{{ formatMessage(messages.platforms) }}</dt>
		<dd class="m-0 flex flex-wrap items-start gap-1">
			<TagItem
				v-for="loader in project?.loaders"
				:key="loader"
				class="!border-surface-4 !bg-surface-3 !text-secondary"
				>{{ formatLoader(formatMessage, loader) }}</TagItem
			>
			<span v-if="!project?.loaders.length">{{ formatMessage(messages.emptyPlatforms) }}</span>
		</dd>
		<template v-if="requiresEnvironmentInfo(project?.project_types ?? [])">
			<dt class="text-sm text-secondary">{{ formatMessage(messages.environments) }}</dt>
			<dd class="m-0 flex flex-wrap items-start gap-1">
				<TagItem
					v-for="environment in project?.environment"
					:key="environment"
					class="max-w-full !text-wrap !border-surface-4 !bg-surface-3 !text-secondary"
					>{{ formatMessage(ENVIRONMENTS_COPY[environment].title) }}</TagItem
				>
				<span v-if="!project?.environment?.length">{{
					formatMessage(messages.emptyEnvironments)
				}}</span>
			</dd>
		</template>
	</dl>
</template>

<script setup lang="ts">
import { requiresEnvironmentInfo } from '@modrinth/moderation'
import { ENVIRONMENTS_COPY, formatLoader, injectTags, TagItem, useVIntl } from '@modrinth/ui'
import { formatVersionsForDisplay } from '@modrinth/utils'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import { projectReviewMessages as messages } from '../../messages'

const { project, projectV2, compatibilityError } = injectProjectReviewPageContext()
const { formatMessage } = useVIntl()
const tags = injectTags(null)
const gameVersions = computed(() => {
	const versions = projectV2.value?.game_versions ?? []
	return tags?.gameVersions.value?.length
		? formatVersionsForDisplay(versions, tags.gameVersions.value)
		: versions
})
</script>
