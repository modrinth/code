<template>
	<article
		class="flex min-w-0 cursor-pointer items-center gap-3 rounded-2xl border border-solid bg-surface-3 p-3"
		:class="selected ? 'border-brand ring-1 ring-brand' : 'hover:border-surface-6 border-surface-5'"
		@click="emit('select')"
	>
		<Avatar :src="result.project.icon_url" size="3.5rem" no-shadow class="rounded-xl" />
		<div class="flex min-w-0 flex-1 flex-col gap-2">
			<div class="flex min-w-0 items-center gap-3">
				<div class="flex min-w-0 flex-1 items-center gap-2">
					<NuxtLink
						:to="`/project/${encodeURIComponent(result.project.slug ?? result.project.id)}`"
						target="_blank"
						class="min-w-0 text-base font-semibold text-contrast hover:underline"
						@click.stop
					>
						<span
							ref="projectTitle"
							v-tooltip="truncatedTooltip(projectTitle, result.project.title)"
							class="block truncate"
						>
							{{ result.project.title }}
						</span>
					</NuxtLink>
					<Tooltip
						v-if="project && primaryProjectType"
						class="flex shrink-0"
						:disabled="project.project_types.length < 2"
					>
						<TagItem tabindex="0">
							<component :is="getProjectTypeIcon(primaryProjectType)" aria-hidden="true" />
							{{ formatProjectType(primaryProjectType, true) }}
							<span v-if="project.project_types.length > 1">({{ project.project_types.length }})</span>
						</TagItem>
						<template #popper>
							<div class="flex items-center gap-2">
								<TagItem v-for="type in project.project_types" :key="type">
									<component :is="getProjectTypeIcon(type)" aria-hidden="true" />
									{{ formatProjectType(type, true) }}
								</TagItem>
							</div>
						</template>
					</Tooltip>
					<TagItem v-if="project" v-tooltip="statusLabel" class="shrink-0">
						<component
							:is="getProjectStatusIcon(project.status)"
							:style="{ color: `var(--color-${getProjectStatusColor(project.status)})` }"
							aria-hidden="true"
						/>
						<template
							v-if="project.requested_status && project.requested_status !== project.status"
						>
							<RightArrowIcon class="text-secondary" aria-hidden="true" />
							<component
								:is="getProjectStatusIcon(project.requested_status)"
								:style="{
									color: `var(--color-${getProjectStatusColor(project.requested_status)})`,
								}"
								aria-hidden="true"
							/>
						</template>
						<span class="sr-only">{{ statusLabel }}</span>
					</TagItem>
				</div>
				<div class="flex shrink-0 items-center gap-2">
					<TagItem
						v-if="flagCount > 0"
						v-tooltip="'Show project flags'"
						class="shrink-0"
						type="button"
						:action="selectProject"
						:aria-label="`Show flags for ${result.project.title} (${flagCount})`"
						:aria-pressed="selected"
						:style="flagCount ? '--_color: var(--color-purple)' : undefined"
					>
						<TagCategoryFlagIcon aria-hidden="true" />
						{{ flagCount }}<span class="sr-only"> flags</span>
					</TagItem>
					<div @click.stop>
						<TeleportOverflowMenu
							:label="`Options for ${result.project.title}`"
							tooltip="Project options"
							:options="projectOptions"
							type="quiet"
							size="sm"
						>
							<MoreHorizontalIcon aria-hidden="true" />
							<template #modview>
								<ScaleIcon class="flag-run-modview-icon" aria-hidden="true" />
								Modview
							</template>
						</TeleportOverflowMenu>
					</div>
				</div>
			</div>
			<div class="flex flex-wrap items-center gap-x-3 gap-y-1 text-sm text-secondary">
				<NuxtLink
					v-if="owner"
					:to="`/${owner.kind}/${owner.id}`"
					target="_blank"
					class="inline-flex min-w-0 items-center gap-1.5 hover:underline"
					@click.stop
				>
					<Avatar :src="owner.icon" size="1.25rem" circle no-shadow />
					<span class="break-words">{{ owner.name }}</span>
				</NuxtLink>
				<span
					v-if="approvedCount !== undefined"
					v-tooltip="`${approvedCount} approved projects`"
					class="inline-flex items-center gap-1"
				>
					<CheckIcon class="size-4" aria-hidden="true" />
					{{ approvedCount }}<span class="sr-only"> approved projects</span>
				</span>
				<span
					v-if="externalCount !== undefined"
					v-tooltip="`${externalCount} external dependencies`"
					class="inline-flex items-center gap-1"
				>
					<LinkIcon class="size-4" aria-hidden="true" />
					{{ externalCount }}<span class="sr-only"> external dependencies</span>
				</span>
				<time
					v-if="projectDate"
					v-tooltip="`${project?.queued ? 'Submitted' : 'Created'} ${formatDate(projectDate, true)}`"
					:datetime="projectDate"
					class="ml-auto"
				>
					{{ formatRelativeTime(projectDate) }}
				</time>
			</div>
		</div>
	</article>
