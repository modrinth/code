<script setup lang="ts">
import {
	AlignLeftIcon,
	BookTextIcon,
	ChartIcon,
	GlobeIcon,
	ImageIcon,
	InfoIcon,
	LinkIcon,
	TagsIcon,
	UsersIcon,
	VersionIcon,
} from '@modrinth/assets'
import { commonMessages, commonProjectSettingsMessages } from '@modrinth/ui'
import type { Project, ProjectV3Partial } from '@modrinth/utils'
import { useVIntl } from '@vintl/vintl'
import { computed } from 'vue'

import NavStack from '~/components/ui/NavStack.vue'

const { formatMessage } = useVIntl()

defineProps<{
	currentMember: any
	patchProject: any
	patchIcon: any
	resetProject: any
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
		flags.value.newProjectEnvironmentSettings &&
		projectV3.value.project_types.some((type: string) => ['mod', 'modpack'].includes(type))
			? {
					link: `/${base}/settings/environment`,
					label: formatMessage(commonProjectSettingsMessages.environment),
					badge: formatMessage(commonMessages.newBadge),
					icon: GlobeIcon,
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
			link: `/${base}/settings/license`,
			label: formatMessage(commonProjectSettingsMessages.license),
			icon: BookTextIcon,
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
		{ type: 'heading', label: formatMessage(commonProjectSettingsMessages.view) },
		{
			link: `/${base}/settings/analytics`,
			label: formatMessage(commonProjectSettingsMessages.analytics),
			icon: ChartIcon,
			chevron: true,
		},
		{ type: 'heading', label: formatMessage(commonProjectSettingsMessages.upload) },
		{
			link: `/${base}/gallery`,
			label: formatMessage(commonProjectSettingsMessages.gallery),
			icon: ImageIcon,
			chevron: true,
		},
		{
			link: `/${base}/versions`,
			label: formatMessage(commonProjectSettingsMessages.versions),
			icon: VersionIcon,
			chevron: true,
		},
	]
	return items.filter(Boolean) as any[]
})
</script>
<template>
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
				:reset-organization="resetOrganization"
				:reset-members="resetMembers"
			/>
		</div>
	</div>
</template>
