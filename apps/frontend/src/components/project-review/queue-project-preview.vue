<template>
	<NuxtLink
		v-if="project"
		:to="`/project/${project.id}`"
		target="_blank"
		rel="noopener noreferrer"
		class="flex min-w-0 items-center gap-2 text-primary hover:text-contrast hover:underline"
	>
		<Avatar :src="project.icon_url" size="1.25rem" no-shadow />
		<span class="truncate" :title="project.name">{{ project.name }}</span>
	</NuxtLink>
	<span v-else class="text-secondary">{{ projectError ? 'Unavailable' : 'Loading...' }}</span>
</template>

<script setup lang="ts">
import { Avatar, injectModrinthClient } from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed } from 'vue'

import { queueProjectPreviewOptions } from './queue-project-cache'

const props = defineProps<{ projectId: string }>()
const client = injectModrinthClient()
const queryClient = useQueryClient()

const { data: project, isError: projectError } = useQuery(
	computed(() => queueProjectPreviewOptions(props.projectId, client, queryClient)),
)
</script>
