<template>
	<div
		class="flex min-w-0 flex-col gap-3 self-start lg:sticky lg:top-4 lg:max-h-[calc(100dvh-2rem)]"
	>
		<slot />
		<div v-if="rows.length" ref="cards" class="min-h-0 lg:overflow-y-auto">
			<div class="flex flex-col gap-2 p-1">
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
		</div>
		<p v-else-if="emptyMessage" class="universal-card m-0 text-secondary">
			{{ emptyMessage }}
		</p>
		<div
			v-if="categoryTotal > pageSize"
			class="flex shrink-0 flex-wrap items-center justify-between gap-2"
		>
			<p role="status" class="m-0 text-sm text-secondary">
				Showing {{ offset + 1 }}–{{ offset + rows.length }} of {{ categoryTotal }}
			</p>
			<Pagination
				:page="Math.floor(offset / pageSize) + 1"
				:count="Math.ceil(categoryTotal / pageSize)"
				@switch-page="emit('changePage', $event)"
			/>
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth, ProjectFlagRunResult } from '@modrinth/api-client'
import { Pagination } from '@modrinth/ui'
import { useTemplateRef, watch } from 'vue'

import ProjectCard from './project-card.vue'

const props = defineProps<{
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

const emit = defineEmits<{ select: [id: string]; changePage: [page: number] }>()

const cards = useTemplateRef<HTMLElement>('cards')
watch([() => props.offset, () => props.rows[0]?.project.id], () => {
	if (cards.value) cards.value.scrollTop = 0
})
</script>
