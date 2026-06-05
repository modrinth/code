<template>
	<Combobox
		v-model="projectId"
		placeholder="Select project"
		:options="options"
		:search-value="selectedProjectOption?.label"
		search-placeholder="Search by name or paste ID..."
		:no-options-message="searchLoading ? 'Loading...' : 'No results found'"
		searchable
		disable-search-filter
		select-search-text-on-focus
		:show-chevron="false"
		@search-input="(query) => handleSearch(query)"
		@search-blur="handleSearchBlur"
		@select="handleSelect"
	/>
</template>

<script lang="ts" setup>
import type { ComboboxOption } from '@modrinth/ui'
import { Combobox, injectModrinthClient, injectNotificationManager } from '@modrinth/ui'
import { useDebounceFn } from '@vueuse/core'
import { defineAsyncComponent, h, markRaw, ref, watch } from 'vue'

const { addNotification } = injectNotificationManager()
const projectId = defineModel<string>()

const searchLoading = ref(false)
const options = ref<ComboboxOption<string>[]>([])
const selectedProjectOption = ref<ComboboxOption<string>>()
const selectedProjectSearchQuery = ref('')

const { labrinth } = injectModrinthClient()
let latestSearchQuery = ''

function hitToOption(hit: { title: string; project_id: string; icon_url?: string | null }) {
	return {
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
	}
}

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

		options.value = [...resultsByProjectId.hits, ...results.hits].map(hitToOption)
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

const throttledSearch = useDebounceFn(search, 250)

const runSearch = async (query: string, debounce: boolean) => {
	query = query.trim()
	latestSearchQuery = query

	if (!query) {
		searchLoading.value = false
		options.value = []
		await throttledSearch(query)
		return
	}

	searchLoading.value = true
	await (debounce ? throttledSearch(query) : search(query))
}

const handleSearch = async (query: string) => {
	await runSearch(query, true)
}

const handleSelect = (option: ComboboxOption<string>) => {
	selectedProjectOption.value = option
	selectedProjectSearchQuery.value = latestSearchQuery
}

const handleSearchBlur = async () => {
	if (!projectId.value) return

	const selectedOption =
		options.value.find((option) => option.value === projectId.value) ??
		(selectedProjectOption.value?.value === projectId.value
			? selectedProjectOption.value
			: undefined)
	if (!selectedOption) return

	await runSearch(selectedProjectSearchQuery.value || selectedOption.label, false)

	if (!options.value.some((option) => option.value === selectedOption.value)) {
		options.value = [selectedOption, ...options.value]
	}
}

watch(projectId, (value) => {
	if (!value) {
		selectedProjectOption.value = undefined
		selectedProjectSearchQuery.value = ''
		return
	}

	const option = options.value.find((option) => option.value === value)
	if (option) {
		selectedProjectOption.value = option
	}
})
</script>
