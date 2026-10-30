<script setup lang="ts">
import {
	ArchiveIcon,
	BrainCogIcon,
	CheckIcon,
	CircleDollarSignIcon,
	CodeIcon,
	CopyIcon,
	ExternalIcon,
	EyeIcon,
	GitForkIcon,
	MegaphoneIcon,
	MonitorCogIcon,
	MoreVerticalIcon,
	RadioTowerIcon,
	ServerIcon,
	SparklesIcon,
} from '@modrinth/assets'
import {
	Avatar,
	ButtonLink,
	type ButtonMenuOption,
	getProjectTypeIcon,
	TagItem,
	TeleportOverflowMenu,
	truncatedTooltip,
} from '@modrinth/ui'
import { getPrimaryProjectType } from '@modrinth/utils'
import { useClipboard, useMagicKeys } from '@vueuse/core'
import { computed, ref } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'
import { reviewExternalUrl } from '~/providers/project-review/project-links'

import { injectReviewContext } from '../../review-panel/context'
import { projectTypeLabels } from '../../project-info/type-labels'
import ProjectMember from './project-member.vue'

const {
	project,
	projectV2,
	members,
	memberStats,
	membersLoading,
	membersError,
	organizationMembers,
	disclosures,
} = injectProjectReviewPageContext()
const projectNameRef = ref<HTMLElement | null>(null)
const { revealPanel } = injectReviewContext()
const disclosurePresentation = {
	ai_content: { label: 'AI-generated content', icon: SparklesIcon, key: 'ai' },
	ai_functionality: {
		label: 'Generative AI functionality',
		icon: BrainCogIcon,
		key: 'ai-functionality',
	},
	advertisements: { label: 'Contains advertising', icon: MegaphoneIcon, key: 'ads' },
	paid_features: {
		label: 'Contains paid features',
		icon: CircleDollarSignIcon,
		key: 'paid-features',
	},
	telemetry: { label: 'Contains telemetry', icon: RadioTowerIcon, key: 'telemetry' },
	derivative_work: { label: 'Derivative work', icon: GitForkIcon, key: 'derivative-content' },
	epilepsy_triggers: { label: 'Photosensitivity warning', icon: EyeIcon, key: 'photosensitivity' },
	system_interactions: {
		label: 'External system interactions',
		icon: MonitorCogIcon,
		key: 'system-interactions',
	},
	archived: { label: 'Archived', icon: ArchiveIcon, key: 'archive' },
}
const activeDisclosures = computed(() =>
	disclosures.currentSnapshot.value.disclosures.map(({ type }) => ({
		type,
		...disclosurePresentation[type],
	})),
)
const sourceUrl = computed(() => reviewExternalUrl(project.value?.link_urls.source?.url))
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
		class="max-h-[35%] min-h-0 shrink-0 overflow-y-auto border-0 border-t border-solid border-surface-3 pr-2 pt-1.5"
	>
		<div>
			<div class="flex min-w-0 items-center gap-1.5">
				<NuxtLink
					:to="projectUrl"
					target="_blank"
					rel="noopener noreferrer"
					class="group/project flex min-w-0 items-center gap-2 rounded text-primary"
				>
					<Avatar :src="project.icon_url" :alt="project.name" size="30px" no-shadow />
					<span
						class="min-w-0 truncate text-sm font-semibold group-hover/project:underline"
						ref="projectNameRef"
						v-tooltip="truncatedTooltip(projectNameRef, project.name)"
					>
						{{ project.name }}
					</span>
				</NuxtLink>
				<TagItem class="shrink-0 !px-1.5 !py-0.5 !text-[10px] !font-medium !leading-tight">
					<component
						:is="projectType === 'server' ? ServerIcon : getProjectTypeIcon(projectType)"
						class="!size-3"
						aria-hidden="true"
					/>
					{{ projectTypeLabels[projectType] ?? 'Project' }}
				</TagItem>
				<div class="ml-auto flex shrink-0 items-center gap-1">
					<ButtonLink
						v-if="sourceUrl"
						v-tooltip="'Source code'"
						:href="sourceUrl"
						target="_blank"
						type="quiet"
						size="xs"
						icon-only
						circular
						aria-label="Source code"
					>
						<CodeIcon aria-hidden="true" />
					</ButtonLink>
					<TeleportOverflowMenu
						label="Project options"
						:options="options"
						type="quiet"
						size="xs"
						class="shrink-0"
					>
						<MoreVerticalIcon aria-hidden="true" />
						<span class="sr-only">Project options</span>
					</TeleportOverflowMenu>
				</div>
			</div>

			<hr class="my-1.5 border-0 border-t border-solid border-surface-3" />
			<p v-if="membersLoading" role="status" class="mb-0 mt-2 text-xs text-secondary">
				Loading members...
			</p>
			<p v-else-if="membersError" role="alert" class="mb-0 mt-2 text-xs text-secondary">
				Could not load members.
			</p>
			<ul v-else aria-label="Project members" class="m-0 flex list-none flex-col gap-1.5 p-0">
				<ProjectMember
					v-for="member in members"
					:key="member.user.id"
					:member="member"
					:stats="memberStats[member.user.id] ?? []"
				/>
			</ul>
			<section
				v-if="activeDisclosures.length"
				aria-label="Project disclosures"
				class="mt-1.5 border-0 border-t border-solid border-surface-3 pt-1.5 pl-1"
			>
				<ul class="m-0 flex list-none flex-col gap-0.5 p-0">
					<li v-for="disclosure in activeDisclosures" :key="disclosure.type">
						<button
							type="button"
							class="group/disclosure flex w-fit items-start gap-1 rounded border-0 bg-transparent p-0 text-left text-xs leading-4 text-secondary focus-visible:outline focus-visible:outline-2 focus-visible:outline-brand"
							:class="{ 'text-orange': disclosure.type === 'epilepsy_triggers' }"
							@click="revealPanel(`${disclosure.key}-disclosure`)"
						>
							<component :is="disclosure.icon" class="mt-0.5 size-3 shrink-0" aria-hidden="true" />
							<span class="group-hover/disclosure:underline">{{ disclosure.label }}</span>
						</button>
					</li>
				</ul>
			</section>
		</div>
	</section>
</template>
