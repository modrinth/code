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
				<!-- TODO_SERVER_PROJECTS, hook up modrinth recent plays when ready -->
				<ServerDetails
					:region="minecraftServer?.country"
					:online-players="playersOnline"
					:recent-plays="12345"
					:ping="ping"
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
				<ServerModpackContent
					v-if="serverModpackContent"
					:name="serverModpackContent.name"
					:icon="serverModpackContent.icon"
					:link="serverModpackContent.link"
				/>
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
import ServerModpackContent from './server/ServerModpackContent.vue'

const router = useRouter()

const { project, projectV3, member } = defineProps<{
	project: Project
	projectV3: Labrinth.Projects.v3.Project | null
	member?: boolean
	ping?: number
}>()

const minecraftServer = computed(() => projectV3?.minecraft_server)
const javaServer = computed(() => projectV3?.minecraft_java_server)
const javaServerPingData = computed(() => projectV3?.minecraft_java_server_ping?.data)
const playersOnline = computed(() => javaServerPingData.value?.players_online ?? 0)

const serverModpackContent = computed(() => {
	if (projectV3?.minecraft_java_server?.content?.kind === 'modpack') {
		const { project_name, project_icon, project_id } = projectV3.minecraft_java_server.content
		if (!project_name) return undefined
		return {
			name: project_name,
			icon: project_icon,
			link: project_id ? `/project/${project_id}` : undefined,
		}
	}
	return undefined
})
</script>
