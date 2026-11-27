<script setup lang="ts">
import { FilterIcon, SearchIcon } from '@modrinth/assets'
import { computed, ref, watch } from 'vue'

import Collapsible from '#ui/components/base/Collapsible.vue'
import DropdownFilterBar, {
	type DropdownFilterBarCategory,
} from '#ui/components/base/DropdownFilterBar.vue'
import Input from '#ui/components/base/inputs/Input.vue'
import Pagination from '#ui/components/base/Pagination.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

export type ListToolbarValue = Record<string, string[]>

const props = withDefaults(
	defineProps<{
		searchPlaceholder?: string
		searchLabel?: string
		filterCategories?: DropdownFilterBarCategory[]
		page?: number
		pageCount?: number
		pageLink?: (page: number) => string | undefined
		filtersShown?: boolean
	}>(),
	{
		searchPlaceholder: undefined,
		searchLabel: undefined,
		filterCategories: () => [],
		page: 1,
		pageCount: 0,
		pageLink: undefined,
		filtersShown: undefined,
	},
)

const search = defineModel<string>('search')
const filters = defineModel<ListToolbarValue>('filters', { default: () => ({}) })

const emit = defineEmits<{
	clearSearch: []
	switchPage: [page: number]
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	filter: {
		id: 'content.page-layout.filter.add',
		defaultMessage: 'Filter',
	},
})

const filterButtonClass =
	'!size-10 !rounded-full !border-0 !bg-transparent !shadow-none !p-0 !text-primary hover:!bg-surface-4 focus-visible:!outline-none focus-visible:!ring-4 focus-visible:!ring-brand-shadow [&>svg]:!size-5'

const hasActiveFilters = computed(() =>
	props.filterCategories.some((category) => (filters.value[category.key]?.length ?? 0) > 0),
)

const showFilterRow = computed(() => props.filtersShown ?? hasActiveFilters.value)

const displayedFilters = ref<ListToolbarValue>(filters.value)
watch(filters, (value) => {
	if (hasActiveFilters.value) displayedFilters.value = value
})
</script>

<template>
	<div class="flex flex-col">
		<div class="flex flex-wrap items-center justify-between gap-x-4 gap-y-2">
			<slot name="leading">
				<Input
					v-if="search !== undefined"
					v-model="search"
					:icon="SearchIcon"
					type="text"
					autocomplete="off"
					:spellcheck="false"
					size="medium"
					wrapper-class="min-w-[12rem] flex-1"
					clearable
					:clear-label="formatMessage(commonMessages.clearButton)"
					:aria-label="searchLabel ?? searchPlaceholder"
					:placeholder="searchPlaceholder"
					@clear="emit('clearSearch')"
				/>
			</slot>
			<div class="ml-auto flex min-w-0 flex-wrap items-center justify-end gap-2">
				<div
					v-if="filterCategories.length > 0"
					class="flex min-w-0 flex-wrap items-center justify-end gap-1.5 [&_[data-button]]:!h-10"
				>
					<DropdownFilterBar
						v-model="filters"
						:categories="filterCategories"
						:show-label="false"
						:add-label="formatMessage(messages.filter)"
						:add-button-class="filterButtonClass"
						:show-previews="false"
						:show-clear-button="false"
						add-button-size="lg"
						checkbox-position="right"
						menu-align="end"
						apply-immediately
					>
						<template #add-button>
							<FilterIcon aria-hidden="true" />
						</template>
						<template #option="{ option, selected: optionSelected }">
							<span
								class="min-w-0 truncate font-semibold leading-tight"
								:class="optionSelected ? 'text-contrast' : 'text-primary'"
							>
								{{ option.label }}
							</span>
						</template>
						<template v-if="$slots['filter-footer']" #category-footer="{ category }">
							<slot name="filter-footer" :category="category" />
						</template>
					</DropdownFilterBar>
				</div>
				<slot name="controls" />
				<template v-if="$slots.actions">
					<div class="h-6 w-px bg-surface-5" aria-hidden="true" />
					<slot name="actions" />
				</template>
				<template v-if="pageCount > 1">
					<div class="h-6 w-px bg-surface-5" aria-hidden="true" />
					<Pagination
						:page="page"
						:count="pageCount"
						:link-function="pageLink"
						animate-width
						@switch-page="(nextPage) => emit('switchPage', nextPage)"
					/>
				</template>
			</div>
		</div>

		<Collapsible :collapsed="!showFilterRow" :duration-ms="200" overflow-visible fade>
			<div class="flex flex-wrap items-center gap-2 pt-2">
				<slot name="filters">
					<DropdownFilterBar
						:model-value="displayedFilters"
						:categories="filterCategories"
						:show-label="false"
						:show-add-button="false"
						compact
						checkbox-position="right"
						apply-immediately
						@update:model-value="filters = $event"
					>
						<template v-if="$slots['filter-footer']" #preview-footer="{ category }">
							<slot name="filter-footer" :category="category" />
						</template>
					</DropdownFilterBar>
				</slot>
			</div>
		</Collapsible>
	</div>
</template>
