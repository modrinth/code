<template>
	<Accordion
		v-bind="$attrs"
		ref="accordion"
		:button-class="buttonClass ?? 'flex flex-col gap-2 justify-start items-start'"
		:content-class="contentClass"
		title-wrapper-class="flex flex-col gap-2 justify-start items-start"
		:open-by-default="openByDefault !== undefined ? openByDefault : true"
	>
		<template #button="{ open }">
			<div class="flex items-center gap-1 w-full text-contrast">
				<slot name="header" :filter="filterType">
					<h2 class="text-base font-semibold text-red">{{ filterType.formatted_name }}</h2>
				</slot>
				<DropdownIcon
					class="ml-auto size-5 transition-transform duration-300 shrink-0 text-primary group-hover:text-contrast"
					:class="{ 'rotate-180': open }"
				/>
			</div>
		</template>
		<template
			v-if="
				locked ||
				(!!accordion &&
					!accordion.isOpen &&
					(selectedFilterOptions.length > 0 || selectedNegativeFilterOptions.length > 0))
			"
			#summary
		>
			<div class="flex gap-1 flex-wrap">
				<div
					v-for="option in selectedFilterOptions"
					:key="`selected-filter-${filterType.id}-${option.id}`"
					class="flex gap-1 text-xs bg-button-bg px-2 py-0.5 rounded-full font-bold text-secondary w-fit shrink-0 items-center"
				>
					{{ option.formatted_name ?? option.id }}
				</div>
				<div
					v-for="option in selectedNegativeFilterOptions"
					:key="`excluded-filter-${filterType.id}-${option.id}`"
					class="flex gap-1 text-xs bg-button-bg px-2 py-0.5 rounded-full font-bold text-secondary w-fit shrink-0 items-center"
				>
					<BanIcon class="text-brand-red" /> {{ option.formatted_name ?? option.id }}
				</div>
			</div>
		</template>
		<template v-if="locked" #default>
			<div class="flex flex-col gap-2 p-3 border-dashed border-2 rounded-2xl border-divider">
				<p class="m-0 font-bold items-center">
					<slot :name="`locked-${filterType.id}`">
						{{ formatMessage(messages.lockedTitle, { type: filterType.formatted_name }) }}
					</slot>
				</p>
				<p class="m-0 text-secondary text-sm">
					{{ formatMessage(messages.lockedDescription) }}
				</p>
				<Button
					class="w-fit"
					@click="
						() => {
							overriddenProvidedFilterTypes.push(filterType.id)
						}
					"
				>
					<LockOpenIcon />
					{{ formatMessage(messages.unlockFilterButton) }}
				</Button>
			</div>
		</template>
		<template v-else #default>
			<slot name="prefix" />
			<SearchDependsOnFilter
				v-if="filterType.display === 'depends-on-project'"
				v-model:selected-filters="selectedFilters"
				:project-type="projectType"
				:inner-panel-class="innerPanelClass"
				:selected-project-class="selectedProjectClass"
				:result-count="resultCount"
				:loading="loading"
				:refreshing="refreshing"
			/>
			<template v-if="filterType.display !== 'depends-on-project'">
				<StyledInput
					v-if="filterType.searchable"
					:id="`search-${filterType.id}`"
					v-model="query"
					:icon="SearchIcon"
					type="text"
					:placeholder="formatMessage(messages.searchPlaceholder)"
					autocomplete="off"
					clearable
					input-class="!bg-button-bg"
					wrapper-class="mx-2 my-1 w-[calc(100%-1rem)]"
				/>
				<ScrollablePanel :class="{ 'h-[16rem]': scrollable }" :disable-scrolling="!scrollable">
					<div :class="innerPanelClass ? innerPanelClass : ''" class="flex flex-col gap-1">
						<template v-if="groupedOptions">
							<SearchFilterGroup
								v-for="[groupName, options] in groupedOptions"
								:key="`${filterType.id}-group-${groupName}`"
								:group-name="groupName"
								:options="options"
								:supports="filterType.supports"
								:included="isIncluded"
								:excluded="isExcluded"
								@toggle="toggleFilter"
								@toggle-exclude="toggleNegativeFilter"
							/>
						</template>
						<template v-else>
							<template v-for="option in visibleOptions" :key="`${filterType.id}-${option.id}`">
								<SearchFilterOption
									:option="option"
									:included="isIncluded(option)"
									:excluded="isExcluded(option)"
									:supports="filterType.supports"
									:has-sub-options="!!option.sub_options?.length"
									:expanded="isExpanded(option)"
									:class="{
										'mr-3': scrollable,
									}"
									@toggle="toggleFilter"
									@toggle-exclude="toggleNegativeFilter"
									@toggle-expand="toggleExpand(option)"
								>
									<slot name="option" :filter="filterType" :option="option">
										<span
											v-if="option.icon"
											class="inline-flex items-center justify-center shrink-0 h-4 w-4"
											:style="iconStyle(option)"
										>
											<div
												v-if="typeof option.icon === 'string'"
												class="h-4 w-4"
												v-html="option.icon"
											/>
											<component :is="option.icon" v-else class="h-4 w-4" />
										</span>
										<span class="truncate text-sm" :style="iconStyle(option)">
											{{ option.formatted_name ?? option.id }}
										</span>
									</slot>
								</SearchFilterOption>
								<div
									v-if="option.sub_options?.length && isExpanded(option)"
									class="ml-4 flex flex-col gap-1"
									:class="{ 'mr-3': scrollable }"
								>
									<SearchFilterOption
										v-for="subOption in option.sub_options"
										:key="`${filterType.id}-${subOption.id}`"
										:option="subOption"
										:included="isIncluded(subOption)"
										:excluded="isExcluded(subOption)"
										:supports="filterType.supports"
										@toggle="toggleFilter"
										@toggle-exclude="toggleNegativeFilter"
									>
										<slot name="option" :filter="filterType" :option="subOption">
											<span class="truncate text-sm">
												{{ subOption.formatted_name ?? subOption.id }}
											</span>
										</slot>
									</SearchFilterOption>
								</div>
							</template>
						</template>
						<button
							v-if="filterType.display === 'expandable'"
							class="flex bg-transparent text-secondary border-none cursor-pointer !w-full items-center gap-2 truncate rounded-xl px-2 py-1 text-sm font-semibold transition-all hover:text-contrast focus-visible:text-contrast active:scale-[0.98]"
							@click="showMore = !showMore"
						>
							<DropdownIcon
								class="h-4 w-4 transition-transform"
								:class="{ 'rotate-180': showMore }"
							/>
							<span class="truncate text-sm">
								{{
									showMore ? formatMessage(messages.showFewer) : formatMessage(messages.showMore)
								}}
							</span>
						</button>
					</div>
				</ScrollablePanel>
			</template>
			<div :class="innerPanelClass ? innerPanelClass : ''" class="empty:hidden">
				<Checkbox
					v-for="group in filterType.toggle_groups"
					:key="`toggle-group-${group.id}`"
					class="mx-2"
					:model-value="groupEnabled(group.id)"
					:label="`${group.formatted_name}`"
					@update:model-value="toggleGroup(group.id)"
				/>
				<div v-if="hasProvidedFilter" class="mt-2 mx-1">
					<Button
						class="w-fit"
						@click="
							() => {
								overriddenProvidedFilterTypes = overriddenProvidedFilterTypes.filter(
									(id) => id !== filterType.id,
								)
								accordion?.close()
								clearFilters()
							}
						"
					>
						<UpdatedIcon />
						<slot name="sync-button">
							{{ formatMessage(messages.syncFilterButton) }}
						</slot>
					</Button>
				</div>
			</div>
		</template>
	</Accordion>
