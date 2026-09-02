<template>
	<div class="flex flex-wrap items-center gap-1 empty:hidden">
		<TagItem
			v-if="selectedTagCount > 1"
			class="transition-transform active:scale-[0.95]"
			:action="clearFilters"
		>
			<XCircleIcon />
			Clear all filters
		</TagItem>
		<TagItem v-if="selectedIncludedProjectItems.length > 0" :action="removeIncludedProjectFilters">
			<XIcon />
			{{
				formatMessage(includedProjectsMessage, {
					projects: formatProjectNames(selectedIncludedProjectItems),
				})
			}}
		</TagItem>
		<TagItem v-if="selectedExcludedProjectItems.length > 0" :action="removeExcludedProjectFilters">
			<XIcon />
			<BanIcon class="text-brand-red" />
			{{
				formatMessage(excludedProjectsMessage, {
					projects: formatProjectNames(selectedExcludedProjectItems),
				})
			}}
		</TagItem>
		<TagItem
			v-for="selectedItem in otherSelectedItems"
			:key="`remove-filter-${selectedItem.type}-${selectedItem.option}`"
			:action="() => removeFilter(selectedItem)"
		>
			<XIcon />
			<BanIcon v-if="selectedItem.negative" class="text-brand-red" />
			{{ selectedItem.formatted_name ?? selectedItem.option }}
		</TagItem>
		<TagItem
			v-for="providedItem in items.filter(
				(x) => x.provided && !overriddenProvidedFilterTypes.includes(x.type),
			)"
			:key="`provided-filter-${providedItem.type}-${providedItem.option}`"
			v-tooltip="
				typeof providedMessage === 'string'
					? providedMessage
					: formatMessage(providedMessage ?? defaultProvidedMessage)
			"
			:style="{ '--_bg-color': `var(--color-raised-bg)` }"
		>
			<LockIcon />
			{{ providedItem.formatted_name ?? providedItem.option }}
		</TagItem>
	</div>
</template>

<script setup lang="ts">
import { BanIcon, LockIcon, XCircleIcon, XIcon } from '@modrinth/assets'
import { useQuery } from '@tanstack/vue-query'
import { computed, type ComputedRef } from 'vue'

import { defineMessage, type MessageDescriptor, useVIntl } from '../../composables/i18n'
import { injectModrinthClient } from '../../providers'
import {
	type FilterOption,
	type FilterType,
	type FilterValue,
	flattenFilterOptions,
	parseDependencyProjectFilterOption,
} from '../../utils/search'
import TagItem from '../base/TagItem.vue'

const { formatMessage } = useVIntl()
const { labrinth } = injectModrinthClient()

const selectedFilters = defineModel<FilterValue[]>('selectedFilters', { required: true })

const props = defineProps<{
	filters: FilterType[]
	projectType: string
	providedFilters: FilterValue[]
	overriddenProvidedFilterTypes: string[]
	providedMessage?: MessageDescriptor | string
}>()

const defaultProvidedMessage = defineMessage({
	id: 'search.filter.locked.default',
	defaultMessage: 'Filter locked',
})
const dependentProjectMessage = defineMessage({
	id: 'search.filter.dependent_project',
	defaultMessage: 'Depends on: {project}',
})
const includedProjectsMessage = defineMessage({
	id: 'search.filter.included_projects',
	defaultMessage: 'Includes: {projects}',
})
const excludedProjectsMessage = defineMessage({
	id: 'search.filter.excluded_projects',
	defaultMessage: 'Excludes: {projects}',
})

type Item = {
	type: string
	option: string
	negative?: boolean
	formatted_name?: string
	provided: boolean
}

const dependentProjectIds = computed(() =>
	[
		...new Set(
			[...selectedFilters.value, ...props.providedFilters]
				.filter((filter) => filter.type === 'compatible_dependency_project_ids')
				.map((filter) => parseDependencyProjectFilterOption(filter.option).projectId),
		),
	].sort(),
)

