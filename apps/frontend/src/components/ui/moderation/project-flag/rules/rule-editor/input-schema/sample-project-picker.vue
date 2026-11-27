<template>
	<Combobox
		v-model="projectId"
		:options="selectedOption ? [selectedOption] : []"
		:search-options="searchOptions"
		:search-value="selectedOption?.label ?? ''"
		:no-options-message="searchMessage"
		:show-chevron="false"
		:search-input-attrs="{
			'aria-label': 'Sample project',
			class: '[&::-webkit-search-cancel-button]:hidden',
		}"
		searchable
		disable-search-filter
		clearable
		search-placeholder="Select project"
		search-type="search"
		search-name="flag-rule-sample-project-search"
		search-autocomplete="new-password"
		dropdown-width="22rem"
		@search-input="onSearchInput"
		@search-blur="searchInput = ''"
		@select="selectProject"
	>
		<template #option="{ item }">
			<Avatar :src="item.iconUrl ?? undefined" size="1.25rem" no-shadow />
			<span class="min-w-0 flex-1 truncate font-semibold">{{ item.label }}</span>
			<TagItem class="shrink-0 font-mono">{{ item.value }}</TagItem>
		</template>
	</Combobox>
</template>

<script setup lang="ts">
import { Avatar, Combobox, type ComboboxOption, injectModrinthClient, TagItem } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { refDebounced } from '@vueuse/core'
import { computed, ref, shallowRef } from 'vue'

import { projectQueryOptions } from '~/composables/queries/project'

interface SampleProjectOption extends ComboboxOption<string> {
	iconUrl?: string | null
}

const projectId = defineModel<string>()
const client = injectModrinthClient()
const projects = client.labrinth.projects_v2
const searchInput = ref('')
const term = computed(() => searchInput.value.trim())
const debouncedTerm = refDebounced(term, 250)
const selectedResult = shallowRef<SampleProjectOption>()
const { data: selectedProject } = useQuery(
	computed(() => ({
		...projectQueryOptions.v2(projectId.value ?? '', client),
		enabled: Boolean(projectId.value),
	})),
)
const selectedOption = computed<SampleProjectOption | undefined>(() => {
	if (!projectId.value) return
	const project = selectedProject.value
	if (project?.id === projectId.value) {
		return { label: project.title, value: project.id, iconUrl: project.icon_url }
	}
	if (selectedResult.value?.value === projectId.value) return selectedResult.value
	return { label: projectId.value, value: projectId.value }
})

const { data, isPending, isError } = useQuery(
	computed(() => ({
		queryKey: ['moderation', 'project-flag', 'sample-project-search', debouncedTerm.value] as const,
		enabled: Boolean(debouncedTerm.value),
		queryFn: async ({ queryKey: [, , , query] }) => {
			const [indexed, lookup] = await Promise.allSettled([
				projects.search({ query, limit: 20 }),
				/^[A-Za-z0-9]+$/.test(query) ? projects.get(query) : undefined,
			])
			const exact = lookup.status === 'fulfilled' ? lookup.value : undefined
			const options = new Map<string, SampleProjectOption>()
			if (exact?.id === query) {
				options.set(exact.id, { label: exact.title, value: exact.id, iconUrl: exact.icon_url })
			}
			if (indexed.status === 'fulfilled') {
				for (const hit of indexed.value.hits) {
					if (options.has(hit.project_id)) continue
					options.set(hit.project_id, {
						label: hit.title,
						value: hit.project_id,
						iconUrl: hit.icon_url,
					})
				}
			} else if (!options.size) {
				throw indexed.reason
			}
			return [...options.values()]
		},
	})),
)
const searchOptions = computed(() =>
	term.value && term.value === debouncedTerm.value ? (data.value ?? []) : [],
)
const searchMessage = computed(() => {
	if (term.value !== debouncedTerm.value || isPending.value) return 'Searching…'
	return isError.value ? 'Could not search projects' : 'No projects found'
})

function onSearchInput(value: string) {
	searchInput.value = value
	if (!value.trim()) projectId.value = undefined
}

function selectProject(option: SampleProjectOption) {
	selectedResult.value = option
	searchInput.value = ''
}
</script>
