<template>
	<dl class="m-0 grid grid-cols-[max-content_minmax(0,1fr)] gap-x-3 gap-y-2.5">
		<dt class="text-sm text-secondary">{{ formatMessage(messages.gameVersions) }}</dt>
		<dd class="m-0 flex flex-wrap items-start gap-1">
			<TagItem
				v-for="version in projectV2?.game_versions"
				:key="version"
				class="!border-surface-4 !bg-surface-3 !text-secondary"
				>{{ version }}</TagItem
			>
			<span v-if="compatibilityError">{{ formatMessage(messages.unavailable) }}</span>
		</dd>
		<dt class="text-sm text-secondary">{{ formatMessage(messages.platforms) }}</dt>
		<dd class="m-0 flex flex-wrap items-start gap-1">
			<TagItem
				v-for="loader in project?.loaders"
				:key="loader"
				class="!border-surface-4 !bg-surface-3 !text-secondary"
				>{{ formatLoader(formatMessage, loader) }}</TagItem
			>
		</dd>
		<dt class="text-sm text-secondary">{{ formatMessage(messages.environments) }}</dt>
		<dd class="m-0 flex flex-wrap items-start gap-1">
			<TagItem
				v-for="environment in project?.environment"
				:key="environment"
				class="max-w-full !text-wrap !border-surface-4 !bg-surface-3 !text-secondary"
				>{{ formatMessage(ENVIRONMENTS_COPY[environment].title) }}</TagItem
			>
		</dd>
	</dl>
</template>

<script setup lang="ts">
import { ENVIRONMENTS_COPY, formatLoader, TagItem, useVIntl } from '@modrinth/ui'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import { projectReviewMessages as messages } from '../../messages'

const { project, projectV2, compatibilityError } = injectProjectReviewPageContext()
const { formatMessage } = useVIntl()
</script>
