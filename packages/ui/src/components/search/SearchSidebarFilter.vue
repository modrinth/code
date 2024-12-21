<template>
  <Accordion
    v-bind="$attrs"
    ref="accordion"
    :button-class="buttonClass ?? 'flex flex-col gap-2 justify-start items-start'"
    :content-class="contentClass"
    title-wrapper-class="flex flex-col gap-2 justify-start items-start"
    :open-by-default="!locked && (openByDefault !== undefined ? openByDefault : true)"
  >
    <template #title>
      <slot name="header" :filter="filterType">
        <h2>{{ filterType.formatted_name }}</h2>
      </slot>
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
          :key="`selected-filter-${filterType.id}-${option}`"
          class="flex gap-1 text-xs bg-button-bg px-2 py-0.5 rounded-full font-bold text-secondary w-fit shrink-0 items-center"
        >
          {{ option.formatted_name ?? option.id }}
        </div>
        <div
          v-for="option in selectedNegativeFilterOptions"
          :key="`excluded-filter-${filterType.id}-${option}`"
          class="flex gap-1 text-xs bg-button-bg px-2 py-0.5 rounded-full font-bold text-secondary w-fit shrink-0 items-center"
        >
          <BanIcon class="text-brand-red" /> {{ option.formatted_name ?? option.id }}
        </div>
      </div>
    </template>
    <template v-if="locked" #default>
      <div class="flex flex-col gap-2 p-3 border-dashed border-2 rounded-2xl border-divider mx-2">
        <p class="m-0 font-bold items-center">
          <slot :name="`locked-${filterType.id}`">
            {{ formatMessage(messages.lockedTitle, { type: filterType.formatted_name }) }}
          </slot>
        </p>
        <p class="m-0 text-secondary text-sm">
          {{ formatMessage(messages.lockedDescription) }}
        </p>
        <ButtonStyled>
          <button
            class="w-fit"
            @click="
              () => {
                overriddenProvidedFilterTypes.push(filterType.id)
              }
            "
          >
            <LockOpenIcon />
            {{ formatMessage(messages.unlockFilterButton) }}
          </button>
        </ButtonStyled>
      </div>
    </template>
    <template v-else #default>
      <div v-if="filterType.searchable" class="iconified-input mx-2 my-1 !flex">
        <SearchIcon aria-hidden="true" />
        <input
          :id="`search-${filterType.id}`"
          v-model="query"
          class="!min-h-9 text-sm"
          type="text"
          :placeholder="`Search...`"
          autocomplete="off"
        />
        <Button v-if="query" class="r-btn" aria-label="Clear search" @click="() => (query = '')">
          <XIcon aria-hidden="true" />
        </Button>
      </div>

      <ScrollablePanel :class="{ 'h-[16rem]': scrollable }" :disable-scrolling="!scrollable">
        <div :class="innerPanelClass ? innerPanelClass : ''" class="flex flex-col gap-1">
          <SearchFilterOption
            v-for="option in visibleOptions"
            :key="`${filterType.id}-${option}`"
            :option="option"
            :included="isIncluded(option)"
            :excluded="isExcluded(option)"
            :supports-negative-filter="filterType.supports_negative_filter"
            :class="{
              'mr-3': scrollable,
            }"
            @toggle="toggleFilter"
            @toggle-exclude="toggleNegativeFilter"
          >
            <slot name="option" :filter="filterType" :option="option">
              <div v-if="typeof option.icon === 'string'" class="h-4 w-4" v-html="option.icon" />
              <component :is="option.icon" v-else-if="option.icon" class="h-4 w-4" />
              <span class="truncate text-sm">{{ option.formatted_name ?? option.id }}</span>
            </slot>
          </SearchFilterOption>
          <button
            v-if="filterType.display === 'expandable'"
            class="flex bg-transparent text-secondary border-none cursor-pointer !w-full items-center gap-2 truncate rounded-xl px-2 py-1 text-sm font-semibold transition-all hover:text-contrast focus-visible:text-contrast active:scale-[0.98]"
            @click="showMore = !showMore"
          >
            <DropdownIcon
              class="h-4 w-4 transition-transform"
              :class="{ 'rotate-180': showMore }"
            />
            <span class="truncate text-sm">{{ showMore ? 'Show fewer' : 'Show more' }}</span>
          </button>
        </div>
      </ScrollablePanel>
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
          <ButtonStyled>
            <button
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
            </button>
          </ButtonStyled>
        </div>
      </div>
    </template>
  </Accordion>
</template>

<script setup lang="ts">
import Accordion from '../base/Accordion.vue'
import type { FilterOption, FilterType, FilterValue } from '../../utils/search'
import {
  BanIcon,
  SearchIcon,
  XIcon,
  UpdatedIcon,
  LockOpenIcon,
  DropdownIcon,
} from '@modrinth/assets'
import { Button, Checkbox, ScrollablePanel } from '../index'
import { computed, ref } from 'vue'
import ButtonStyled from '../base/ButtonStyled.vue'
import SearchFilterOption from './SearchFilterOption.vue'
import { defineMessages, useVIntl } from '@vintl/vintl'

const { formatMessage } = useVIntl()

const selectedFilters = defineModel<FilterValue[]>('selectedFilters', { required: true })
const toggledGroups = defineModel<string[]>('toggledGroups', { required: true })
const overriddenProvidedFilterTypes = defineModel<string[]>('overriddenProvidedFilterTypes', {
  required: false,
  default: [],
})

const props = defineProps<{
  filterType: FilterType
  buttonClass?: string
  contentClass?: string
  innerPanelClass?: string
  openByDefault?: boolean
  providedFilters: FilterValue[]
}>()

defineOptions({
  inheritAttrs: false,
})

const query = ref('')
const showMore = ref(false)

const accordion = ref<InstanceType<typeof Accordion> | null>()

const selectedFilterOptions = computed(() =>
  props.filterType.options.filter((option) =>
    locked.value ? isProvided(option, false) : isIncluded(option),
  ),
)
const selectedNegativeFilterOptions = computed(() =>
  props.filterType.options.filter((option) =>
    locked.value ? isProvided(option, true) : isExcluded(option),
  ),
)
const visibleOptions = computed(() =>
  props.filterType.options
    .filter((option) => isVisible(option) || isIncluded(option) || isExcluded(option))
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
  const newFilters = selectedFilters.value.filter((selected) => selected.option !== filter.id)

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

const messages = defineMessages({
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
