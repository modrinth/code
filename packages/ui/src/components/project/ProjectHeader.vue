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
									count: project.downloads,
								}),
							)
						"
						class="flex items-center gap-2"
					>
						{{ formatCompactNumber(project.downloads) }} downloads
					</div>
					<div
						v-tooltip="
							capitalizeString(
								formatMessage(commonMessages.projectFollowers, {
									count: project.followers,
								}),
							)
						"
						class="flex items-center gap-2"
						:class="{ 'md:border-r': project.categories.length > 0 }"
					>
						{{ formatCompactNumber(project.followers) }} followers
					</div>
				</template>
			</div>
		</template>
		<template #actions>
			<slot name="actions" />
		</template>
	</ContentPageHeader>
</template>
<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { capitalizeString, type Project } from '@modrinth/utils'
import { computed } from 'vue'

import { useCompactNumber, useVIntl } from '../../composables'
import { commonMessages } from '../../utils'
import Avatar from '../base/Avatar.vue'
import ContentPageHeader from '../base/ContentPageHeader.vue'
import ProjectStatusBadge from './ProjectStatusBadge.vue'
import ServerDetails from './server/ServerDetails.vue'

const { formatMessage } = useVIntl()
const { formatCompactNumber } = useCompactNumber()

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

const isServerProject = computed(() => !!props.projectV3?.minecraft_server)
const javaServer = computed(() => props.projectV3?.minecraft_java_server)
const javaServerPingData = computed(() => props.projectV3?.minecraft_java_server?.ping?.data)
const playersOnline = computed(() => javaServerPingData.value?.players_online ?? 0)
const statusOnline = computed(() => !!javaServerPingData.value)
</script>
