<script setup lang="ts">
import {
	AlignLeftIcon,
	BookTextIcon,
	ChartIcon,
	ImageIcon,
	InfoIcon,
	LinkIcon,
	TagsIcon,
	UsersIcon,
	VersionIcon,
} from '@modrinth/assets'
import {
	commonMessages,
	commonProjectSettingsMessages,
	injectNotificationManager,
} from '@modrinth/ui'
import type { Project, ProjectV3Partial } from '@modrinth/utils'
import { useVIntl } from '@vintl/vintl'
import { useLocalStorage, useScroll } from '@vueuse/core'
import { computed } from 'vue'

import ModerationProjectNags from '~/components/ui/moderation/ModerationProjectNags.vue'
import NavStack from '~/components/ui/NavStack.vue'

const { formatMessage } = useVIntl()

defineProps<{
	currentMember: any
	patchProject: any
	patchIcon: any
	resetProject: any
	resetVersions: any
	resetOrganization: any
	resetMembers: any
}>()

const flags = useFeatureFlags()

const project = defineModel<Project>('project', { required: true })
const projectV3 = defineModel<ProjectV3Partial>('projectV3', { required: true })
const versions = defineModel<any>('versions')
const members = defineModel<any>('members')
const allMembers = defineModel<any>('allMembers')
const dependencies = defineModel<any>('dependencies')
const organization = defineModel<any>('organization')

const navItems = computed(() => {
	const base = `${project.value.project_type}/${project.value.slug ? project.value.slug : project.value.id}`
	const items = [
		{
			link: `/${base}/settings`,
			label: formatMessage(commonProjectSettingsMessages.general),
			icon: InfoIcon,
		},
		flags.value.newProjectGeneralSettings
			? {
					link: `/${base}/settings/general`,
					label: formatMessage(commonProjectSettingsMessages.general),
					badge: formatMessage(commonMessages.newBadge),
					icon: InfoIcon,
				}
			: null,
		{
			link: `/${base}/settings/tags`,
			label: formatMessage(commonProjectSettingsMessages.tags),
			icon: TagsIcon,
		},
		{
			link: `/${base}/settings/description`,
			label: formatMessage(commonProjectSettingsMessages.description),
			icon: AlignLeftIcon,
		},
		{
			link: `/${base}/settings/versions`,
			label: formatMessage(commonProjectSettingsMessages.versions),
			icon: VersionIcon,
		},
		{
			link: `/${base}/settings/license`,
			label: formatMessage(commonProjectSettingsMessages.license),
			icon: BookTextIcon,
		},
		{
			link: `/${base}/settings/gallery`,
			label: formatMessage(commonProjectSettingsMessages.gallery),
			icon: ImageIcon,
		},
		{
			link: `/${base}/settings/links`,
			label: formatMessage(commonProjectSettingsMessages.links),
			icon: LinkIcon,
		},
		{
			link: `/${base}/settings/members`,
			label: formatMessage(commonProjectSettingsMessages.members),
			icon: UsersIcon,
		},
		{
			link: `/${base}/settings/analytics`,
			label: formatMessage(commonProjectSettingsMessages.analytics),
			icon: ChartIcon,
		},
	]
	return items.filter(Boolean) as any[]
})

const { addNotification } = injectNotificationManager()

const tags = useGeneratedState()
const route = useRoute()
const collapsedChecklist = useLocalStorage(`project-checklist-collapsed-${project.value.id}`, false)

async function setProcessing() {
	startLoading()

	try {
		await useBaseFetch(`project/${project.value.id}`, {
			method: 'PATCH',
			body: {
				status: 'processing',
			},
		})

		project.value.status = 'processing'
	} catch (err: any) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}

	stopLoading()
}

// To persist scroll position through settings pages
// This scroll code is jank asf, if anyone has a better way please do suggest it
const scroll = useScroll(window)
watch(route, () => {
	const scrollY = scroll.y.value
	setTimeout(() => window.scrollTo(0, scrollY), 10)
})
</script>

<template>
	<div class="mb-8 flex w-full flex-col gap-4">
		<ModerationProjectNags
			v-if="
				(currentMember && project.status === 'draft') ||
				tags.rejectedStatuses.includes(project.status)
			"
			:project="project"
			:versions="versions"
			:current-member="currentMember"
			:collapsed="collapsedChecklist"
			:route-name="route.name as string"
			:tags="tags"
			@toggle-collapsed="() => (collapsedChecklist = !collapsedChecklist)"
			@set-processing="setProcessing"
		/>
		<div class="experimental-styles-within grid gap-4 lg:grid-cols-[1fr_3fr]">
			<div>
				<NavStack :items="navItems" />
			</div>
			<div class="min-w-0">
				<NuxtPage
					v-model:project="project"
					v-model:project-v3="projectV3"
					v-model:versions="versions"
					v-model:members="members"
					v-model:all-members="allMembers"
					v-model:dependencies="dependencies"
					v-model:organization="organization"
					:current-member="currentMember"
					:patch-project="patchProject"
					:patch-icon="patchIcon"
					:reset-project="resetProject"
					:reset-versions="resetVersions"
					:reset-organization="resetOrganization"
					:reset-members="resetMembers"
				/>
			</div>
		</div>
	</div>
</template>
