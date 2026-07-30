<script setup lang="ts">
import { ScaleIcon, XIcon } from '@modrinth/assets'
import { AutoLink, ButtonStyled, injectModrinthClient, NewModal } from '@modrinth/ui'
import { ref, useTemplateRef } from 'vue'

import { useGeneratedState } from '~/composables/generated'
import { getProjectTypeForUrlShorthand } from '~/helpers/projects.js'

const props = defineProps<{
	completedIds: string[]
	skippedIds: string[]
}>()

const emit = defineEmits<{
	(e: 'review-skipped'): void
}>()

const client = injectModrinthClient()
const tags = useGeneratedState()
const modalRef = useTemplateRef<InstanceType<typeof NewModal>>('modalRef')

interface QueueSummaryEntry {
	id: string
	title: string
	link: string
}

const completedEntries = ref<QueueSummaryEntry[]>([])
const skippedEntries = ref<QueueSummaryEntry[]>([])

function toEntries(
	ids: string[],
	projectsById: Map<string, { id: string; slug: string; title: string; project_types: string[] }>,
): QueueSummaryEntry[] {
	return ids
		.map((id) => projectsById.get(id))
		.filter((project): project is NonNullable<typeof project> => !!project)
		.map((project) => ({
			id: project.id,
			title: project.title,
			link: `/${getProjectTypeForUrlShorthand(project.project_types[0], [], tags.value)}/${project.slug}`,
		}))
}

async function show() {
	const ids = [...new Set([...props.completedIds, ...props.skippedIds])]
	const projects =
		ids.length > 0 ? await client.labrinth.projects_v3.getMultiple(ids).catch(() => []) : []
	const projectsById = new Map(projects.map((project) => [project.id, project]))

	completedEntries.value = toEntries(props.completedIds, projectsById)
	skippedEntries.value = toEntries(props.skippedIds, projectsById)

	modalRef.value?.show()
}

function hide() {
	modalRef.value?.hide()
}

function reviewSkipped() {
	emit('review-skipped')
	hide()
}

defineExpose({ show, hide })
</script>

<template>
	<NewModal ref="modalRef" header="Queue completed">
		<div class="flex flex-col gap-4">
			<div v-if="completedEntries.length > 0" class="flex flex-col gap-2">
				<span class="font-bold text-contrast">Completed ({{ completedEntries.length }})</span>
				<ul class="m-0 flex list-none flex-col gap-1 p-0">
					<li v-for="entry in completedEntries" :key="entry.id">
						<AutoLink :to="entry.link" class="text-primary hover:underline">{{
							entry.title
						}}</AutoLink>
					</li>
				</ul>
			</div>

			<div v-if="skippedEntries.length > 0" class="flex flex-col gap-2">
				<span class="font-bold text-contrast">Skipped ({{ skippedEntries.length }})</span>
				<ul class="m-0 flex list-none flex-col gap-1 p-0">
					<li v-for="entry in skippedEntries" :key="entry.id">
						<AutoLink :to="entry.link" class="text-primary hover:underline">{{
							entry.title
						}}</AutoLink>
					</li>
				</ul>
			</div>

			<div v-if="completedEntries.length === 0 && skippedEntries.length === 0" class="text-secondary">
				No projects were reviewed during this queue.
			</div>

			<div class="flex justify-end gap-2">
				<ButtonStyled v-if="skippedEntries.length > 0" color="orange">
					<button @click="reviewSkipped">
						<ScaleIcon />
						Review skipped ({{ skippedEntries.length }})
					</button>
				</ButtonStyled>
				<ButtonStyled>
					<button @click="hide">
						<XIcon />
						Close
					</button>
				</ButtonStyled>
			</div>
		</div>
	</NewModal>
</template>