</template>

<script setup lang="ts">
import type { Labrinth, ProjectFlagRunResult } from '@modrinth/api-client'
import {
	CheckIcon,
	CopyIcon,
	LinkIcon,
	MoreHorizontalIcon,
	RightArrowIcon,
	ScaleIcon,
	TagCategoryFlagIcon,
} from '@modrinth/assets'
import {
	Avatar,
	type ButtonMenuOption,
	getProjectStatusColor,
	getProjectStatusIcon,
	getProjectTypeIcon,
	injectModrinthClient,
	injectNotificationManager,
	TagItem,
	TeleportOverflowMenu,
	Tooltip,
	truncatedTooltip,
	useRelativeTime,
} from '@modrinth/ui'
import { formatProjectStatus, formatProjectType, getPrimaryProjectType } from '@modrinth/utils'
import { useQuery } from '@tanstack/vue-query'

import { creatorProjectStatsQueryOptions } from '~/composables/creator-project-stats'

import { formatDate } from '../run-format'
import { useFlagErrorNotification } from '../use-flag-error-notification'

const props = defineProps<{
	result: ProjectFlagRunResult
	project?: Labrinth.Projects.v3.Project
	owner?: { id: string; name: string; icon?: string | null; kind: 'user' | 'organization' }
	flagCount: number
	selected: boolean
}>()
const emit = defineEmits<{ select: [] }>()
const projectTitle = useTemplateRef<HTMLElement>('projectTitle')
const primaryProjectType = computed(() =>
	props.project ? getPrimaryProjectType(props.project) : undefined,
)

function selectProject(event: MouseEvent) {
	event.stopPropagation()
	emit('select')
}

const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const projectOptions = computed<ButtonMenuOption[]>(() => [
	{
		id: 'modview',
		type: 'link',
		label: 'Modview',
		tone: 'orange',
		to: { path: '/moderation/project-review', query: { project: props.result.project.id } },
		target: '_blank',
		rel: 'noopener noreferrer',
	},
	{ id: 'copy-project-id', label: 'Copy project ID', icon: CopyIcon, action: copyProjectId },
])

async function copyProjectId() {
	try {
		await navigator.clipboard.writeText(props.result.project.id)
		addNotification({ title: 'Project ID copied', type: 'success' })
	} catch {
		addNotification({ title: 'Could not copy project ID', type: 'error' })
	}
}

const formatRelativeTime = useRelativeTime()
const projectDate = computed(() => props.project?.queued || props.project?.published)
const statusLabel = computed(() => {
	const project = props.project
	if (!project) return ''
	const current = formatProjectStatus(project.status)
	return project.requested_status && project.requested_status !== project.status
		? `${current} → ${formatProjectStatus(project.requested_status)}`
		: current
})
const { data: ownerProjects, error: ownerProjectsError } = useQuery(
	computed(() => {
		const id = props.owner?.id ?? ''
		const options =
			props.owner?.kind === 'organization'
				? creatorProjectStatsQueryOptions.organizationProjects(id, client)
				: creatorProjectStatsQueryOptions.memberProjects(id, client)
		return { ...options, enabled: Boolean(id) }
	}),
)
const approvedCount = computed(
	() => ownerProjects.value?.filter((project) => project.status === 'approved').length,
)
const { data: externalCount, error: externalCountError } = useQuery({
	queryKey: computed(
		() =>
			['moderation', 'project', props.result.project.id, 'external-dependencies-count'] as const,
	),
	queryFn: async ({ queryKey: [, , id] }) => {
		const versions = await client.labrinth.versions_v3.getProjectVersions(id, {
			apiVersion: 3,
			include_changelog: false,
		})
		return versions.reduce(
			(count, version) =>
				count + version.dependencies.filter((dependency) => dependency.file_name != null).length,
			0,
		)
	},
	enabled: computed(() => !!props.project),
	staleTime: 60_000,
})

useFlagErrorNotification(ownerProjectsError, 'Could not load owner project stats')
useFlagErrorNotification(externalCountError, 'Could not load project dependency stats')
</script>

<style scoped>
:global(.button-menu-item:has(.flag-run-modview-icon):is(:hover, :focus-visible)) {
	background-color: var(--color-orange-bg);
}
</style>
