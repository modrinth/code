<script setup lang="ts" generic="T">
import { ref, computed } from 'vue'
import type { Ref } from 'vue'
import Checkbox from '../base/Checkbox.vue'
import ContentListItem from './ContentListItem.vue'
import { type ContentItem } from './ContentListItem.vue'
import { DropdownIcon } from '@modrinth/assets'
// @ts-ignore
import { RecycleScroller } from 'vue-virtual-scroller'

const props = withDefaults(
  defineProps<{
    items: ContentItem<T>[]
    locked?: boolean
  }>(),
  {
    locked: false,
  },
)

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
        v-if="!locked"
        v-model="allSelected"
        class="select-checkbox"
        @update:model-value="setSelected"
        :indeterminate="selected.length > 0 && selected.length < items.length"
      />
      <slot name="headers">
        <div class="flex items-center gap-2" :class="{ 'col-span-2': locked }">
          <!--        <div class="w-[48px]"></div>-->
          Name <DropdownIcon />
        </div>
        <div class="flex items-center gap-1 max-w-60">Updated <DropdownIcon v-if="false" /></div>
        <div class="flex justify-end gap-2">
          <slot name="header-actions" />
        </div>
      </slot>
    </div>
    <div class="bg-bg-raised rounded-xl">
      <RecycleScroller
        :items="items.slice().sort((a, b) => a.filename.localeCompare(b.filename))"
        :item-size="64"
        disable-transform
        key-field="filename"
        style="height: 100%"
        v-slot="{ item, index }"
      >
        <ContentListItem
          :item="item"
          :locked="locked"
          v-model="selectionStates[item.filename]"
          @update:model-value="updateSelection"
          :last="props.items.length - 1 === index"
          class="mb-2"
        >
          <template #actions="{ item }">
            <slot name="actions" :item="item" />
          </template>
        </ContentListItem>
      </RecycleScroller>
    </div>
  </div>
</template>
