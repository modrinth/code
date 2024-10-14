<script setup lang="ts" generic="T">
import { ref, computed, type Ref } from 'vue'
import Checkbox from '../base/Checkbox.vue'
import ContentListItem from './ContentListItem.vue'
import { type ContentItem } from './ContentListItem.vue'
import {
  CheckCircleIcon,
  ClipboardCopyIcon,
  DropdownIcon,
  MoreVerticalIcon,
  SlashIcon
} from '@modrinth/assets'
import { ButtonStyled, OverflowMenu } from '../index'

const props = defineProps<{
  items: ContentItem<T>[],
}>()

const selected: Ref<string[]> = ref([])

function toggle(filename: string) {
  setSelected(filename, !selected.value.includes(filename))
}

function setSelected(filename: string, value: boolean) {
  if (value) {
    selected.value.push(filename)
  } else {
    selected.value = selected.value.filter(item => item !== filename)
  }
}
</script>

<template>
  <div class="flex flex-col grid-cols-[min-content,auto,auto,auto,auto] gap-2">
    <div class="grid grid-cols-[min-content,4fr,2fr,3fr,auto] gap-2 items-center px-2 pb-2 text-contrast font-bold">
      <Checkbox :model-value="false" class="select-checkbox" />
      <div class="flex items-center gap-1"><div class="w-[36px]"></div>Name <DropdownIcon /></div>
      <div class="flex items-center gap-1">Owner <DropdownIcon v-if="false" /></div>
      <div class="flex items-center gap-1 max-w-60">Updated <DropdownIcon v-if="false" /></div>
      <div class="flex w-[36px]">
      </div>
    </div>
    <ContentListItem v-for="item in items.slice().sort((a, b) => a.filename.localeCompare(b.filename))" :item="item">
      <template #actions="{ item }">
        <slot name="actions" :item="item" />
      </template>
    </ContentListItem>
  </div>
</template>
