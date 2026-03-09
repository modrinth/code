<template>
	<ContentPageHeader>
		<template #icon>
			<Avatar :src="project.icon_url" :alt="project.title" size="96px" />
		</template>
		<template #title>
			{{ project.title }}
		</template>
		<template #title-suffix>
			<ProjectStatusBadge v-if="member || project.status !== 'approved'" :status="project.status" />
		</template>
		<template #summary>
			{{ project.description }}
		</template>
		<template #stats>
			<div class="flex items-center gap-3 flex-wrap gap-y-0">
				<template v-if="isServerProject">
					<ServerDetails
						v-if="projectV3?.status !== 'draft'"
						:online-players="playersOnline"
						:status-online="statusOnline"
						:recent-plays="javaServer?.verified_plays_2w ?? 0"
					/>
				</template>
				<template v-else>
					<div
						v-tooltip="
							capitalizeString(
								formatMessage(commonMessages.projectDownloads, {
									count: formatNumber(project.downloads, false),
								}),
							)
						"
						class="flex items-center gap-2 font-semibold cursor-help"
					>
						<DownloadIcon class="h-6 w-6 text-secondary" />
						{{ formatNumber(project.downloads) }}
					</div>
					<div
						v-tooltip="
							capitalizeString(
								formatMessage(commonMessages.projectFollowers, {
									count: formatNumber(project.followers, false),
								}),
							)
						"
						class="flex items-center gap-2 cursor-help"
						:class="{ 'md:border-r': project.categories.length > 0 }"
					>
						<HeartIcon class="h-6 w-6 text-secondary" />
						<span class="font-semibold">
							{{ formatNumber(project.followers) }}
						</span>
					</div>
				</template>
				<div v-if="project.categories.length > 0" class="hidden items-center gap-2 md:flex">
					<div class="flex flex-wrap gap-2">
						<TagItem
							v-for="(category, index) in project.categories"
							:key="index"
							:action="() => router.push(`${searchUrl}?f=categories:${category}`)"
						>
							<FormattedTag :tag="category" />
						</TagItem>
					</div>
				</div>
			</div>
		</template>
		<template #actions>
			<slot name="actions" />
		</template>
	</ContentPageHeader>
</template>
<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { DownloadIcon, HeartIcon } from '@modrinth/assets'
import { capitalizeString, formatNumber, type Project } from '@modrinth/utils'
import { computed } from 'vue'
import { useRouter } from 'vue-router'

import { useVIntl } from '../../composables'
import { commonMessages } from '../../utils'
import Avatar from '../base/Avatar.vue'
import ContentPageHeader from '../base/ContentPageHeader.vue'
import FormattedTag from '../base/FormattedTag.vue'
import TagItem from '../base/TagItem.vue'
import ProjectStatusBadge from './ProjectStatusBadge.vue'
import ServerDetails from './server/ServerDetails.vue'

const router = useRouter()
const { formatMessage } = useVIntl()

const props = withDefaults(
	defineProps<{
		project: Project
		member?: boolean
		projectV3?: Labrinth.Projects.v3.Project | null
		ping?: number
	}>(),
	{
		member: false,
	},
)

const searchUrl = computed(
	() => `/discover/${isServerProject.value ? 'servers' : `${props.project.project_type}s`}`,
)

const isServerProject = computed(() => !!props.projectV3?.minecraft_server)
const javaServer = computed(() => props.projectV3?.minecraft_java_server)
const javaServerPingData = computed(() => props.projectV3?.minecraft_java_server?.ping?.data)
const playersOnline = computed(() => javaServerPingData.value?.players_online ?? 0)
const statusOnline = computed(() => !!javaServerPingData.value)
</script>
