<template>
	<Combobox
		v-model="projectId"
		placeholder="Select project"
		:options="options"
		:searchable="true"
		search-placeholder="Search by name or paste ID..."
		:no-options-message="searchLoading ? 'Loading...' : 'No results found'"
		:disable-search-filter="true"
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

		options.value = [...resultsByProjectId.hits, ...results.hits].map((hit) => ({
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
	} catch (error: any) {
		addNotification({
			title: 'An error occurred',
			text: error.data ? error.data.description : error,
			type: 'error',
		})
	}
	searchLoading.value = false
}

const throttledSearch = useDebounceFn(search, 500)

const handleSearch = async (query: string) => {
	searchLoading.value = true
	await throttledSearch(query)
}
</script>
