<script setup lang="ts">
import {
	GridIcon,
	ImageIcon,
	ListIcon,
	RotateCounterClockwiseIcon,
	SearchIcon,
} from '@modrinth/assets'
import { computed, toValue } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import { Button, IconButton } from '#ui/components/base/buttons'
import Combobox, { type ComboboxOption } from '#ui/components/base/Combobox.vue'
import Input from '#ui/components/base/inputs/Input.vue'
import NavTabs from '#ui/components/base/NavTabs.vue'
import Pagination from '#ui/components/base/Pagination.vue'
import SearchFilterControl from '#ui/components/search/SearchFilterControl.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages, formatProjectTypeSentence } from '#ui/utils/common-messages'
import type { FilterValue, SortType } from '#ui/utils/search'

import {
	SERVER_ENVIRONMENT_FILTER_ID,
	useServerEnvironmentFilter,
} from '../composables/use-server-environment-filter'
import { injectBrowseManager } from '../providers/browse-manager'

const ctx = injectBrowseManager()
const { view } = ctx
const { formatMessage } = useVIntl()

const messages = defineMessages({
	searchPlaceholder: {
		id: 'browse.search.placeholder',
		defaultMessage: 'Search {projectType}...',
	},
	viewPrefix: {
		id: 'browse.view-prefix',
		defaultMessage: 'View:',
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

const sortOptions = computed<ComboboxOption<SortType>[]>(() =>
	view.sortTypes.value.map((sortType) => ({ value: sortType, label: sortType.display })),
)

const maxResultsOptions = computed<ComboboxOption<number>[]>(() =>
	view.maxResultsOptions.value.map((count) => ({ value: count, label: String(count) })),
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
		<NavTabs v-if="ctx.projectTypes" :links="ctx.projectTypes.value" replace animate-collapse />

		<Input
			v-model="view.query.value"
			:icon="SearchIcon"
			type="text"
			autocomplete="off"
			:placeholder="searchPlaceholder"
			clearable
			wrapper-class="w-full"
			size="large"
			@clear="view.clearSearch()"
		/>

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

		<div class="flex flex-wrap items-center gap-2">
			<Combobox
				v-model="view.sort.value"
				:options="sortOptions"
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
				:options="maxResultsOptions"
				trigger-type="base"
				class="!w-[9rem] min-w-max max-w-full flex-grow md:flex-grow-0"
				:placeholder="formatMessage(commonMessages.viewLabel)"
			>
				<template #prefix>
					<span class="font-semibold text-primary">{{ formatMessage(messages.viewPrefix) }}</span>
				</template>
			</Combobox>

			<div v-if="ctx.filtersMenuOpen && !ctx.filtersMenuOpen.value" class="lg:hidden">
				<Button @click="ctx.filtersMenuOpen.value = true">
					{{ formatMessage(messages.filterResults) }}
				</Button>
			</div>

			<IconButton
				v-if="ctx.display"
				:label="formatMessage(messages.changeDisplayMode)"
				@click="ctx.display.cycle()"
			>
				<component :is="displayModeIcon" />
			</IconButton>

			<Pagination
				:page="view.page.value"
				:count="view.pageCount.value"
				class="mx-auto sm:ml-auto sm:mr-0"
				@switch-page="view.setPage"
			/>
		</div>

		<SearchFilterControl
			v-model:selected-filters="selectedFilters"
			:filters="filterTypes"
			:project-type="ctx.projectType.value"
			:provided-filters="view.isServerType.value ? [] : (ctx.providedFilters?.value ?? [])"
			:overridden-provided-filter-types="
				view.isServerType.value ? [] : view.overriddenProvidedFilterTypes.value
			"
			:provided-message="view.isServerType.value ? undefined : lockedMessages?.providedBy"
		/>
	</div>
</template>
