<template>
	<div class="flex flex-wrap items-center gap-1 empty:hidden">
		<TagItem
			v-if="selectedItems.length > 1"
			class="transition-transform active:scale-[0.95]"
			:action="clearFilters"
		>
			<XCircleIcon />
			Clear all filters
		</TagItem>
		<TagItem
			v-for="selectedItem in selectedItems"
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
import type { FilterOption, FilterType, FilterValue } from '../../utils/search'
import TagItem from '../base/TagItem.vue'

const { formatMessage } = useVIntl()
const { labrinth } = injectModrinthClient()

const selectedFilters = defineModel<FilterValue[]>('selectedFilters', { required: true })

const props = defineProps<{
	filters: FilterType[]
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
				.map((filter) => filter.option),
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
		const optionItems = type.options
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
			...customValues.map((filter) => ({
				type: type.id,
				option: filter.option,
				negative: filter.negative,
				provided: props.providedFilters.some(
					(provided) => provided.type === type.id && provided.option === filter.option,
				),
				formatted_name: formatMessage(dependentProjectMessage, {
					project: dependentProjectNames.value.get(filter.option) ?? filter.option,
				}),
			})),
		]
	})
})

const selectedItems = computed(() => items.value.filter((x) => !x.provided))

function removeFilter(filter: Item) {
	selectedFilters.value = selectedFilters.value.filter(
		(x) => x.type !== filter.type || x.option !== filter.option,
	)
}

async function clearFilters() {
	selectedFilters.value = []
}
</script>