const { data: dependentProjects } = useQuery({
	queryKey: computed(() => [
		'search-filter-control',
		'dependent-projects',
		dependentProjectIds.value,
	]),
	queryFn: () => labrinth.projects_v2.getMultiple(dependentProjectIds.value),
	enabled: computed(() => dependentProjectIds.value.length > 0),
	placeholderData: [],
	refetchOnWindowFocus: false,
})

const dependentProjectNames = computed(
	() => new Map(dependentProjects.value?.map((project) => [project.id, project.title]) ?? []),
)

function filterMatches(type: FilterType, option: FilterOption, list: FilterValue[]) {
	return list.some((provided) => provided.type === type.id && provided.option === option.id)
}

const items: ComputedRef<Item[]> = computed(() => {
	return props.filters.flatMap((type) => {
		const optionItems = flattenFilterOptions(type.options)
			.filter(
				(option) =>
					filterMatches(type, option, selectedFilters.value) ||
					filterMatches(type, option, props.providedFilters),
			)
			.map((option) => ({
				type: type.id,
				option: option.id,
				negative: selectedFilters.value.find((x) => x.type === type.id && x.option === option.id)
					?.negative,
				provided: filterMatches(type, option, props.providedFilters),
				formatted_name: option.formatted_name,
			}))

		if (type.id !== 'compatible_dependency_project_ids') {
			return optionItems
		}

		const customValues = [...selectedFilters.value, ...props.providedFilters].filter(
			(filter) => filter.type === type.id,
		)
		return [
			...optionItems,
			...customValues.map((filter) => {
				const projectId = parseDependencyProjectFilterOption(filter.option).projectId
				return {
					type: type.id,
					option: filter.option,
					negative: filter.negative,
					provided: props.providedFilters.some(
						(provided) => provided.type === type.id && provided.option === filter.option,
					),
					formatted_name: formatMessage(dependentProjectMessage, {
						project: dependentProjectNames.value.get(projectId) ?? projectId,
					}),
				}
			}),
		]
	})
})

const selectedItems = computed(() => items.value.filter((x) => !x.provided))
const selectedIncludedProjectItems = computed(() =>
	props.projectType === 'modpack'
		? selectedItems.value.filter(
				(item) => item.type === 'compatible_dependency_project_ids' && !item.negative,
			)
		: [],
)
const selectedExcludedProjectItems = computed(() =>
	props.projectType === 'modpack'
		? selectedItems.value.filter(
				(item) => item.type === 'compatible_dependency_project_ids' && item.negative,
			)
		: [],
)
const otherSelectedItems = computed(() =>
	props.projectType === 'modpack'
		? selectedItems.value.filter((item) => item.type !== 'compatible_dependency_project_ids')
		: selectedItems.value,
)
const selectedTagCount = computed(
	() =>
		otherSelectedItems.value.length +
		(selectedIncludedProjectItems.value.length > 0 ? 1 : 0) +
		(selectedExcludedProjectItems.value.length > 0 ? 1 : 0),
)

function formatProjectNames(items: Item[]) {
	return items
		.map((item) => {
			const projectId = parseDependencyProjectFilterOption(item.option).projectId
			return dependentProjectNames.value.get(projectId) ?? projectId
		})
		.join(', ')
}

function removeFilter(filter: Item) {
	selectedFilters.value = selectedFilters.value.filter(
		(x) => x.type !== filter.type || x.option !== filter.option,
	)
}

function removeIncludedProjectFilters() {
	selectedFilters.value = selectedFilters.value.filter(
		(filter) => filter.type !== 'compatible_dependency_project_ids' || filter.negative,
	)
}

function removeExcludedProjectFilters() {
	selectedFilters.value = selectedFilters.value.filter(
		(filter) => filter.type !== 'compatible_dependency_project_ids' || !filter.negative,
	)
}

async function clearFilters() {
	selectedFilters.value = []
}
</script>
