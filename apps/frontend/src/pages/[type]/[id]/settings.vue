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

import NavStack from '~/components/ui/NavStack.vue'
import NavStackItem from '~/components/ui/NavStackItem.vue'

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
</script>
<template>
	<div class="experimental-styles-within grid gap-4 lg:grid-cols-[1fr_3fr]">
		<div>
			<aside class="universal-card">
				<NavStack>
					<NavStackItem
						:link="`/${project.project_type}/${project.slug ? project.slug : project.id}/settings`"
						:label="formatMessage(commonProjectSettingsMessages.general)"
					>
						<InfoIcon aria-hidden="true" />
					</NavStackItem>
					<NavStackItem
						v-if="flags.newProjectGeneralSettings"
						:link="`/${project.project_type}/${project.slug ? project.slug : project.id}/settings/general`"
						:label="formatMessage(commonProjectSettingsMessages.general)"
						:badge="formatMessage(commonMessages.newBadge)"
					>
						<InfoIcon aria-hidden="true" />
					</NavStackItem>
					<NavStackItem
						v-if="
							flags.newProjectEnvironmentSettings &&
							projectV3.project_types.some((type) => ['mod', 'modpack'].includes(type))
						"
						:link="`/${project.project_type}/${
							project.slug ? project.slug : project.id
						}/settings/environment`"
						:label="formatMessage(commonProjectSettingsMessages.environment)"
						:badge="formatMessage(commonMessages.newBadge)"
					>
						<GlobeIcon aria-hidden="true" />
					</NavStackItem>
					<NavStackItem
						:link="`/${project.project_type}/${
							project.slug ? project.slug : project.id
						}/settings/tags`"
						:label="formatMessage(commonProjectSettingsMessages.tags)"
					>
						<TagsIcon aria-hidden="true" />
					</NavStackItem>
					<NavStackItem
						:link="`/${project.project_type}/${
							project.slug ? project.slug : project.id
						}/settings/description`"
						:label="formatMessage(commonProjectSettingsMessages.description)"
					>
						<AlignLeftIcon aria-hidden="true" />
					</NavStackItem>
					<NavStackItem
						:link="`/${project.project_type}/${
							project.slug ? project.slug : project.id
						}/settings/license`"
						:label="formatMessage(commonProjectSettingsMessages.license)"
					>
						<BookTextIcon aria-hidden="true" />
					</NavStackItem>
					<NavStackItem
						:link="`/${project.project_type}/${
							project.slug ? project.slug : project.id
						}/settings/links`"
						:label="formatMessage(commonProjectSettingsMessages.links)"
					>
						<LinkIcon aria-hidden="true" />
					</NavStackItem>
					<NavStackItem
						:link="`/${project.project_type}/${
							project.slug ? project.slug : project.id
						}/settings/members`"
						:label="formatMessage(commonProjectSettingsMessages.members)"
					>
						<UsersIcon aria-hidden="true" />
					</NavStackItem>
					<h3>{{ formatMessage(commonProjectSettingsMessages.view) }}</h3>
					<NavStackItem
						:link="`/${project.project_type}/${
							project.slug ? project.slug : project.id
						}/settings/analytics`"
						:label="formatMessage(commonProjectSettingsMessages.analytics)"
						chevron
					>
						<ChartIcon aria-hidden="true" />
					</NavStackItem>
					<h3>{{ formatMessage(commonProjectSettingsMessages.upload) }}</h3>
					<NavStackItem
						:link="`/${project.project_type}/${project.slug ? project.slug : project.id}/gallery`"
						:label="formatMessage(commonProjectSettingsMessages.gallery)"
						chevron
					>
						<ImageIcon aria-hidden="true" />
					</NavStackItem>
					<NavStackItem
						:link="`/${project.project_type}/${project.slug ? project.slug : project.id}/versions`"
						:label="formatMessage(commonProjectSettingsMessages.versions)"
						chevron
					>
						<VersionIcon aria-hidden="true" />
					</NavStackItem>
				</NavStack>
			</aside>
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
