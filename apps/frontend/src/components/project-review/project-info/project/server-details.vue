<template>
	<Section heading="Server details" :target="{ kind: 'server' }">
		<template #right>
			<EditButton section="Server details" @click="editModal?.show()" />
		</template>
		<EditModal :key="project?.id" ref="editModal" />
		<dl v-if="project" class="m-0 grid grid-cols-[max-content_minmax(0,1fr)] gap-x-3 gap-y-2.5">
			<dt class="font-medium">Address</dt>
			<dd class="m-0 flex items-center gap-1.5 break-all text-primary">
				<CopyCode
					v-if="project.minecraft_java_server?.address"
					:text="project.minecraft_java_server.address"
					class="!-my-0.5"
				/>
				<span v-else>No address.</span>
				<TriangleAlertIcon
					v-if="isTemporaryHost"
					v-tooltip="'This address uses a temporary server host.'"
					aria-label="This address uses a temporary server host."
					class="size-4 shrink-0 text-orange"
				/>
			</dd>
			<dt class="font-medium">Region</dt>
			<dd class="m-0 text-primary">
				<TagItem v-if="region" class="!border-surface-4 !bg-surface-3 !text-secondary">
					{{ region }}
				</TagItem>
				<span v-else>—</span>
			</dd>
			<dt class="font-medium">Languages</dt>
			<dd class="m-0 flex flex-wrap items-start gap-1">
				<TagItem
					v-for="language in languages"
					:key="language.code"
					class="!border-surface-4 !bg-surface-3 !text-secondary"
				>
					{{ language.name }}
				</TagItem>
				<span v-if="languages.length === 0">No languages.</span>
			</dd>

			<dt class="font-medium">Content</dt>
			<dd class="m-0 min-w-0">
				<TagItem
					v-if="content?.kind === 'vanilla'"
					class="!border-surface-4 !bg-surface-3 !text-secondary"
				>
					Vanilla
				</TagItem>
				<TagItem v-else-if="isMrpack" class="!border-surface-4 !bg-surface-3 !text-secondary">
					.mrpack
				</TagItem>
				<TagItem
					v-else-if="content?.kind === 'modpack'"
					class="!border-surface-4 !bg-surface-3 !text-secondary"
				>
					Modpack
				</TagItem>
				<span v-else>—</span>
			</dd>
			<template v-if="content?.kind === 'vanilla'">
				<dt class="font-medium">Game versions</dt>
				<dd class="m-0 flex flex-wrap items-start gap-1">
					<TagItem
						v-if="content.recommended_game_version"
						class="!border-surface-4 !bg-surface-3 !text-secondary"
					>
						{{ content.recommended_game_version }}
						(Recommended)
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
						<div class="flex min-w-0 flex-wrap items-center gap-x-1.5 text-xs">
							<NuxtLink
								v-if="modpackProjectId && content"
								:to="`/modpack/${modpackProjectId}/version/${modpackVersionId}`"
								target="_blank"
								rel="noopener noreferrer"
								class="truncate hover:underline"
							>
								{{ `Version ${modpackVersionLabel}` }}
							</NuxtLink>
							<span v-else class="truncate">
								{{ `Version ${modpackVersionLabel}` }}
							</span>
							<template v-if="modpackAuthorId">
								<BulletDivider aria-hidden="true" />
								<NuxtLink
									:to="`/user/${modpackAuthorId}`"
									target="_blank"
									rel="noopener noreferrer"
									class="truncate hover:underline"
								>
									{{ modpackAuthorQuery.data.value?.username ?? modpackAuthorId }}
								</NuxtLink>
							</template>
						</div>
					</div>
				</div>
			</dd>
		</dl>
	</Section>
</template>

<script setup lang="ts">
import { TriangleAlertIcon } from '@modrinth/assets'
import { isTempHostAddress } from '@modrinth/moderation'
import {
	Avatar,
	BulletDivider,
	CopyCode,
	injectModrinthClient,
	injectTags,
	TagItem,
} from '@modrinth/ui'
import { formatVersionsForDisplay } from '@modrinth/utils'
import { useQuery } from '@tanstack/vue-query'
import { computed, useTemplateRef } from 'vue'

import { versionQueryOptions } from '~/composables/queries/version'
import { injectProjectReviewPageContext } from '~/providers/project-review'

import EditButton from '../edit/button.vue'
import EditModal from '../edit/server-details.vue'
import Section from '../section.vue'
import { SERVER_LANGUAGES, SERVER_REGIONS } from '../server-labels'

const { project } = injectProjectReviewPageContext()
const editModal = useTemplateRef<InstanceType<typeof EditModal>>('editModal')
const client = injectModrinthClient()

const tags = injectTags(null)
const isTemporaryHost = computed(() =>
	isTempHostAddress(project.value?.minecraft_java_server?.address ?? ''),
)
const content = computed(() => project.value?.minecraft_java_server?.content)
const modpackVersionId = computed(() =>
	content.value?.kind === 'modpack' ? content.value.version_id : '',
)
const modpackVersionQuery = useQuery(
	computed(() => ({
		...versionQueryOptions.v3(modpackVersionId.value, client),
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
const modpackAuthorId = computed(() =>
	isPublishedModpack.value ? (modpackVersionQuery.data.value?.author_id ?? '') : '',
)
const modpackAuthorQuery = useQuery(
	computed(() => ({
		queryKey: ['user', modpackAuthorId.value] as const,
		queryFn: () => client.labrinth.users_v3.get(modpackAuthorId.value),
		enabled: !!modpackAuthorId.value,
	})),
)
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
		return { code, name: message ? message : code }
	}),
)
const region = computed(() => {
	const code = project.value?.minecraft_server?.region
	if (!code) return null
	const message = SERVER_REGIONS[code as keyof typeof SERVER_REGIONS]
	return message ? message : code
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
