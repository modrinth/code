<script setup lang="ts">
import {
	ArrowUpDownIcon,
	FilterIcon,
	GridIcon,
	ImageIcon,
	ListIcon,
	RotateCounterClockwiseIcon,
	SearchIcon,
	SlidersVerticalIcon,
} from '@modrinth/assets'
import { computed, toValue } from 'vue'
import { useRoute } from 'vue-router'

import Admonition from '#ui/components/base/Admonition.vue'
import {
	Button,
	type ButtonMenuOption,
	IconButton,
	TeleportOverflowMenu,
} from '#ui/components/base/buttons'
import Combobox, { type ComboboxOption } from '#ui/components/base/Combobox.vue'
import Input from '#ui/components/base/inputs/Input.vue'
import Pagination from '#ui/components/base/Pagination.vue'
import ToolbarTabs, { type ToolbarTab } from '#ui/components/base/ToolbarTabs.vue'
import SearchFilterControl from '#ui/components/search/SearchFilterControl.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages, formatProjectTypeSentence } from '#ui/utils/common-messages'
import type { FilterValue, SortType } from '#ui/utils/search'

import {
	SERVER_ENVIRONMENT_FILTER_ID,
	useServerEnvironmentFilter,
} from '../composables/use-server-environment-filter'
import { injectBrowseManager } from '../providers/browse-manager'

defineProps<{
	stacked?: boolean
}>()

const ctx = injectBrowseManager()
const { view } = ctx
const route = useRoute()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	searchPlaceholder: {
		id: 'browse.search.placeholder',
		defaultMessage: 'Search {projectType}...',
	},
	projectTypes: {
		id: 'browse.project-types',
		defaultMessage: 'Project types',
	},
	resultsPerPage: {
		id: 'browse.results-per-page',
		defaultMessage: 'Results per page',
	},
	resultsPerPageCurrent: {
		id: 'browse.results-per-page.current',
		defaultMessage: 'Results per page: {count}',
	},
	resultsPerPageOption: {
		id: 'browse.results-per-page.option',
		defaultMessage: '{count, plural, one {# item} other {# items}}',
	},
	viewPrefix: {
		id: 'browse.view-prefix',
		defaultMessage: 'View:',
	},
	sortCurrent: {
		id: 'browse.sort.current',
		defaultMessage: 'Sort: {mode}',
	},
	filterResults: {
		id: 'browse.filter-results',
		defaultMessage: 'Filter results...',
	},
	changeDisplayMode: {
		id: 'browse.change-display-mode',
		defaultMessage: 'Change display mode',
	},
	linkOverridingPreferences: {
		id: 'browse.advanced-filters.link-overriding-preferences',
		defaultMessage: "This link's filters differ from your saved advanced exclusions",
	},
	applySavedPreferences: {
		id: 'browse.advanced-filters.apply-saved-preferences',
		defaultMessage: 'Apply saved preferences',
	},
})

const lockedMessages = computed(() => toValue(ctx.lockedFilterMessages))
const { filterType: environmentFilterType, selection: environmentSelection } =
	useServerEnvironmentFilter(ctx)

const environmentShown = computed(
	() => !view.isServerType.value && !!ctx.serverEnvironment?.shown.value,
)

const filterTypes = computed(() => {
	if (view.isServerType.value) return view.filterTypes.value
	const hidden = ctx.hiddenFilterTypes?.value ?? []
	return [
		...(environmentShown.value ? [environmentFilterType.value] : []),
		...view.filterTypes.value.filter(
			(filter) => filter.display !== 'none' && !hidden.includes(filter.id),
		),
	]
})

const selectedFilters = computed<FilterValue[]>({
	get: () => [...environmentSelection.value, ...view.filters.value],
	set: (filters) => {
		const environment = filters.filter((filter) => filter.type === SERVER_ENVIRONMENT_FILTER_ID)
		if (environment.length !== environmentSelection.value.length) {
			environmentSelection.value = environment
		}
		view.filters.value = filters.filter((filter) => filter.type !== SERVER_ENVIRONMENT_FILTER_ID)
	},
})

function hrefPath(href: string) {
	return decodeURIComponent(href.split('?')[0])
}

const projectTypeTabs = computed<ToolbarTab[]>(() =>
	(ctx.projectTypes?.value ?? [])
		.filter((link) => link.shown ?? true)
		.map((link) => ({ value: hrefPath(link.href), label: link.label, to: link.href })),
)

const activeProjectTypeTab = computed(() => {
	const path = decodeURIComponent(route.path)
	return projectTypeTabs.value.find((tab) => path === tab.value || path.startsWith(`${tab.value}/`))
		?.value
})

const sortOptions = computed<ButtonMenuOption[]>(() =>
	view.sortTypes.value.map((sortType) => ({
		id: sortType.name,
		label: sortType.display,
		selected: sortType.name === view.sort.value.name,
		action: () => (view.sort.value = sortType),
	})),
)

const maxResultsOptions = computed<ButtonMenuOption[]>(() => [
	{ type: 'heading', label: formatMessage(messages.resultsPerPage) },
	...view.maxResultsOptions.value.map((count) => ({
		id: String(count),
		label: String(count),
		selected: count === view.maxResults.value,
		action: () => (view.maxResults.value = count),
	})),
])

const sortComboboxOptions = computed<ComboboxOption<SortType>[]>(() =>
	view.sortTypes.value.map((sortType) => ({ value: sortType, label: sortType.display })),
)

const maxResultsComboboxOptions = computed<ComboboxOption<number>[]>(() =>
	view.maxResultsOptions.value.map((count) => ({
		value: count,
		label: formatMessage(messages.resultsPerPageOption, { count }),
	})),
)