</template>

<script setup lang="ts">
import { BanIcon, DropdownIcon, LockOpenIcon, SearchIcon, UpdatedIcon } from '@modrinth/assets'
import { computed, ref, watch } from 'vue'

import { Button } from '#ui/components/base/buttons'

import { defineMessages, useVIntl } from '../../composables/i18n'
import {
	type FilterOption,
	type FilterType,
	type FilterValue,
	findParentFilterOption,
	flattenFilterOptions,
} from '../../utils/search'
import Accordion from '../base/Accordion.vue'
import { Checkbox, ScrollablePanel, StyledInput } from '../index'
import SearchDependsOnFilter from './SearchDependsOnFilter.vue'
import SearchFilterGroup from './SearchFilterGroup.vue'
import SearchFilterOption from './SearchFilterOption.vue'

const { formatMessage } = useVIntl()

const selectedFilters = defineModel<FilterValue[]>('selectedFilters', { required: true })
const toggledGroups = defineModel<string[]>('toggledGroups', { required: true })
const overriddenProvidedFilterTypes = defineModel<string[]>('overriddenProvidedFilterTypes', {
	required: false,
	default: [],
})

const props = defineProps<{
	filterType: FilterType
	projectType: string
	buttonClass?: string
	contentClass?: string
	innerPanelClass?: string
	selectedProjectClass?: string
	openByDefault?: boolean
	providedFilters: FilterValue[]
	resultCount?: number
	loading?: boolean
	refreshing?: boolean
}>()

