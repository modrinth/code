<template>
	<Section :heading="formatMessage(messages.serverDetails)">
		<dl v-if="project" class="m-0 grid grid-cols-[max-content_minmax(0,1fr)] gap-x-3 gap-y-2.5">
			<dt>{{ formatMessage(messages.serverAddress) }}</dt>
			<dd class="m-0 break-all text-primary">
				{{ project.minecraft_java_server?.address || formatMessage(messages.noServerAddress) }}
			</dd>
			<dt>{{ formatMessage(messages.serverRegion) }}</dt>
			<dd class="m-0 text-primary">{{ region || '—' }}</dd>
			<dt>{{ formatMessage(messages.serverLanguages) }}</dt>
			<dd class="m-0 flex flex-wrap items-start gap-1">
				<TagItem
					v-for="language in languages"
					:key="language.code"
					class="!border-surface-4 !bg-surface-3 !text-secondary"
				>
					{{ language.name }}
				</TagItem>
				<span v-if="languages.length === 0">{{ formatMessage(messages.noServerLanguages) }}</span>
			</dd>

			<dt>{{ formatMessage(messages.serverContent) }}</dt>
			<dd class="m-0 min-w-0">
				<TagItem
					v-if="content?.kind === 'vanilla'"
					class="!border-surface-4 !bg-surface-3 !text-secondary"
				>
					{{ formatMessage(messages.vanilla) }}
				</TagItem>
				<TagItem v-else-if="isMrpack" class="!border-surface-4 !bg-surface-3 !text-secondary">
					{{ formatMessage(messages.mrpack) }}
				</TagItem>
				<TagItem
					v-else-if="content?.kind === 'modpack'"
					class="!border-surface-4 !bg-surface-3 !text-secondary"
				>
					{{ formatMessage(messages.modpack) }}
				</TagItem>
				<span v-else>—</span>
			</dd>
			<template v-if="content?.kind === 'vanilla'">
				<dt>{{ formatMessage(messages.gameVersions) }}</dt>
				<dd class="m-0 flex flex-wrap items-start gap-1">
					<TagItem
						v-if="content.recommended_game_version"
						class="!border-surface-4 !bg-surface-3 !text-secondary"
					>
						{{ content.recommended_game_version }}
						{{ formatMessage(messages.recommended) }}
					</TagItem>
					<TagItem
						v-for="version in supportedVersions"
						:key="version"
						class="!border-surface-4 !bg-surface-3 !text-secondary"
					>
						{{ version }}
					</TagItem>
					<span v-if="!content.recommended_game_version && supportedVersions.length === 0">—</span>
				</dd>
			</template>
			<dd v-else-if="isPublishedModpack" class="col-span-2 m-0 min-w-0">
				<div
					class="flex min-w-0 items-center gap-2 rounded-lg border border-solid border-surface-4 bg-surface-2 p-2"
				>
					<Avatar
						:src="content?.project_icon"
						:alt="modpackName"
						:tint-by="modpackName"
						size="2.5rem"
						no-shadow
					/>
					<div class="flex min-w-0 flex-col gap-0.5">
						<NuxtLink
							v-if="modpackProjectId"
							:to="`/modpack/${modpackProjectId}`"
							target="_blank"
							rel="noopener noreferrer"
							class="truncate font-medium text-primary hover:underline"
						>
							{{ modpackName }}
						</NuxtLink>
						<span v-else class="truncate font-medium text-primary">{{ modpackName }}</span>
						<NuxtLink
							v-if="modpackProjectId && content"
							:to="`/modpack/${modpackProjectId}/version/${modpackVersionId}`"
							target="_blank"
							rel="noopener noreferrer"
							class="truncate text-xs hover:underline"
						>
							{{ formatMessage(messages.modpackVersion, { version: modpackVersionLabel }) }}
						</NuxtLink>
						<span v-else class="truncate text-xs">
							{{ formatMessage(messages.modpackVersion, { version: modpackVersionLabel }) }}
						</span>
					</div>
				</div>
			</dd>
		</dl>
	</Section>
</template>

<script setup lang="ts">
import {
	Avatar,
	injectModrinthClient,
	injectTags,
	SERVER_LANGUAGES,
	SERVER_REGIONS,
	TagItem,
	useVIntl,
} from '@modrinth/ui'
import { formatVersionsForDisplay } from '@modrinth/utils'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import { projectReviewMessages as messages } from '../../messages'
import Section from '../section.vue'

const { project } = injectProjectReviewPageContext()
const client = injectModrinthClient()
const { formatMessage } = useVIntl()
const tags = injectTags(null)
const content = computed(() => project.value?.minecraft_java_server?.content)
const modpackVersionId = computed(() =>
	content.value?.kind === 'modpack' ? content.value.version_id : '',
)
const modpackVersionQuery = useQuery(
	computed(() => ({
		queryKey: ['version', 'v3', modpackVersionId.value],
		queryFn: () => client.labrinth.versions_v3.getVersion(modpackVersionId.value),
		enabled:
			!!modpackVersionId.value &&
			!(content.value?.kind === 'modpack' && content.value.project_id === project.value?.id),
	})),
)
const modpackProjectId = computed(() => {
	if (content.value?.kind !== 'modpack') return null
	return content.value.project_id ?? modpackVersionQuery.data.value?.project_id ?? null
})
const isMrpack = computed(
	() => content.value?.kind === 'modpack' && modpackProjectId.value === project.value?.id,
)
const isPublishedModpack = computed(() => content.value?.kind === 'modpack' && !isMrpack.value)
const modpackName = computed(() => {
	if (content.value?.kind !== 'modpack') return ''
	return content.value.project_name || modpackProjectId.value || '—'
})
const modpackVersionLabel = computed(() => {
	if (content.value?.kind !== 'modpack') return ''
	return modpackVersionQuery.data.value?.version_number || content.value.version_id
})
const languages = computed(() =>
	(project.value?.minecraft_server?.languages ?? []).map((code) => {
		const message = SERVER_LANGUAGES[code as keyof typeof SERVER_LANGUAGES]
		return { code, name: message ? formatMessage(message) : code }
	}),
)
const region = computed(() => {
	const code = project.value?.minecraft_server?.region
	if (!code) return null
	const message = SERVER_REGIONS[code as keyof typeof SERVER_REGIONS]
	return message ? formatMessage(message) : code
})
const supportedVersions = computed(() => {
	const vanilla = content.value
	if (vanilla?.kind !== 'vanilla') return []
	const versions = vanilla.supported_game_versions.filter(
		(version) => version !== vanilla.recommended_game_version,
	)
	return tags?.gameVersions.value?.length
		? formatVersionsForDisplay(versions, tags.gameVersions.value)
		: versions
})
</script>
