<template>
  <div class="experimental-styles-within flex flex-wrap items-center gap-1 empty:hidden">
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
      v-for="providedItem in items.filter((x) => x.provided)"
      :key="`provided-filter-${providedItem.type}-${providedItem.option}`"
      v-tooltip="formatMessage(providedMessage ?? defaultProvidedMessage)"
      :style="{ '--_bg-color': `var(--color-raised-bg)` }"
    >
      <LockIcon />
      {{ providedItem.formatted_name ?? providedItem.option }}
    </TagItem>
  </div>
</template>

<script setup lang="ts">
import { XCircleIcon, XIcon, LockIcon, BanIcon } from '@modrinth/assets'
import { computed, type ComputedRef } from 'vue'
import TagItem from '../base/TagItem.vue'
import type { FilterValue, FilterType, FilterOption } from '../../utils/search'
import { defineMessage, type MessageDescriptor, useVIntl } from '@vintl/vintl'

const { formatMessage } = useVIntl()

const selectedFilters = defineModel<FilterValue[]>('selectedFilters', { required: true })

const props = defineProps<{
  filters: FilterType[]
  providedFilters: FilterValue[]
  overriddenProvidedFilterTypes: string[]
  providedMessage?: MessageDescriptor
}>()

const defaultProvidedMessage = defineMessage({
  id: 'search.filter.locked.default',
  defaultMessage: 'Filter locked',
})

type Item = {
  type: string
  option: string
  negative?: boolean
  formatted_name?: string
  provided: boolean
}

function filterMatches(type: FilterType, option: FilterOption, list: FilterValue[]) {
  return list.some((provided) => provided.type === type.id && provided.option === option.id)
}

const items: ComputedRef<Item[]> = computed(() => {
  return props.filters.flatMap((type) =>
    type.options
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
      })),
  )
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
