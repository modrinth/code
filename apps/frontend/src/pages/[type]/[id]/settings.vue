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
import type { Project, ProjectV3Partial } from '@modrinth/utils'

import NavStack from '~/components/ui/NavStack.vue'
import NavStackItem from '~/components/ui/NavStackItem.vue'

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
const featuredVersions = defineModel<any>('featuredVersions')
const members = defineModel<any>('members')
const allMembers = defineModel<any>('allMembers')
const dependencies = defineModel<any>('dependencies')
const organization = defineModel<any>('organization')
</script>
<template>
	<div class="experimental-styles-within grid grid-cols-[1fr_3fr] gap-4">
		<div>
			<aside class="universal-card">
				<NavStack>
					<NavStackItem
						:link="`/${project.project_type}/${project.slug ? project.slug : project.id}/settings`"
						label="General"
					>
						<InfoIcon aria-hidden="true" />
					</NavStackItem>
					<NavStackItem
						v-if="flags.newProjectGeneralSettings"
						:link="`/${project.project_type}/${project.slug ? project.slug : project.id}/settings/general`"
						label="General"
						badge="New"
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
						label="Environment"
						badge="New"
					>
						<GlobeIcon aria-hidden="true" />
					</NavStackItem>
					<NavStackItem
						:link="`/${project.project_type}/${
							project.slug ? project.slug : project.id
						}/settings/tags`"
						label="Tags"
					>
						<TagsIcon aria-hidden="true" />
					</NavStackItem>
					<NavStackItem
						:link="`/${project.project_type}/${
							project.slug ? project.slug : project.id
						}/settings/description`"
						label="Description"
					>
						<AlignLeftIcon aria-hidden="true" />
					</NavStackItem>
					<NavStackItem
						:link="`/${project.project_type}/${
							project.slug ? project.slug : project.id
						}/settings/license`"
						label="License"
					>
						<BookTextIcon aria-hidden="true" />
					</NavStackItem>
					<NavStackItem
						:link="`/${project.project_type}/${
							project.slug ? project.slug : project.id
						}/settings/links`"
						label="Links"
					>
						<LinkIcon aria-hidden="true" />
					</NavStackItem>
					<NavStackItem
						:link="`/${project.project_type}/${
							project.slug ? project.slug : project.id
						}/settings/members`"
						label="Members"
					>
						<UsersIcon aria-hidden="true" />
					</NavStackItem>
					<h3>View</h3>
					<NavStackItem
						:link="`/${project.project_type}/${
							project.slug ? project.slug : project.id
						}/settings/analytics`"
						label="Analytics"
						chevron
					>
						<ChartIcon aria-hidden="true" />
					</NavStackItem>
					<h3>Upload</h3>
					<NavStackItem
						:link="`/${project.project_type}/${project.slug ? project.slug : project.id}/gallery`"
						label="Gallery"
						chevron
					>
						<ImageIcon aria-hidden="true" />
					</NavStackItem>
					<NavStackItem
						:link="`/${project.project_type}/${project.slug ? project.slug : project.id}/versions`"
						label="Versions"
						chevron
					>
						<VersionIcon aria-hidden="true" />
					</NavStackItem>
				</NavStack>
			</aside>
		</div>
		<div>
			<NuxtPage
				v-model:project="project"
				v-model:project-v3="projectV3"
				v-model:versions="versions"
				v-model:featured-versions="featuredVersions"
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
