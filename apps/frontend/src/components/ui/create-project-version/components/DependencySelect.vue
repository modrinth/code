<template>
	<Combobox
		v-model="projectId"
		placeholder="Select project"
		:options="options"
		search-placeholder="Search by name or paste ID..."
		:no-options-message="searchLoading ? 'Loading...' : 'No results found'"
		searchable
		disable-search-filter
		select-search-text-on-focus
		:show-chevron="false"
		@search-input="(query) => handleSearch(query)"
	/>
</template>

<script lang="ts" setup>
import type { ComboboxOption } from '@modrinth/ui'
import { Combobox, injectModrinthClient, injectNotificationManager } from '@modrinth/ui'
import { useDebounceFn } from '@vueuse/core'
import { defineAsyncComponent, h } from 'vue'

const { addNotification } = injectNotificationManager()
const projectId = defineModel<string>()

const searchLoading = ref(false)
const options = ref<ComboboxOption<string>[]>([])

const { labrinth } = injectModrinthClient()
let latestSearchQuery = ''

const search = async (query: string) => {
	query = query.trim()
	if (!query) {
		searchLoading.value = false
		return
	}

	try {
		const results = await labrinth.projects_v2.search({
			query: query,
			limit: 20,
			facets: [
				[
					'project_type:mod',
					'project_type:plugin',
					'project_type:shader ',
					'project_type:resourcepack',
					'project_type:datapack',
				],
			],
		})

		const resultsByProjectId = await labrinth.projects_v2.search({
			query: '',
			limit: 20,
			facets: [[`project_id:${query.replace(/[^a-zA-Z0-9]/g, '')}`]], // remove any non-alphanumeric characters
		})

		if (query !== latestSearchQuery) return

		options.value = [...resultsByProjectId.hits, ...results.hits].map((hit) => ({
			label: hit.title,
			value: hit.project_id,
			icon: markRaw(
				defineAsyncComponent(() =>
					Promise.resolve({
						setup: () => () =>
							h('img', {
								src: hit.icon_url,
								alt: hit.title,
								class: 'h-5 w-5 rounded',
							}),
					}),
				),
			),
		}))
	} catch (error: any) {
		if (query !== latestSearchQuery) return

		addNotification({
			title: 'An error occurred',
			text: error.data ? error.data.description : error,
			type: 'error',
		})
	}

	if (query === latestSearchQuery) {
		searchLoading.value = false
	}
}

const throttledSearch = useDebounceFn(search, 500)

const handleSearch = async (query: string) => {
	query = query.trim()
	latestSearchQuery = query

	if (!query) {
		searchLoading.value = false
		options.value = []
		await throttledSearch(query)
		return
	}

	searchLoading.value = true
	await throttledSearch(query)
}
</script>
