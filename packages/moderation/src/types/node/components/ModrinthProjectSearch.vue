<template>
	<Combobox
		v-model="selectedId"
		placeholder="Search Modrinth to auto-fill…"
		:options="options"
		:search-value="selectedOption?.label"
		search-placeholder="Search by name or paste ID…"
		:no-options-message="searchLoading ? 'Loading…' : 'No results found'"
		searchable
		disable-search-filter
		select-search-text-on-focus
		:show-chevron="false"
		class="min-w-40 flex-1"
		@search-input="handleSearch"
		@select="handleSelect"
	/>
</template>

<script lang="ts" setup>
import type { ComboboxOption } from '@modrinth/ui'
import { Combobox, injectModrinthClient } from '@modrinth/ui'
import { useDebounceFn } from '@vueuse/core'
import {defineAsyncComponent, h, markRaw, ref} from 'vue'

/**
 * Reupload reports need the original project's title and author, but that original may not even
 * be on Modrinth — so this stays an *assistive* picker layered next to plain text fields, not a
 * replacement for them. Selecting a hit fills both fields at once via `onSelect`.
 */
const props = defineProps<{
	modelValue?: string
	onSelect?: (hit: { title: string; author: string; projectId: string }) => void
}>()

const emit = defineEmits<{
	'update:modelValue': [string]
}>()

const { labrinth } = injectModrinthClient()

interface Hit {
	title: string
	author: string
	project_id: string
	icon_url?: string | null
}

const options = ref<ComboboxOption<string>[]>([])
const hitsById = new Map<string, Hit>()
const selectedOption = ref<ComboboxOption<string>>()
const searchLoading = ref(false)
let latestQuery = ''

const selectedId = ref(props.modelValue ?? '')

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

async function search(query: string) {
	query = query.trim()
	latestQuery = query
	if (!query) {
		options.value = []
		searchLoading.value = false
		return
	}
	searchLoading.value = true
	try {
		const results = await labrinth.projects_v2.search({ query, limit: 10 })
		if (query !== latestQuery) return
		hitsById.clear()
		for (const hit of results.hits as Hit[]) hitsById.set(hit.project_id, hit)
		options.value = (results.hits as Hit[]).map(hitToOption)
	} catch {
		if (query === latestQuery) options.value = []
	} finally {
		if (query === latestQuery) searchLoading.value = false
	}
}

const debouncedSearch = useDebounceFn(search, 250)

function handleSearch(query: string) {
	void debouncedSearch(query)
}

function handleSelect(option: ComboboxOption<string>) {
	selectedOption.value = option
	selectedId.value = option.value
	emit('update:modelValue', option.value)
	const hit = hitsById.get(option.value)
	if (hit) props.onSelect?.({ title: hit.title, author: hit.author, projectId: hit.project_id })
}
</script>
