<script setup lang="ts">
import { CheckIcon, CopyIcon, ExternalIcon, MoreVerticalIcon, ServerIcon } from '@modrinth/assets'
import {
	Avatar,
	type ButtonMenuOption,
	getProjectTypeIcon,
	TagItem,
	TeleportOverflowMenu,
} from '@modrinth/ui'
import { getPrimaryProjectType } from '@modrinth/utils'
import { useClipboard, useMagicKeys } from '@vueuse/core'
import { computed } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import { projectTypeLabels } from '../../project-info/type-labels'
import ProjectMember from './project-member.vue'

const { project, projectV2, members, memberStats, membersLoading, membersError, organizationMembers } =
	injectProjectReviewPageContext()
const config = useRuntimeConfig()
const { shift: shiftHeld } = useMagicKeys()
const { copy: copyProjectId, copied: projectIdCopied } = useClipboard({ copiedDuring: 2000 })
const { copy: copyPermalink, copied: permalinkCopied } = useClipboard({ copiedDuring: 2000 })
const projectType = computed(() =>
	project.value?.minecraft_server
		? 'server'
		: project.value
			? getPrimaryProjectType(project.value)
			: 'project',
)
const projectUrl = computed(
	() =>
		`/${projectV2.value?.project_type ?? project.value?.project_types[0]}/${project.value?.slug ?? project.value?.id}`,
)
const permalink = computed(
	() =>
		`${shiftHeld.value ? 'https://modrinth.com' : config.public.siteUrl}/project/${project.value?.id}`,
)
const analyticsOwnerId = computed(
	() =>
		members.value.find((member) => member.is_owner)?.user.id ??
		organizationMembers.value.find((member) => member.is_owner)?.user.id,
)
const options = computed<ButtonMenuOption[]>(() => [
	{
		id: 'copy-project-id',
		label: projectIdCopied.value ? 'Copied project ID' : 'Copy project ID',
		icon: projectIdCopied.value ? CheckIcon : CopyIcon,
		action: () => copyProjectId(project.value?.id ?? ''),
	},
	{
		id: 'copy-permalink',
		label: permalinkCopied.value ? 'Copied permanent link' : 'Copy permanent link',
		icon: permalinkCopied.value ? CheckIcon : CopyIcon,
		action: () => copyPermalink(permalink.value),
	},
	{
		id: 'analytics',
		type: 'link',
		label: 'Go to analytics',
		icon: ExternalIcon,
		href: `/dashboard/analytics?${new URLSearchParams({
			user: analyticsOwnerId.value ?? '',
			a_projects: project.value?.id ?? '',
		})}`,
		target: '_blank',
		rel: 'noopener noreferrer',
		disabled: !analyticsOwnerId.value,
	},
])
</script>

<template>
	<section
		v-if="project"
		aria-label="Project overview"
		class="max-h-[35%] min-h-0 shrink-0 overflow-y-auto border-0 border-t border-solid border-surface-3 pr-2 pt-3"
	>
		<div>
			<div class="flex min-w-0 items-center gap-1.5">
				<NuxtLink
					:to="projectUrl"
					target="_blank"
					rel="noopener noreferrer"
					class="group/project flex min-w-0 items-center gap-2 rounded text-primary"
				>
					<Avatar :src="project.icon_url" :alt="project.name" size="1.75rem" no-shadow />
					<span
						class="min-w-0 truncate text-sm font-semibold group-hover/project:underline"
						:title="project.name"
					>
						{{ project.name }}
					</span>
				</NuxtLink>
				<TagItem
					class="shrink-0 !px-1.5 !py-0.5 !text-[10px] !font-medium !leading-tight"
				>
					<component
						:is="projectType === 'server' ? ServerIcon : getProjectTypeIcon(projectType)"
						class="!size-3"
						aria-hidden="true"
					/>
					{{ projectTypeLabels[projectType] ?? 'Project' }}
				</TagItem>
				<TeleportOverflowMenu
					label="Project options"
					:options="options"
					type="quiet"
					size="sm"
					class="ml-auto shrink-0"
				>
					<MoreVerticalIcon aria-hidden="true" />
					<span class="sr-only">Project options</span>
				</TeleportOverflowMenu>
			</div>
			<hr class="my-2.5 border-0 border-t border-solid border-surface-3" />
			<p v-if="membersLoading" role="status" class="mb-0 mt-2 text-xs text-secondary">
				Loading members...
			</p>
			<p v-else-if="membersError" role="alert" class="mb-0 mt-2 text-xs text-secondary">
				Could not load members.
			</p>
			<ul v-else aria-label="Project members" class="m-0 flex list-none flex-col gap-2.5 p-0">
				<ProjectMember
					v-for="member in members"
					:key="member.user.id"
					:member="member"
					:stats="memberStats[member.user.id] ?? []"
				/>
			</ul>
		</div>
	</section>
</template>
