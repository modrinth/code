<template>
	<Section heading="Details">
		<template #right>
			<EditButton section="Details" @click="editModal?.show()" />
		</template>
		<EditModal :key="project?.id" ref="editModal" />
		<dl v-if="project" class="m-0 grid grid-cols-[5.5rem_minmax(0,1fr)] gap-x-3 gap-y-2">
			<dt class="font-medium">Project</dt>
			<dd class="m-0 flex min-w-0 items-center gap-1">
				<Tooltip class="flex shrink-0">
					<template #popper>
						<ProjectStatusBadge
							:status="project.status"
							:style="{ color: `var(--color-${getProjectStatusColor(project.status)})` }"
						/>
					</template>
					<component
						:is="getProjectStatusIcon(project.status)"
						class="size-4 shrink-0"
						:style="{ color: `var(--color-${getProjectStatusColor(project.status)})` }"
						aria-hidden="true"
					/>
				</Tooltip>
				<NuxtLink
					:to="projectUrl"
					target="_blank"
					rel="noopener noreferrer"
					class="min-w-0 break-words font-semibold text-secondary hover:underline"
				>
					{{ project.name }}
				</NuxtLink>
			</dd>
			<template v-if="project.status === 'processing' && project.requested_status">
				<dt class="font-medium">Requesting</dt>
				<dd class="m-0">
					<ProjectStatusBadge
						:status="project.requested_status"
						:style="{ color: `var(--color-${getProjectStatusColor(project.requested_status)})` }"
					/>
				</dd>
			</template>
			<dt class="font-medium">Type</dt>
			<dd class="m-0 flex flex-col gap-1">
				<div
					v-for="type in projectTypes"
					:key="type.id"
					class="flex items-center gap-1 font-semibold text-secondary"
				>
					<component :is="type.icon" class="size-4 shrink-0" aria-hidden="true" />
					<span
						:class="{
							underline: projectTypes.length > 1 && type.id === primaryProjectType,
						}"
						>{{ projectTypeLabels[type.id] ?? 'Project' }}</span
					>
				</div>
			</dd>
			<dt class="font-medium">Created</dt>
			<dd class="m-0">
				<time v-tooltip="formatDateTime(project.published)" :datetime="project.published">{{
					relativeTime(project.published)
				}}</time>
			</dd>
			<dt class="font-medium">Updated</dt>
			<dd class="m-0">
				<time v-tooltip="formatDateTime(project.updated)" :datetime="project.updated">{{
					relativeTime(project.updated)
				}}</time>
			</dd>
			<template v-if="project.queued">
				<dt class="font-medium">Submitted</dt>
				<dd class="m-0 flex flex-wrap items-center gap-x-1.5">
					<time v-tooltip="formatDateTime(project.queued)" :datetime="project.queued">{{
						relativeTime(project.queued)
					}}</time>
					<template v-if="submissionCount">
						<BulletDivider aria-hidden="true" />
						<span>{{
							submissionCount === 1
								? `${submissionCount} submission`
								: `${submissionCount} submissions`
						}}</span>
					</template>
				</dd>
			</template>
			<dt class="font-medium">Downloads</dt>
			<dd class="m-0">{{ formatNumber(project.downloads) }}</dd>
			<dt class="font-medium">Monetization</dt>
			<dd class="m-0">
				{{
					monetizationStatus === 'monetized'
						? 'Enabled'
						: monetizationStatus === 'demonetized'
							? 'Disabled'
							: monetizationStatus === 'forceDemonetized'
								? 'Force disabled'
								: 'Unknown'
				}}
			</dd>

			<dt class="mt-0.5 self-start font-medium">Actions</dt>
			<dd class="m-0 flex flex-wrap items-center gap-2">
				<Button
					v-tooltip="`Copy: ${project.id}`"
					type="outlined"
					class="!text-primary"
					size="sm"
					aria-label="Copy ID"
					@click="copyProjectId(project.id)"
				>
					Project ID
					<CheckIcon v-if="projectIdCopied" aria-hidden="true" />
					<CopyIcon v-else aria-hidden="true" />
				</Button>
				<Button
					v-tooltip="`Copy: ${permalink}`"
					type="outlined"
					class="!text-primary"
					size="xs"
					aria-label="Copy permanent link"
					@click="copyPermalink(permalink)"
				>
					Perma link
					<CheckIcon v-if="permalinkCopied" aria-hidden="true" />
					<CopyIcon v-else aria-hidden="true" />
				</Button>
				<ButtonLink
					:to="analyticsUrl"
					:disabled="!analyticsOwnerId"
					target="_blank"
					rel="noopener noreferrer"
					type="outlined"
					class="!text-primary"
					size="xs"
				>
					Analytics
					<ExternalIcon aria-hidden="true" />
				</ButtonLink>
			</dd>
		</dl>
	</Section>
</template>

<script setup lang="ts">
import { CheckIcon, CopyIcon, ExternalIcon, ServerIcon } from '@modrinth/assets'
import {
	BulletDivider,
	Button,
	ButtonLink,
	getProjectStatusColor,
	getProjectStatusIcon,
	getProjectTypeIcon,
	ProjectStatusBadge,
	Tooltip,
} from '@modrinth/ui'
import { getPrimaryProjectType } from '@modrinth/utils'
import { useClipboard, useMagicKeys } from '@vueuse/core'
import { computed, useTemplateRef } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import { createDateFormatter, createRelativeTimeFormatter, formatNumber } from '../../format'
import EditButton from '../edit/button.vue'
import EditModal from '../edit/details.vue'
import Section from '../section.vue'
import { projectTypeLabels } from '../type-labels'

const { project, projectV2, submissionCount, members, organizationMembers } =
	injectProjectReviewPageContext()
const config = useRuntimeConfig()
const { shift: shiftHeld } = useMagicKeys()
const permalink = computed(
	() =>
		`${shiftHeld.value ? 'https://modrinth.com' : config.public.siteUrl}/project/${project.value?.id}`,
)
const { copy: copyProjectId, copied: projectIdCopied } = useClipboard({
	copiedDuring: 2000,
})
const { copy: copyPermalink, copied: permalinkCopied } = useClipboard({
	copiedDuring: 2000,
})
const editModal = useTemplateRef<InstanceType<typeof EditModal>>('editModal')
const monetizationStatus = computed(() =>
	project.value?.monetization_status === 'force-demonetized'
		? 'forceDemonetized'
		: project.value?.monetization_status,
)
const projectUrl = computed(
	() =>
		`/${projectV2.value?.project_type ?? project.value?.project_types[0]}/${project.value?.slug ?? project.value?.id}`,
)
const analyticsOwnerId = computed(
	() =>
		members.value.find((member) => member.is_owner)?.user.id ??
		organizationMembers.value.find((member) => member.is_owner)?.user.id,
)
const analyticsUrl = computed(() => ({
	path: '/dashboard/analytics',
	query: { user: analyticsOwnerId.value, a_projects: project.value?.id },
}))
const primaryProjectType = computed(() =>
	project.value ? getPrimaryProjectType(project.value) : undefined,
)
const projectTypes = computed(() => {
	if (project.value?.minecraft_server) return [{ id: 'server', icon: ServerIcon }]
	return (project.value?.project_types ?? [])
		.toSorted(
			(a, b) => Number(b === primaryProjectType.value) - Number(a === primaryProjectType.value),
		)
		.map((type) => ({ id: type, icon: getProjectTypeIcon(type) }))
})

const relativeTime = createRelativeTimeFormatter({ style: 'narrow' })
const formatDateTime = createDateFormatter({
	dateStyle: 'long',
	timeStyle: 'short',
})
</script>
