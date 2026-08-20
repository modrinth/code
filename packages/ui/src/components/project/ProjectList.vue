<template>
	<ProjectCardList :layout="layout">
		<template v-for="project in projects" :key="project.id">
			<ProjectCard
				v-if="isProjectServer(project)"
				:link="getProjectPagePath(project, linkMode)"
				:title="project.name"
				:icon-url="project.icon_url"
				:summary="project.summary"
				:tags="getProjectCardTags(project)"
				:server-online-players="project.minecraft_java_server?.ping?.data?.players_online ?? 0"
				:server-recent-plays="project.minecraft_java_server?.verified_plays_2w ?? 0"
				:server-region="project.minecraft_server?.region"
				:server-status-online="!!project.minecraft_java_server?.ping?.data"
				:server-modpack-content="getServerModpackContent(project, openModpackProject)"
				:status="showStatus ? project.status : undefined"
				:max-tags="2"
				:layout="cardLayout"
				is-server-project
				exclude-loaders
			>
				<template v-if="$slots.actions" #actions>
					<slot name="actions" :project="project" />
				</template>
			</ProjectCard>
			<ProjectCard
				v-else
				:link="getProjectPagePath(project, linkMode)"
				:title="project.name"
				:icon-url="project.icon_url"
				:date-updated="project.updated"
				:downloads="project.downloads"
				:summary="project.summary"
				:tags="getProjectCardTags(project)"
				:all-tags="getProjectCardAllTags(project)"
				:followers="project.followers"
				:banner="project.gallery?.find((image) => image.featured)?.url"
				:color="project.color"
				:environment="project.environment?.[0]"
				:layout="cardLayout"
				:status="showStatus ? project.status : undefined"
			>
				<template v-if="$slots.actions" #actions>
					<slot name="actions" :project="project" />
				</template>
			</ProjectCard>
		</template>
	</ProjectCardList>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { computed } from 'vue'
import { useRouter } from 'vue-router'

import { PROJECT_DEP_MARKER_QUERY } from '#ui/providers'
import {
	getProjectCardAllTags,
	getProjectCardTags,
	getProjectPagePath,
	getServerModpackContent,
	isProjectServer,
	type ProjectLinkMode,
} from '#ui/utils/v3-projects'

import ProjectCard from './card/ProjectCard.vue'
import ProjectCardList from './ProjectCardList.vue'

const props = withDefaults(
	defineProps<{
		projects: Labrinth.Projects.v3.Project[]
		layout?: 'list' | 'grid' | 'gallery'
		linkMode?: ProjectLinkMode
		showStatus?: boolean
	}>(),
	{
		layout: 'list',
		linkMode: 'website',
		showStatus: false,
	},
)

defineSlots<{
	actions?: (props: { project: Labrinth.Projects.v3.Project }) => unknown
}>()

const router = useRouter()
const cardLayout = computed(() => (props.layout === 'list' ? 'list' : 'grid'))

function openModpackProject(projectId: string): void {
	const path = `/project/${projectId}`
	if (props.linkMode === 'app') {
		void router.push(path)
		return
	}
	void router.push({
		path,
		query: { ...PROJECT_DEP_MARKER_QUERY },
	})
}
</script>
