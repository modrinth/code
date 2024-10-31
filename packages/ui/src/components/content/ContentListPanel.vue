<script setup lang="ts" generic="T">
import { ref, type Ref } from 'vue'
import Checkbox from '../base/Checkbox.vue'
import ContentListItem from './ContentListItem.vue'
import { type ContentItem } from './ContentListItem.vue'
import {
  DropdownIcon,
} from '@modrinth/assets'
// @ts-ignore
import { RecycleScroller } from 'vue-virtual-scroller'

defineProps<{
  items: ContentItem<T>[]
}>()

const selected: Ref<string[]> = ref([])

function toggle(filename: string) {
  setSelected(filename, !selected.value.includes(filename))
}

function setSelected(filename: string, value: boolean) {
  if (value) {
    selected.value.push(filename)
  } else {
    selected.value = selected.value.filter((item) => item !== filename)
  }
}
</script>

<template>
  <div class="flex flex-col grid-cols-[min-content,auto,auto,auto,auto]">
    <div
      class="grid grid-cols-[min-content,4fr,3fr,2fr] gap-3 items-center px-2 pt-1 pb-4 text-contrast font-bold"
    >
      <Checkbox :model-value="false" class="select-checkbox" />
      <div class="flex items-center gap-2">
<!--        <div class="w-[48px]"></div>-->
        Name <DropdownIcon />
      </div>
      <div class="flex items-center gap-1 max-w-60">Updated <DropdownIcon v-if="false" /></div>
      <div class="flex"></div>
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
