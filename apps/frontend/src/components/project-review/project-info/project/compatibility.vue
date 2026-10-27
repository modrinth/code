<template>
	<Section heading="Compatibility" :target="{ kind: 'compatibility' }">
		<dl class="m-0 grid grid-cols-[max-content_minmax(0,1fr)] gap-x-3 gap-y-2.5">
			<dt class="text-sm font-medium text-secondary">Game versions</dt>
			<dd class="m-0 flex flex-wrap items-start gap-1">
				<TagItem
					v-for="version in gameVersions"
					:key="version"
					class="!border-surface-4 !bg-surface-3 !text-secondary"
					>{{ version }}</TagItem
				>
				<span v-if="compatibilityError">Unavailable</span>
				<span v-else-if="gameVersions.length === 0">No versions</span>
			</dd>
			<dt class="text-sm font-medium text-secondary">Platforms</dt>
			<dd class="m-0 flex flex-wrap items-start gap-1">
				<TagItem
					v-for="loader in project?.loaders"
					:key="loader"
					:style="`--_color: var(--color-platform-${loader})`"
				>
					<component
						:is="getLoaderIcon(loader)"
						v-if="loader !== 'mrpack' && getLoaderIcon(loader)"
						aria-hidden="true"
					/>
					{{ loaderName(loader) }}
				</TagItem>
				<span v-if="!project?.loaders.length">No platforms</span>
			</dd>
			<template v-if="requiresEnvironmentInfo(project?.project_types ?? [])">
				<dt class="text-sm font-medium text-secondary">Environments</dt>
				<dd class="m-0 flex flex-wrap items-start gap-1">
					<TagItem
						v-for="environment in project?.environment"
						:key="environment"
						class="max-w-full !text-wrap !border-surface-4 !bg-surface-3 !text-secondary"
						>{{ environmentLabels[environment] }}</TagItem
					>
					<span v-if="!project?.environment?.length">No environments</span>
				</dd>
			</template>
		</dl>
	</Section>
</template>

<script setup lang="ts">
import { getLoaderIcon } from '@modrinth/assets'
import { requiresEnvironmentInfo } from '@modrinth/moderation'
import { injectTags, TagItem } from '@modrinth/ui'
import { formatVersionsForDisplay } from '@modrinth/utils'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import Section from '../section.vue'
import { loaderName } from '../tag-labels'

const environmentLabels = {
	client_only: 'Client-side only',
	server_only: 'Server-side only, works in singleplayer too',
	singleplayer_only: 'Singleplayer only',
	dedicated_server_only: 'Server-side only',
	client_and_server: 'Client and server, required on both',
	client_only_server_optional: 'Client and server, optional on server',
	server_only_client_optional: 'Client and server, optional on client',
	client_or_server: 'Client and server, optional on both',
	client_or_server_prefers_both: 'Client and server, best when installed on both',
	unknown: 'Unknown environment',
}

const { project, projectV2, compatibilityError } = injectProjectReviewPageContext()

const tags = injectTags(null)
const gameVersions = computed(() => {
	const versions = projectV2.value?.game_versions ?? []
	return tags?.gameVersions.value?.length
		? formatVersionsForDisplay(versions, tags.gameVersions.value)
		: versions
})
</script>
