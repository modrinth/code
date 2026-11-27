<script setup lang="ts">
import { computed, ref, watch } from 'vue'

import Collapsible from '#ui/components/base/Collapsible.vue'
import DropdownFilterBar, {
	type DropdownFilterBarCategory,
} from '#ui/components/base/DropdownFilterBar.vue'

import type { ContentSortMode } from '../composables/content-sort'
import type { ContentMetadataFilterValue } from '../composables/use-content-metadata-filters'
import type { ContentType } from '../types'
import FilterMenu from './filter-menu.vue'
import SortMenu from './sort-menu.vue'
import TypeTabs from './type-tabs.vue'

const props = defineProps<{
	types: ContentType[]
	filterCategories: DropdownFilterBarCategory[]
	showDateSort: boolean
	flat?: boolean
}>()

const type = defineModel<ContentType>('type', { required: true })
const filters = defineModel<ContentMetadataFilterValue>('filters', { required: true })
const sort = defineModel<ContentSortMode>('sort', { required: true })

const hasActiveFilters = computed(() =>
	props.filterCategories.some((category) => (filters.value[category.key]?.length ?? 0) > 0),
)

const displayedFilters = ref<ContentMetadataFilterValue>(filters.value)
watch(filters, (value) => {
	if (hasActiveFilters.value) displayedFilters.value = value
})
</script>

<template>
	<div class="flex flex-col">
		<div class="flex flex-wrap items-center justify-between gap-2">
			<slot name="leading">
				<TypeTabs
					v-if="types.length > 1"
					v-model="type"
					:types="types"
					:class="{ '!shadow-none': flat }"
				/>
			</slot>
			<div class="ml-auto flex min-w-0 flex-wrap items-center justify-end gap-2">
				<FilterMenu
					v-if="filterCategories.length > 0"
					v-model="filters"
					:categories="filterCategories"
				/>
				<SortMenu v-model="sort" :show-date-options="showDateSort" />
				<template v-if="$slots.actions">
					<div class="h-6 w-px bg-surface-5" aria-hidden="true" />
					<slot name="actions" />
				</template>
			</div>
		</div>

		<Collapsible
			:collapsed="!hasActiveFilters"
			overflow-visible
			class="flex flex-wrap items-center gap-2 pt-2"
		>
			<DropdownFilterBar
				:model-value="displayedFilters"
				:categories="filterCategories"
				:show-label="false"
				:show-add-button="false"
				compact
				checkbox-position="right"
				apply-immediately
				@update:model-value="filters = $event"
			/>
		</Collapsible>
	</div>
</template>
