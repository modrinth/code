<template>
	<aside
		class="flex shrink-0 flex-col overflow-hidden border-0 border-r border-solid border-divider bg-surface-1 transition-[width] duration-200 ease-in-out"
		:class="collapsed ? 'w-12' : 'w-[19rem]'"
	>
		<!-- Collapsed rail -->
		<template v-if="collapsed">
			<button
				v-tooltip="'Expand sidebar'"
				class="flex h-12 w-12 items-center justify-center text-secondary hover:bg-button-bg hover:text-contrast"
				aria-label="Expand sidebar"
				@click="layout.toggleSidebar()"
			>
				<ChevronRightIcon class="size-5" />
			</button>
			<div class="mt-1 h-px w-full bg-divider" />
			<button
				v-for="id in REVIEW_TAB_ORDER"
				:key="id"
				v-tooltip="reviewTab(id).label"
				class="flex h-11 w-12 items-center justify-center"
				:class="
					layout.dockOf(id)
						? 'text-contrast'
						: 'text-secondary hover:bg-button-bg hover:text-contrast'
				"
				:aria-label="`Open ${reviewTab(id).label}`"
				@click="layout.openTab(id, { dock: 'main' })"
			>
				<component :is="reviewTab(id).icon" class="size-5" />
			</button>
		</template>

		<!-- Expanded -->
		<template v-else>
			<div class="flex items-start gap-2 p-3">
				<Avatar
					:src="project.icon_url"
					:raw-src="project.raw_icon_url"
					:tint-by="project.id"
					size="44px"
				/>
				<div class="min-w-0 flex-1">
					<a
						:href="projectPath"
						target="_blank"
						class="line-clamp-2 font-extrabold leading-tight text-contrast hover:underline"
						>{{ project.title }}</a
					>
					<div class="mt-1 flex flex-wrap items-center gap-1">
						<ProjectStatusBadge :status="projectV3.status" />
					</div>
				</div>
				<button
					v-tooltip="'Collapse sidebar'"
					class="shrink-0 rounded p-1 text-secondary hover:bg-button-bg hover:text-contrast"
					aria-label="Collapse sidebar"
					@click="layout.toggleSidebar()"
				>
					<ChevronLeftIcon class="size-5" />
				</button>
			</div>

			<dl
				class="m-0 grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 border-0 border-y border-solid border-divider px-3 py-2 text-sm"
			>
				<template v-if="queuePosition">
					<dt class="text-secondary">Queue</dt>
					<dd class="m-0 text-contrast">{{ queuePosition }}</dd>
				</template>
				<dt class="text-secondary">Submitted</dt>
				<dd class="m-0 text-contrast">{{ submittedRelative }}</dd>
				<dt class="text-secondary">Created</dt>
				<dd class="m-0 text-contrast">{{ createdRelative }}</dd>
				<template v-if="project.updated">
					<dt class="text-secondary">Updated</dt>
					<dd class="m-0 text-contrast">{{ updatedRelative }}</dd>
				</template>
			</dl>

			<div class="flex min-h-0 flex-1 flex-col gap-2 overflow-y-auto p-2">
				<ProjectSidebarCreators
					:organization="organization"
					:members="members"
					:loading="creatorsLoading"
					:org-link="(slug) => `/organization/${slug}`"
					:user-link="(username) => `/user/${username}`"
					class="flex-card-reduced"
				/>
				<ProjectSidebarServerInfo
					v-if="isServerProject"
					:loading="!serverDataLoaded"
					:project-v3="projectV3"
					:tags="tags"
					:required-content="serverRequiredContent"
					:recommended-version="serverRecommendedVersion"
					:supported-versions="serverSupportedVersions"
					:loaders="serverModpackLoaders"
					:status-online="projectV3?.minecraft_java_server?.ping?.data != null"
					class="flex-card-reduced"
				/>
				<ProjectSidebarCompatibility
					v-if="!isServerProject"
					:project="project"
					:tags="tags"
					:project-v3="projectV3"
					:compact-mode="true"
					class="flex-card-reduced"
				/>
				<ProjectSidebarLinks
					:project="project"
					:project-v3="projectV3"
					link-target="_blank"
					class="flex-card-reduced"
				/>
				<ProjectSidebarTags :project="project" class="flex-card-reduced" />
				<ProjectSidebarDetails
					:project="project"
					link-target="_blank"
					:hide-license="isServerProject"
					:show-followers="isServerProject"
					class="flex-card-reduced"
				/>
			</div>
		</template>
	</aside>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ChevronLeftIcon, ChevronRightIcon } from '@modrinth/assets'
import {
	Avatar,
	ProjectSidebarCompatibility,
	ProjectSidebarCreators,
	ProjectSidebarDetails,
	ProjectSidebarLinks,
	ProjectSidebarServerInfo,
	ProjectSidebarTags,
	ProjectStatusBadge,
	useRelativeTime,
} from '@modrinth/ui'
import { computed } from 'vue'

import { useModerationQueue } from '~/services/moderation/queue.ts'
import { REVIEW_TAB_ORDER, useModerationReviewLayout } from '~/services/moderation/review-layout'

import { reviewTab } from './review-tabs'

type LooseProject = Labrinth.Projects.v2.Project & Record<string, unknown>

const props = withDefaults(
	defineProps<{
		project: LooseProject
		projectV3: Labrinth.Projects.v3.Project
		organization?: Record<string, unknown> | null
		members?: unknown[]
		creatorsLoading?: boolean
		isServerProject?: boolean
		serverDataLoaded?: boolean
		serverRequiredContent?: unknown
		serverRecommendedVersion?: unknown
		serverSupportedVersions?: unknown[]
		serverModpackLoaders?: unknown[]
	}>(),
	{
		organization: null,
		members: () => [],
		creatorsLoading: false,
		isServerProject: false,
		serverDataLoaded: false,
		serverRequiredContent: null,
		serverRecommendedVersion: null,
		serverSupportedVersions: () => [],
		serverModpackLoaders: () => [],
	},
)

const layout = useModerationReviewLayout()
const moderationQueue = useModerationQueue()
const formatRelativeTime = useRelativeTime()
const tags = useGeneratedState()

const collapsed = computed(() => layout.sidebarCollapsed.value)

const projectPath = computed(
	() => `/${props.project.project_type}/${props.project.slug ?? props.project.id}`,
)

const submittedRelative = computed(() => {
	const date = (props.project.queued ?? props.project.published) as string | undefined
	return date ? formatRelativeTime(date) : 'unknown'
})
const createdRelative = computed(() =>
	props.project.published ? formatRelativeTime(props.project.published as string) : 'unknown',
)
const updatedRelative = computed(() =>
	props.project.updated ? formatRelativeTime(props.project.updated as string) : 'unknown',
)

const queuePosition = computed(() => {
	if (!moderationQueue.isQueueMode) return null
	const items = moderationQueue.currentQueue.items
	const index = items.indexOf(props.project.id)
	if (index < 0) return null
	return `${index + 1} / ${items.length}`
})
</script>

<style scoped>
/* Halved-padding variant of .flex-card for the moderation sidebar. */
.flex-card-reduced {
	display: flex;
	flex-direction: column;
	gap: var(--gap-6);
	padding: var(--gap-8);
	margin: 0;

	:deep(h2) {
		font-size: var(--text-16);
		font-weight: 600;
		color: var(--color-contrast);
		line-height: initial;
		margin: 0;
	}

	:deep(h3) {
		font-size: var(--text-14);
		font-weight: 600;
		color: var(--color-base);
		margin: 0;
	}

	:deep(section) {
		gap: var(--gap-4);
	}
}
</style>
