<script setup lang="ts" generic="T">
import { ref, computed } from 'vue'
import type { Ref } from 'vue'
import Checkbox from '../base/Checkbox.vue'
import ContentListItem from './ContentListItem.vue'
import { type ContentItem } from './ContentListItem.vue'
import {
  DropdownIcon,
} from '@modrinth/assets'
// @ts-ignore
import { RecycleScroller } from 'vue-virtual-scroller'

const props = withDefaults(defineProps<{
  items: ContentItem<T>[],
  locked?: boolean,
}>(), {
  locked: false,
})

const manualSelections: Ref<Record<string, boolean>> = ref({})
const selected: Ref<string[]> = computed(() => Object.keys(manualSelections.value).filter((item) => manualSelections.value[item]))

const allSelected = ref(false)

function setSelected(value: boolean) {
  for (const item of props.items) {
    manualSelections.value[item.filename] = value
  }
}

defineExpose({
  selected,
})
</script>

<template>
  <div class="flex flex-col grid-cols-[min-content,auto,auto,auto,auto]">
    <div
      :class="`${$slots.headers ? 'flex' : 'grid'} grid-cols-[min-content,4fr,3fr,2fr] gap-3 items-center px-2 pt-1 h-10 mb-3 text-contrast font-bold`"
    >
      <Checkbox v-if="!locked" v-model="allSelected" class="select-checkbox" @update:model-value="setSelected" :indeterminate="selected.length > 0 && selected.length < items.length" />
      <slot name="headers">
        <div class="flex items-center gap-2" :class="{ 'col-span-2': locked }">
          <!--        <div class="w-[48px]"></div>-->
          Name <DropdownIcon />
        </div>
        <div class="flex items-center gap-1 max-w-60">Updated <DropdownIcon v-if="false" /></div>
        <div class="flex">
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
        v-slot="{ item }"
      >
        <ContentListItem
          :item="item"
          :locked="locked"
          v-model="manualSelections[item.filename]"
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
