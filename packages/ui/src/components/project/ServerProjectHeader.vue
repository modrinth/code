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
			<div class="flex items-center gap-3 gap-y-1 flex-wrap">
				<ServerDetails
					:online-players="playersOnline"
					:recent-plays="javaServer?.verified_plays_4w"
				/>
				<div v-if="project.categories.length > 0" class="hidden items-center gap-2 md:flex">
					<div class="flex gap-2">
						<TagItem
							v-for="(category, index) in project.categories"
							:key="index"
							:action="() => router.push(`/${project.project_type}s?f=categories:${category}`)"
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
import type { Project } from '@modrinth/utils'
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import Avatar from '../base/Avatar.vue'
import ContentPageHeader from '../base/ContentPageHeader.vue'
import FormattedTag from '../base/FormattedTag.vue'
import TagItem from '../base/TagItem.vue'
import ProjectStatusBadge from './ProjectStatusBadge.vue'
import ServerDetails from './server/ServerDetails.vue'

const router = useRouter()

const { project, projectV3, member } = defineProps<{
	project: Project
	projectV3: Labrinth.Projects.v3.Project | null
	member?: boolean
	ping?: number
}>()

const javaServer = computed(() => projectV3?.minecraft_java_server)
const javaServerPingData = computed(() => projectV3?.minecraft_java_server?.ping?.data)
const playersOnline = computed(() => javaServerPingData.value?.players_online ?? 0)
</script>
