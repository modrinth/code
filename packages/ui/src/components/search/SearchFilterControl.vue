<template>
	<div class="flex flex-wrap items-center gap-2 empty:hidden">
		<div
			v-for="locked in lockedPills"
			:key="`locked-filter-${locked.key}`"
			v-tooltip="lockedTooltip"
			class="flex h-9 min-w-0 max-w-[16rem] cursor-default select-none items-center gap-2 rounded-xl border border-dashed border-surface-5 px-3 text-base"
		>
			<LockIcon class="size-4 shrink-0 text-secondary" aria-hidden="true" />
			<span class="min-w-0 flex-1 truncate">
				<span class="font-medium text-secondary">{{ locked.label }}:</span>
				<span class="ml-1 font-semibold text-primary">{{ locked.summary }}</span>
			</span>
		</div>
		<DropdownFilterBar
			v-if="categories.length > 0"
			:model-value="selectedValue"
			:categories="categories"
			:show-label="false"
			:show-add-button="false"
			checkbox-position="right"
			apply-immediately
			@update:model-value="updateSelectedFilters"
		>
			<template #preview-content="{ category, label, summary }">
				<span class="flex min-w-0 flex-1 items-center gap-1.5 truncate">
					<BanIcon
						v-if="isExcludedKey(category.key)"
						class="size-4 shrink-0 text-brand-red"
						aria-hidden="true"
					/>
					<span class="min-w-0 truncate">
						<span class="font-medium">{{ label }}:</span>
						<span class="ml-1 font-semibold text-contrast">{{ summary }}</span>
					</span>
				</span>
			</template>
		</DropdownFilterBar>
	</div>
</template>

<script setup lang="ts">
import { BanIcon, LockIcon } from '@modrinth/assets'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'

import { defineMessages, type MessageDescriptor, useVIntl } from '../../composables/i18n'
import { injectModrinthClient } from '../../providers'
import {
	type FilterType,
	type FilterValue,
	flattenFilterOptions,
	parseDependencyProjectFilterOption,
} from '../../utils/search'
import DropdownFilterBar, {
	type DropdownFilterBarCategory,
	type DropdownFilterBarOption,
} from '../base/DropdownFilterBar.vue'

const { formatMessage } = useVIntl()
const { labrinth } = injectModrinthClient()

const selectedFilters = defineModel<FilterValue[]>('selectedFilters', { required: true })

const props = defineProps<{
	filters: FilterType[]
	providedFilters: FilterValue[]
	overriddenProvidedFilterTypes: string[]
	providedMessage?: MessageDescriptor | string
}>()

const messages = defineMessages({
	defaultProvided: {
		id: 'search.filter.locked.default',
		defaultMessage: 'Filter locked',
	},
	excludedCategory: {
		id: 'search.filter.excluded_category',
		defaultMessage: '{name} (excluded)',
	},
})

const DEPENDENCY_FILTER_ID = 'compatible_dependency_project_ids'
const EXCLUDED_KEY_SUFFIX = ':excluded'

const lockedTooltip = computed(() =>
	typeof props.providedMessage === 'string'
		? props.providedMessage
		: formatMessage(props.providedMessage ?? messages.defaultProvided),
)

const dependentProjectIds = computed(() =>
	[
		...new Set(
			[...selectedFilters.value, ...props.providedFilters]
				.filter((filter) => filter.type === DEPENDENCY_FILTER_ID)
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

function categoryKey(typeId: string, negative?: boolean) {
	return negative ? `${typeId}${EXCLUDED_KEY_SUFFIX}` : typeId
}

function isExcludedKey(key: string) {
	return key.endsWith(EXCLUDED_KEY_SUFFIX)
}

function parseCategoryKey(key: string) {
	const negative = isExcludedKey(key)
	return { typeId: negative ? key.slice(0, -EXCLUDED_KEY_SUFFIX.length) : key, negative }
}

function formatOptionLabel(type: FilterType, option: string) {
	if (type.id === DEPENDENCY_FILTER_ID) {
		const projectId = parseDependencyProjectFilterOption(option).projectId
		return dependentProjectNames.value.get(projectId) ?? projectId
	}
	return (
		flattenFilterOptions(type.options).find((candidate) => candidate.id === option)
			?.formatted_name ?? option
	)
}

function getTypeOptions(type: FilterType, values: FilterValue[]): DropdownFilterBarOption[] {
	const options = flattenFilterOptions(type.options).map((option) => ({
		value: option.id,
		label: option.formatted_name ?? option.id,
	}))
	const known = new Set(options.map((option) => option.value))
	const custom = values
		.filter((filter) => filter.type === type.id && !known.has(filter.option))
		.map((filter) => ({ value: filter.option, label: formatOptionLabel(type, filter.option) }))
	return [...options, ...custom]
}

const selectedValue = computed<Record<string, string[]>>(() => {
	const value: Record<string, string[]> = {}
	for (const filter of selectedFilters.value) {
		const key = categoryKey(filter.type, filter.negative)
		;(value[key] ??= []).push(filter.option)
	}
	return value
})

const categories = computed<DropdownFilterBarCategory[]>(() =>
	props.filters.flatMap((type) => {
		const typeFilters = selectedFilters.value.filter((filter) => filter.type === type.id)
		if (typeFilters.length === 0) return []

		const options = getTypeOptions(type, typeFilters)
		const base = {
			options,
			searchable: type.searchable,
			direct: type.display === 'radio',
		}
		const result: DropdownFilterBarCategory[] = []
		if (typeFilters.some((filter) => !filter.negative)) {
			result.push({ ...base, key: categoryKey(type.id), label: type.formatted_name })
		}
		if (typeFilters.some((filter) => filter.negative)) {
			result.push({
				...base,
				key: categoryKey(type.id, true),
				label: formatMessage(messages.excludedCategory, { name: type.formatted_name }),
			})
		}
		return result
	}),
)

const lockedPills = computed(() =>
	props.filters.flatMap((type) => {
		if (props.overriddenProvidedFilterTypes.includes(type.id)) return []
		const provided = props.providedFilters.filter((filter) => filter.type === type.id)
		if (provided.length === 0) return []
		return [
			{
				key: type.id,
				label: type.formatted_name,
				summary: provided.map((filter) => formatOptionLabel(type, filter.option)).join(', '),
			},
		]
	}),
)

function updateSelectedFilters(next: Record<string, string[]>) {
	const handledKeys = new Set(Object.keys(next))
	const kept = selectedFilters.value.filter(
		(filter) => !handledKeys.has(categoryKey(filter.type, filter.negative)),
	)
	const updated = Object.entries(next).flatMap(([key, options]) => {
		const { typeId, negative } = parseCategoryKey(key)
		return options.map((option) => ({ type: typeId, option, ...(negative ? { negative } : {}) }))
	})
	selectedFilters.value = [...kept, ...updated]
}
</script>