defineOptions({
	inheritAttrs: false,
})

const query = ref('')
const showMore = ref(false)
const expandedOptionIds = ref<string[]>([])

const accordion = ref<InstanceType<typeof Accordion> | null>()

const allOptions = computed(() => flattenFilterOptions(props.filterType.options))

const selectedFilterOptions = computed(() =>
	allOptions.value.filter((option) =>
		locked.value ? isProvided(option, false) : isIncluded(option),
	),
)
const selectedNegativeFilterOptions = computed(() =>
	allOptions.value.filter((option) =>
		locked.value ? isProvided(option, true) : isExcluded(option),
	),
)
const visibleOptions = computed(() =>
	props.filterType.options
		.filter(
			(option) =>
				isVisible(option) ||
				isIncluded(option) ||
				isExcluded(option) ||
				hasSelectedSubOption(option),
		)
		.slice()
		.sort((a, b) => {
			if (props.filterType.display === 'expandable') {
				const aDefault = props.filterType.default_values.includes(a.id)
				const bDefault = props.filterType.default_values.includes(b.id)

				if (aDefault && !bDefault) {
					return -1
				} else if (!aDefault && bDefault) {
					return 1
				}
			}
			return 0
		}),
)

const hasGroups = computed(() => visibleOptions.value.some((o) => o.group))
const groupedOptions = computed(() => {
	if (!hasGroups.value) return null
	const groups = new Map<string, FilterOption[]>()
	for (const option of visibleOptions.value) {
		const groupName = option.group ?? ''
		if (!groups.has(groupName)) {
			groups.set(groupName, [])
		}
		groups.get(groupName)!.push(option)
	}
	return groups
})

const hasProvidedFilter = computed(() =>
	props.providedFilters.some((filter) => filter.type === props.filterType.id),
)
const locked = computed(
	() =>
		hasProvidedFilter.value && !overriddenProvidedFilterTypes.value.includes(props.filterType.id),
)

const scrollable = computed(
	() => visibleOptions.value.length >= 10 && props.filterType.display === 'scrollable',
)

function iconStyle(option: FilterOption) {
	// Match project page platform coloring (Forge/Fabric/Velocity/etc.) while leaving other
	// filter icons unchanged.
	if (
		props.filterType.id === 'mod_loader' ||
		props.filterType.id === 'modpack_loader' ||
		props.filterType.id === 'plugin_loader' ||
		props.filterType.id === 'plugin_platform' ||
		props.filterType.id === 'shader_loader'
	) {
		return { color: `var(--color-platform-${option.id})` }
	}

	return undefined
}

function groupEnabled(group: string) {
	return toggledGroups.value.includes(group)
}

function toggleGroup(group: string) {
	if (toggledGroups.value.includes(group)) {
		toggledGroups.value = toggledGroups.value.filter((x) => x !== group)
	} else {
		toggledGroups.value.push(group)
	}
}

function isIncluded(filter: FilterOption) {
	return selectedFilters.value.some((value) => value.option === filter.id && !value.negative)
}

function isExcluded(filter: FilterOption) {
	return selectedFilters.value.some((value) => value.option === filter.id && value.negative)
}

function hasSelectedSubOption(filter: FilterOption) {
	return (
		filter.sub_options?.some((subOption) => isIncluded(subOption) || isExcluded(subOption)) ?? false
	)
}

