<template>
	<div class="flex min-w-40 flex-1 flex-col gap-1.5">
		<Combobox
			v-model="selectedId"
			:placeholder="
				multiple ? 'Search Modrinth to add a dependency…' : 'Search Modrinth to auto-fill…'
			"
			:options="options"
			:search-value="selectedOption?.label"
			search-placeholder="Search by name or paste ID…"
			:no-options-message="searchLoading ? 'Loading…' : 'No results found'"
			searchable
			disable-search-filter
			select-search-text-on-focus
			:show-chevron="false"
			@search-input="handleSearch"
			@select="handleSelect"
		/>
		<div v-if="multiple && selectedList.length > 0" class="flex flex-wrap gap-1">
			<span
				v-for="dep in selectedList"
				:key="dep.projectId"
				class="flex items-center gap-1 rounded-full border border-solid border-divider bg-surface-2 py-0.5 pl-2 pr-1 text-xs text-primary"
			>
				{{ dep.title }}
				<button
					class="rounded p-0.5 text-secondary hover:bg-button-bg hover:text-red"
					aria-label="Remove dependency"
					@click="removeEntry(dep.projectId)"
				>
					<XIcon class="size-3" />
				</button>
			</span>
		</div>
	</div>
</template>

<script lang="ts" setup>
import { XIcon } from '@modrinth/assets'
import type { ComboboxOption } from '@modrinth/ui'
import { Combobox, injectModrinthClient } from '@modrinth/ui'
import { useDebounceFn } from '@vueuse/core'
import { defineAsyncComponent, h, markRaw, ref } from 'vue'

/**
 * A dependency (or a reupload's original) may not even be on Modrinth, so this stays an
 * *assistive* picker layered next to plain text fields, never a replacement for them.
 *
 * Single-select (the default): selecting a hit fills sibling fields at once via `onSelect` (e.g.
 * a reupload's "original project" + "original author"). Its own value is just the picked project
 * id, mostly for the combobox's own display state.
 *
 * `multiple`: builds up a removable-chip list instead, one pick at a time. Its own value is that
 * whole list JSON-encoded (`{title, author, projectId, slug, projectType}[]`) — there's no fixed
 * set of sibling fields to fan a single pick out to, the list *is* the data — so a stage reads it
 * back with `JSON.parse` in its `.message()` to format each entry as a markdown link.
 */
const props = defineProps<{
	modelValue?: string
	multiple?: boolean
	onSelect?: (hit: DependencyEntry) => void
}>()

const emit = defineEmits<{
	'update:modelValue': [string]
}>()

const { labrinth } = injectModrinthClient()

interface Hit {
	title: string
	author: string
	project_id: string
	project_type: string
	slug: string | null
	icon_url?: string | null
}

export interface DependencyEntry {
	title: string
	author: string
	projectId: string
	slug: string
	projectType: string
}

function parseList(json: string | undefined): DependencyEntry[] {
	if (!json) return []
	try {
		const parsed: unknown = JSON.parse(json)
		return Array.isArray(parsed) ? (parsed as DependencyEntry[]) : []
	} catch {
		return []
	}
}

function hitToEntry(hit: Hit): DependencyEntry {
	return {
		title: hit.title,
		author: hit.author,
		projectId: hit.project_id,
		slug: hit.slug ?? hit.project_id,
		projectType: hit.project_type,
	}
}

const options = ref<ComboboxOption<string>[]>([])
const hitsById = new Map<string, Hit>()
const selectedOption = ref<ComboboxOption<string>>()
const searchLoading = ref(false)
let latestQuery = ''

const selectedId = ref(props.multiple ? '' : (props.modelValue ?? ''))
const selectedList = ref<DependencyEntry[]>(props.multiple ? parseList(props.modelValue) : [])

function hitToOption(hit: Hit) {
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
	const hit = hitsById.get(option.value)
	if (!hit) return
	const entry = hitToEntry(hit)

	if (props.multiple) {
		if (!selectedList.value.some((d) => d.projectId === entry.projectId)) {
			selectedList.value = [...selectedList.value, entry]
			emit('update:modelValue', JSON.stringify(selectedList.value))
		}
		// Clear the search box so the next pick starts fresh.
		selectedId.value = ''
		selectedOption.value = undefined
		options.value = []
	} else {
		selectedOption.value = option
		selectedId.value = option.value
		emit('update:modelValue', option.value)
	}
	props.onSelect?.(entry)
}

function removeEntry(projectId: string) {
	selectedList.value = selectedList.value.filter((d) => d.projectId !== projectId)
	emit('update:modelValue', JSON.stringify(selectedList.value))
}
</script>
