<template>
	<ProjectCard
		v-if="project"
		layout="list"
		:link="`/project/${project.slug ?? project.id}`"
		:icon-url="project.icon_url"
		:title="project.name"
		:summary="project.summary"
		:downloads="project.downloads"
		:followers="project.followers"
		:color="project.color"
	/>
	<MarkdownEmbedCard
		v-else
		:to="''"
		:loading="isLoading"
		:not-found="!isLoading"
		not-found-message="Project not found"
	/>
</template>

<script setup lang="ts">
import { useQuery } from '@tanstack/vue-query'

import { injectModrinthClient } from '../../../providers/api-client'
import ProjectCard from '../../project/card/ProjectCard.vue'
import MarkdownEmbedCard from './MarkdownEmbedCard.vue'

const props = defineProps<{
	id: string
}>()

const client = injectModrinthClient()

const { data: project, isLoading } = useQuery({
	queryKey: ['markdown-embed-project', () => props.id],
	queryFn: () => client.labrinth.projects_v3.get(props.id),
	enabled: () => !!props.id,
	retry: false,
})
</script>