function isExpanded(filter: FilterOption) {
	return expandedOptionIds.value.includes(filter.id)
}

function toggleExpand(filter: FilterOption) {
	if (isExpanded(filter)) {
		expandedOptionIds.value = expandedOptionIds.value.filter((id) => id !== filter.id)
	} else {
		expandedOptionIds.value = [...expandedOptionIds.value, filter.id]
	}
}

function isVisible(filter: FilterOption) {
	const filterKey = filter.formatted_name?.toLowerCase() ?? filter.id.toLowerCase()
	const matchesQuery = !query.value || filterKey.includes(query.value.toLowerCase())

	if (props.filterType.display === 'expandable') {
		return matchesQuery && (showMore.value || props.filterType.default_values.includes(filter.id))
	}

	if (filter.toggle_group) {
		return toggledGroups.value.includes(filter.toggle_group) && matchesQuery
	} else {
		return matchesQuery
	}
}

function isProvided(filter: FilterOption, negative: boolean) {
	return props.providedFilters.some(
		(x) => x.type === props.filterType.id && x.option === filter.id && !x.negative === !negative,
	)
}

type FilterState = 'include' | 'exclude' | 'ignore'

function toggleFilter(filter: FilterOption) {
	setFilter(filter, isIncluded(filter) || isExcluded(filter) ? 'ignore' : 'include')
}

function toggleNegativeFilter(filter: FilterOption) {
	setFilter(filter, isExcluded(filter) ? 'ignore' : 'exclude')
}

function setFilter(filter: FilterOption, state: FilterState) {
	let newFilters = selectedFilters.value.filter((selected) => selected.option !== filter.id)

	if (state !== 'ignore') {
		const subOptionIds = new Set(filter.sub_options?.map((subOption) => subOption.id) ?? [])
		if (subOptionIds.size > 0) {
			newFilters = newFilters.filter((selected) => !subOptionIds.has(selected.option))
		}

		const parent = findParentFilterOption(props.filterType.options, filter.id)
		if (parent) {
			newFilters = newFilters.filter((selected) => selected.option !== parent.id)
			if (!expandedOptionIds.value.includes(parent.id)) {
				expandedOptionIds.value = [...expandedOptionIds.value, parent.id]
			}
		}
	}

	const baseValues = {
		type: props.filterType.id,
		option: filter.id,
	}

	if (state === 'include') {
		newFilters.push({
			...baseValues,
			negative: false,
		})
	} else if (state === 'exclude') {
		newFilters.push({
			...baseValues,
			negative: true,
		})
	}

	selectedFilters.value = newFilters
}

function clearFilters() {
	selectedFilters.value = selectedFilters.value.filter(
		(filter) => filter.type !== props.filterType.id,
	)
}

watch(
	selectedFilters,
	() => {
		for (const option of props.filterType.options) {
			if (hasSelectedSubOption(option) && !expandedOptionIds.value.includes(option.id)) {
				expandedOptionIds.value = [...expandedOptionIds.value, option.id]
			}
		}
	},
	{ deep: true, immediate: true },
)

const messages = defineMessages({
	searchPlaceholder: {
		id: 'search.filter.option.search.placeholder',
		defaultMessage: 'Search...',
	},
	clearSearchAriaLabel: {
		id: 'search.filter.option.search.clear.aria_label',
		defaultMessage: 'Clear search',
	},
	showFewer: {
		id: 'search.filter.option.show_fewer',
		defaultMessage: 'Show fewer',
	},
	showMore: {
		id: 'search.filter.option.show_more',
		defaultMessage: 'Show more',
	},
	unlockFilterButton: {
		id: 'search.filter.locked.default.unlock',
		defaultMessage: 'Unlock filter',
	},
	syncFilterButton: {
		id: 'search.filter.locked.default.sync',
		defaultMessage: 'Sync filter',
	},
	lockedTitle: {
		id: 'search.filter.locked.default.title',
		defaultMessage: '{type} is locked',
	},
	lockedDescription: {
		id: 'search.filter.locked.default.description',
		defaultMessage: 'Unlocking this filter may allow you to install incompatible content.',
	},
})
</script>
