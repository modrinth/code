<template>
	<Combobox
		v-model="projectId"
		:placeholder="placeholder"
		:options="options"
		:searchable="true"
		:search-placeholder="searchPlaceholder"
		:no-options-message="searchLoading ? loadingMessage : noResultsMessage"
		:disable-search-filter="true"
		:disabled="disabled"
		show-icon-in-selected
		@search-input="(query) => handleSearch(query)"
	/>
</template>

<script lang="ts" setup>
import { useDebounceFn } from '@vueuse/core'
import { defineAsyncComponent, h, ref, watch } from 'vue'

import { injectModrinthClient, injectNotificationManager } from '../../providers'
import type { ComboboxOption } from '../base/Combobox.vue'
import Combobox from '../base/Combobox.vue'

export type ProjectType =
	| 'mod'
	| 'modpack'
	| 'resourcepack'
	| 'shader'
	| 'datapack'
	| 'plugin'
	| 'server'

interface SearchHit {
	project_id: string
	title: string
	icon_url?: string
	project_type: string
	slug: string
}

const props = withDefaults(
	defineProps<{
		/** Filter by project types */
		projectTypes?: ProjectType[]
		/** Placeholder text for the combobox */
		placeholder?: string
		/** Placeholder text for the search input */
		searchPlaceholder?: string
		/** Message shown when loading */
		loadingMessage?: string
		/** Message shown when no results found */
		noResultsMessage?: string
		/** Whether the combobox is disabled */
		disabled?: boolean
		/** Maximum number of results to show */
		limit?: number
	}>(),
	{
		placeholder: 'Select project',
		searchPlaceholder: 'Search by name or paste ID...',
		loadingMessage: 'Loading...',
		noResultsMessage: 'No results found',
		disabled: false,
		limit: 20,
	},
)

const { addNotification } = injectNotificationManager()
const projectId = defineModel<string>()

const searchLoading = ref(false)
const options = ref<ComboboxOption<string>[]>([])
const selectedProject = ref<SearchHit | null>(null)
const searchResultsCache = ref<Map<string, SearchHit>>(new Map())

const { labrinth } = injectModrinthClient()

// Watch for external changes to projectId to update selectedProject
watch(
	projectId,
	async (newId) => {
		if (!newId) {
			selectedProject.value = null
			return
		}

		if (searchResultsCache.value.has(newId)) {
			selectedProject.value = searchResultsCache.value.get(newId) || null
			return
		}

		try {
			const project = await labrinth.projects_v2.get(newId)
			if (project) {
				const hit: SearchHit = {
					project_id: project.id,
					title: project.title,
					icon_url: project.icon_url ?? undefined,
					project_type: project.project_type,
					slug: project.slug,
				}
				searchResultsCache.value.set(project.id, hit)
				selectedProject.value = hit
			}
		} catch {
			selectedProject.value = null
		}
	},
	{ immediate: true },
)

const search = async (query: string) => {
	query = query.trim()
	if (!query) {
		searchLoading.value = false
		options.value = []
		return
	}

	try {
		const projectTypeFacets = props.projectTypes?.map((type) => `project_type:${type}`)

		const results = await labrinth.projects_v2.search({
			query: query,
			limit: props.limit,
			facets: projectTypeFacets ? [projectTypeFacets] : undefined,
		})

		const resultsByProjectId = await labrinth.projects_v2.search({
			query: '',
			limit: props.limit,
			facets: [[`project_id:${query.replace(/[^a-zA-Z0-9]/g, '')}`]],
		})

		const allHits = [...resultsByProjectId.hits, ...results.hits]
		const seenIds = new Set<string>()
		const uniqueHits: SearchHit[] = []

		for (const hit of allHits) {
			if (!seenIds.has(hit.project_id)) {
				seenIds.add(hit.project_id)
				uniqueHits.push(hit)
				// Cache the hit for later lookup
				searchResultsCache.value.set(hit.project_id, hit)
			}
		}

		options.value = uniqueHits.map((hit) => ({
			label: hit.title,
			value: hit.project_id,
			icon: defineAsyncComponent(() =>
				Promise.resolve({
					setup: () => () =>
						h('img', {
							src: hit.icon_url,
							alt: hit.title,
							class: 'h-5 w-5 rounded',
						}),
				}),
			),
		}))
	} catch (error: unknown) {
		const err = error as { data?: { description?: string } }
		addNotification({
			title: 'An error occurred',
			text: err.data ? err.data.description : String(error),
			type: 'error',
		})
	}
	searchLoading.value = false
}

const throttledSearch = useDebounceFn(search, 250)

const handleSearch = async (query: string) => {
	searchLoading.value = true
	await throttledSearch(query)
}

defineExpose({
	selectedProject,
})
</script>
