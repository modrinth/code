<template>
	<aside
		class="flex shrink-0 flex-col overflow-hidden border-0 border-r border-solid border-divider bg-surface-1"
		:class="collapsed ? 'w-12 transition-[width] duration-200 ease-in-out' : ''"
		:style="collapsed ? undefined : { width: `${width}px` }"
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
				v-for="id in railTabs"
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
			<div class="flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto p-2">
				<ChecklistStageButtons stage-id="title-slug">
					<div class="grid grid-cols-[auto_1fr] items-center gap-x-3 gap-y-1.5">
						<span class="text-xs font-semibold uppercase tracking-wide text-secondary">Title</span>
						<span class="min-w-0 text-primary">{{ project.title || project.name || '—' }}</span>

						<span class="text-xs font-semibold uppercase tracking-wide text-secondary">Slug</span>
						<code class="min-w-0 break-all text-primary">{{ project.slug || '—' }}</code>
					</div>
				</ChecklistStageButtons>
				<div class="my-1 h-[2px] w-full flex-shrink-0 bg-divider" />
				<dl class="m-0 grid grid-cols-2 gap-x-3 gap-y-1 text-sm">
					<dt class="items-center justify-center flex">
						<Avatar
							:src="project.icon_url"
							:raw-src="project.raw_icon_url"
							:tint-by="project.id"
							size="105px"
						/>
					</dt>
					<dd class="m-0 text-contrast h-full ">
						<div class="flex flex-col h-full justify-evenly">
							<div class="flex flex-col">
								<span class="text-secondary">Applying for:</span>
								<ProjectStatusBadge :status="requestedStatus" />
							</div>

							<div class="flex flex-col">
								<span class="text-secondary">Status:</span>
								<ProjectStatusBadge :status="currentStatus" />
							</div>
						</div>
					</dd>

					<template v-if="project.approved">
						<dt class="text-secondary">
							<CalendarIcon aria-hidden="true" /> Published
						</dt>
						<dd class="m-0 text-contrast" v-tooltip="formatDateTime(project.approved)">{{ publishedRelative }}</dd>
					</template>
					<template v-else>
						<dt class="text-secondary">
							<CalendarIcon aria-hidden="true" /> Created
						</dt>
						<dd class="m-0 text-contrast" v-tooltip="props.project.published ? formatDateTime(props.project.published) : 'unknown'">{{ createdRelative }}</dd>
					</template>

					<dt class="text-secondary">
						<ScaleIcon aria-hidden="true" /> Submitted
					</dt>
					<dd class="m-0 text-contrast" v-tooltip="formatDateTime(project.queued ?? project.published)">{{ submittedRelative }}</dd>

					<template v-if="project.versions.length > 0 && project.updated">
						<dt class="text-secondary">
							<VersionIcon aria-hidden="true" /> Updated
						</dt>
						<dd class="m-0 text-contrast" v-tooltip="project.updated ? formatDateTime(project.updated) : 'unknown'">{{ updatedRelative }}</dd>
					</template>

				</dl>
				<div class="my-1 h-[2px] w-full flex-shrink-0 bg-divider" />
				<ProjectSidebarCreators
					:organization="organization"
					:members="members"
					:loading="creatorsLoading"
					:org-link="(slug) => `/organization/${slug}`"
					:user-link="(username) => `/user/${username}`"
					:disable-header="true"
					class="flex-card-reduced"
				/>
				<div class="my-1 h-[2px] w-full flex-shrink-0 bg-divider" />
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
					:disable-header="true"
					class="flex-card-reduced"
				/>
				<div v-if="isServerProject" class="my-1 h-[2px] w-full flex-shrink-0 bg-divider" />

				<ChecklistStageButtons stage-id="links" variant="inline" inline-node-mode="full" />

				<div class="my-1 h-[2px] w-full flex-shrink-0 bg-divider" />

				<ChecklistStageButtons stage-id="tags" variant="inline">
					<ProjectSidebarTags :project="project" :disable-header="true" class="flex-card-reduced" />
				</ChecklistStageButtons>

				<div class="my-1 h-[2px] w-full flex-shrink-0 bg-divider" />

				<ChecklistStageButtons stage-id="metadata" variant="inline">
					<ProjectSidebarCompatibility
						:project="project"
						:tags="tags"
						:project-v3="projectV3"
						:compact-mode="true"
					/>
				</ChecklistStageButtons>

				<div class="my-1 h-[2px] w-full flex-shrink-0 bg-divider" />

				<ChecklistStageButtons stage-id="license" variant="inline" />
				<ProjectSidebarDetails
					:project="project"
					link-target="_blank"
					:hide-license="isServerProject"
					:show-followers="isServerProject"
					class="flex-card-reduced"
					:disable-header="true"
					:remove-time-info="true"
				/>
			</div>
		</template>
	</aside>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {CalendarIcon, ChevronRightIcon, ScaleIcon, VersionIcon} from '@modrinth/assets'
import {
	Avatar, commonMessages, ProjectSidebarCompatibility,
	ProjectSidebarCreators,
	ProjectSidebarDetails,
	ProjectSidebarServerInfo,
	ProjectSidebarTags,
	ProjectStatusBadge, useFormatDateTime,
	useRelativeTime, useVIntl,
} from '@modrinth/ui'
import { computed } from 'vue'

import { useModerationQueue } from '~/services/moderation/queue.ts'
import { REVIEW_TAB_ORDER, useModerationReviewLayout } from '~/services/moderation/review-layout'

import ChecklistStageButtons from './ChecklistStageButtons.vue'
import { reviewTab, selectableReviewTabs } from './review-tabs'
import {capitalizeString} from "@modrinth/utils";

const { formatMessage, locale } = useVIntl()

const formatDateTime = useFormatDateTime({
	timeStyle: 'short',
	dateStyle: 'long',
})

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
		width?: number
	}>(),
	{
		width: 304,
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
const formatRelativeTime = useRelativeTime()
const tags = useGeneratedState()

const collapsed = computed(() => layout.sidebarCollapsed.value)

const railTabs = computed(() =>
	selectableReviewTabs(
		REVIEW_TAB_ORDER,
		(props.projectV3?.project_types ?? []).includes('modpack'),
	),
)

const submittedRelative = computed(() => {
	const date = (props.project.queued ?? props.project.published) as string | undefined
	return date ? formatRelativeTime(date) : 'unknown'
})
const createdRelative = computed(() =>
	props.project.published ? formatRelativeTime(props.project.published as string) : 'unknown',
)
const publishedRelative = computed(() =>
	props.project.approved ? formatRelativeTime(props.project.approved) : 'unknown',
)
const updatedRelative = computed(() =>
	props.project.updated ? formatRelativeTime(props.project.updated as string) : 'unknown',
)

const requestedStatus = computed(() => props.project.requested_status ?? 'unknown')

const currentStatus = computed(() => props.project.status ?? 'unknown')
</script>

<style scoped>
/* Halved-padding variant of .flex-card for the moderation sidebar. */
.flex-card-reduced {
	display: flex;
	flex-direction: column;
	gap: var(--gap-6);
	padding: 0;
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
