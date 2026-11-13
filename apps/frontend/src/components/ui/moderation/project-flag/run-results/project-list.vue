<template>
	<div class="flex min-w-0 flex-col gap-3">
		<slot />
		<div v-if="rows.length" class="flex flex-col gap-2 p-1">
			<ProjectCard
				v-for="row in rows"
				:key="row.project.id"
				:result="row"
				:project="row.details"
				:owner="row.owner"
				:flag-count="row.flagCount"
				:selected="selectedId === row.project.id"
				@select="emit('select', row.project.id)"
			/>
		</div>
		<p v-else-if="emptyMessage" class="universal-card m-0 text-secondary">
			{{ emptyMessage }}
		</p>
		<div v-if="categoryTotal > pageSize" class="flex flex-wrap items-center justify-between gap-2">
			<p role="status" class="m-0 text-sm text-secondary">
				Showing {{ offset + 1 }}–{{ offset + rows.length }} of {{ categoryTotal }}
			</p>
			<div class="flex gap-2">
				<Button size="sm" :disabled="offset === 0" @click="emit('changePage', -1)">Previous</Button>
				<Button
					size="sm"
					:disabled="offset + pageSize >= categoryTotal"
					@click="emit('changePage', 1)"
					>Next</Button
				>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth, ProjectFlagRunResult } from '@modrinth/api-client'
import { Button } from '@modrinth/ui'

import ProjectCard from './project-card.vue'

defineProps<{
	rows: (ProjectFlagRunResult & {
		flagCount: number
		details?: Labrinth.Projects.v3.Project
		owner?: { id: string; name: string; icon?: string | null; kind: 'user' | 'organization' }
	})[]
	selectedId?: string
	emptyMessage?: string
	offset: number
	pageSize: number
	categoryTotal: number
}>()

const emit = defineEmits<{ select: [id: string]; changePage: [direction: -1 | 1] }>()
</script>
