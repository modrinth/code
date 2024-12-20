<script setup lang="ts" generic="T">
import { ref, computed } from 'vue'
import type { Ref } from 'vue'
import Checkbox from '../base/Checkbox.vue'
import ContentListItem from './ContentListItem.vue'
import type { ContentItem } from './ContentListItem.vue'
import { DropdownIcon } from '@modrinth/assets'
import { createVirtualScroller } from 'vue-typed-virtual-list'

const props = withDefaults(
  defineProps<{
    items: ContentItem<T>[]
    sortColumn: string
    sortAscending: boolean
    updateSort: (column: string) => void
  }>(),
  {},
)

const VirtualScroller = createVirtualScroller()

const selectionStates: Ref<Record<string, boolean>> = ref({})
const selected: Ref<string[]> = computed(() =>
  Object.keys(selectionStates.value).filter(
    (item) => selectionStates.value[item] && props.items.some((x) => x.filename === item),
  ),
)

const allSelected = ref(false)

const model = defineModel()

function updateSelection() {
  model.value = selected.value
}

function setSelected(value: boolean) {
  if (value) {
    selectionStates.value = Object.fromEntries(props.items.map((item) => [item.filename, true]))
  } else {
    selectionStates.value = {}
  }
  updateSelection()
}
</script>

<template>
  <div class="flex flex-col grid-cols-[min-content,auto,auto,auto,auto]">
    <div
      :class="`${$slots.headers ? 'flex' : 'grid'} grid-cols-[min-content,4fr,3fr,2fr] gap-3 items-center px-2 pt-1 h-10 mb-3 text-contrast font-bold`"
    >
      <Checkbox
        v-model="allSelected"
        class="select-checkbox"
        :indeterminate="selected.length > 0 && selected.length < items.length"
        @update:model-value="setSelected"
      />
      <slot name="headers">
        <div class="flex items-center gap-2 cursor-pointer" @click="updateSort('Name')">
          Name
          <DropdownIcon
            v-if="sortColumn === 'Name'"
            class="transition-all transform"
            :class="{ 'rotate-180': sortAscending }"
          />
        </div>
        <div class="flex items-center gap-1 max-w-60 cursor-pointer" @click="updateSort('Updated')">
          Updated
          <DropdownIcon
            v-if="sortColumn === 'Updated'"
            class="transition-all transform"
            :class="{ 'rotate-180': sortAscending }"
          />
        </div>
        <div class="flex justify-end gap-2">
          <slot name="header-actions" />
        </div>
      </slot>
    </div>
    <div class="bg-bg-raised rounded-xl">
      <VirtualScroller :items="items" :default-size="64" style="height: 100%">
        <template #item="{ ref, index }">
          <ContentListItem
            v-model="selectionStates[ref.filename]"
            :item="ref"
            :last="index === items.length - 1"
            class="mb-2"
            @update:model-value="updateSelection"
          >
            <template #actions="{ item }">
              <slot name="actions" :item="item" />
            </template>
          </ContentListItem>
        </template>
      </VirtualScroller>
    </div>
  </div>
</template>