const displayModeIcon = computed(() => {
	const mode = ctx.display?.mode.value
	return mode === 'grid' ? GridIcon : mode === 'gallery' ? ImageIcon : ListIcon
})

const searchPlaceholder = computed(() =>
	formatMessage(messages.searchPlaceholder, {
		projectType: formatProjectTypeSentence(formatMessage, ctx.projectType.value, 2),
	}),
)
</script>

<template>
	<div class="flex flex-col gap-4">
		<ToolbarTabs
			v-if="projectTypeTabs.length > 1"
			:model-value="activeProjectTypeTab"
			:tabs="projectTypeTabs"
			:label="formatMessage(messages.projectTypes)"
			replace
			class="self-start"
		/>

		<template v-if="stacked">
			<Input
				v-model="view.query.value"
				:icon="SearchIcon"
				type="text"
				autocomplete="off"
				:spellcheck="false"
				:placeholder="searchPlaceholder"
				clearable
				wrapper-class="w-full"
				size="large"
				@clear="view.clearSearch()"
			/>

			<div class="flex flex-wrap items-center gap-2">
				<Combobox
					v-model="view.sort.value"
					:options="sortComboboxOptions"
					trigger-type="base"
					class="!w-[16rem] min-w-max max-w-full flex-grow md:flex-grow-0"
				>
					<template #prefix>
						<span class="font-semibold text-primary">
							{{ formatMessage(commonMessages.sortByLabel) }}
						</span>
					</template>
				</Combobox>

				<Combobox
					v-model="view.maxResults.value"
					:options="maxResultsComboboxOptions"
					trigger-type="base"
					class="!w-[11rem] min-w-max max-w-full flex-grow md:flex-grow-0"
				>
					<template #prefix>
						<span class="font-semibold text-primary">{{ formatMessage(messages.viewPrefix) }}</span>
					</template>
				</Combobox>

				<Button
					v-if="ctx.filtersMenuOpen && !ctx.filtersMenuOpen.value"
					class="lg:hidden"
					@click="ctx.filtersMenuOpen.value = true"
				>
					{{ formatMessage(messages.filterResults) }}
				</Button>

				<IconButton
					v-if="ctx.display"
					:label="formatMessage(messages.changeDisplayMode)"
					@click="ctx.display.cycle()"
				>
					<component :is="displayModeIcon" aria-hidden="true" />
				</IconButton>

				<Pagination
					:page="view.page.value"
					:count="view.pageCount.value"
					class="mx-auto sm:ml-auto sm:mr-0"
					@switch-page="view.setPage"
				/>
			</div>
		</template>

		<div v-else class="flex flex-wrap items-center gap-x-4 gap-y-2">
			<Input
				v-model="view.query.value"
				:icon="SearchIcon"
				type="text"
				autocomplete="off"
				:spellcheck="false"
				:placeholder="searchPlaceholder"
				clearable
				size="medium"
				wrapper-class="min-w-[12rem] flex-1"
				@clear="view.clearSearch()"
			/>

			<div class="ml-auto flex min-w-0 flex-wrap items-center justify-end gap-2">
				<IconButton
					v-if="ctx.filtersMenuOpen && !ctx.filtersMenuOpen.value"
					v-tooltip="formatMessage(messages.filterResults)"
					type="quiet"
					size="lg"
					class="lg:hidden"
					:label="formatMessage(messages.filterResults)"
					@click="ctx.filtersMenuOpen.value = true"
				>
					<FilterIcon aria-hidden="true" />
				</IconButton>

				<TeleportOverflowMenu
					:tooltip="formatMessage(messages.resultsPerPage)"
					type="quiet"
					size="lg"
					:label="formatMessage(messages.resultsPerPageCurrent, { count: view.maxResults.value })"
					:options="maxResultsOptions"
				>
					<SlidersVerticalIcon aria-hidden="true" />
				</TeleportOverflowMenu>

				<IconButton
					v-if="ctx.display"
					v-tooltip="formatMessage(messages.changeDisplayMode)"
					type="quiet"
					size="lg"
					:label="formatMessage(messages.changeDisplayMode)"
					@click="ctx.display.cycle()"
				>
					<component :is="displayModeIcon" aria-hidden="true" />
				</IconButton>

				<TeleportOverflowMenu
					type="quiet"
					size="lg"
					:label="formatMessage(messages.sortCurrent, { mode: view.sort.value.display })"
					:options="sortOptions"
					:icon-only="false"
					animate-width
				>
					<ArrowUpDownIcon aria-hidden="true" />
					{{ view.sort.value.display }}
				</TeleportOverflowMenu>

				<template v-if="view.pageCount.value > 1">
					<div class="h-6 w-px bg-surface-4" aria-hidden="true" />
					<Pagination
						:page="view.page.value"
						:count="view.pageCount.value"
						animate-width
						@switch-page="view.setPage"
					/>
				</template>
			</div>
		</div>

		<Admonition
			v-if="view.linkOverridesAdvancedPrefs.value"
			type="info"
			:header="formatMessage(messages.linkOverridingPreferences)"
			inline-actions
			center-content
		>
			<template #actions>
				<Button type="colored" color="blue" @click="view.applySavedAdvancedPrefs()">
					<RotateCounterClockwiseIcon />
					{{ formatMessage(messages.applySavedPreferences) }}
				</Button>
			</template>
		</Admonition>

		<SearchFilterControl
			v-model:selected-filters="selectedFilters"
			:filters="filterTypes"
			:provided-filters="view.isServerType.value ? [] : (ctx.providedFilters?.value ?? [])"
			:overridden-provided-filter-types="
				view.isServerType.value ? [] : view.overriddenProvidedFilterTypes.value
			"
			:provided-message="view.isServerType.value ? undefined : lockedMessages?.providedBy"
		/>
	</div>
</template>
