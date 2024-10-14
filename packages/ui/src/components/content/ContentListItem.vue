<script setup lang="ts" generic="T">
import AutoLink from '../base/AutoLink.vue'
import Avatar from '../base/Avatar.vue'
import Checkbox from '../base/Checkbox.vue'
import { type RouteLocationRaw } from 'vue-router'
import { SlashIcon } from '@modrinth/assets'

import { ref } from 'vue'

export interface ContentCreator {
  name: string
  type: 'user' | 'organization'
  id: string
  link?: string | RouteLocationRaw
  linkProps?: any
}

export interface ContentProject {
  id: string
  link?: string | RouteLocationRaw
  linkProps?: any
}

export interface ContentItem<T> {
  disabled: boolean
  filename: string
  data: T

  icon?: string
  title?: string
  project?: ContentProject
  creator?: ContentCreator

  version?: string
  versionId?: string
}

defineProps<{
  item: ContentItem<T>
}>()

const selected = ref(false)
const emit = defineEmits(['update:selected'])
</script>
<template>
  <div
    class="grid grid-cols-[min-content,4fr,2fr,3fr,min-content] gap-2 items-center rounded-xl bg-bg-raised p-2"
  >
    <Checkbox
      v-model="selected"
      :description="``"
      class="select-checkbox"
      @update:model-value="emit('update:selected', $event)"
    />
    <AutoLink :to="item.project?.link ?? ''" v-bind="item.project?.linkProps ?? {}">
      <div
        class="flex items-center gap-2 text-contrast font-medium"
        :class="{ 'opacity-50': item.disabled }"
      >
        <Avatar :src="item.icon ?? ''" :class="{ grayscale: item.disabled }" />
        <div class="flex flex-col">
          <span :class="item.disabled ? `line-clamp-1 line-through` : `line-clamp-2`">
            {{ item.title ?? item.filename }}
          </span>
          <span v-if="item.disabled" class="flex gap-1 items-center text-sm leading-tight">
            <SlashIcon /> Disabled
          </span>
        </div>
      </div>
    </AutoLink>
    <AutoLink :to="item.creator?.link ?? ''" v-bind="item.creator?.linkProps ?? {}">
      <div class="line-clamp-1 break-all text-primary" :class="{ 'opacity-50': item.disabled }">
        <slot v-if="item.creator" :item="item">
          {{ item.creator.name }}
        </slot>
      </div>
    </AutoLink>
    <div class="flex flex-col max-w-60" :class="{ 'opacity-50': item.disabled }">
      <div v-if="item.version" class="line-clamp-1 break-all">
        <slot :creator="item.creator">
          {{ item.version }}
        </slot>
      </div>
      <div class="text-secondary text-xs line-clamp-1 break-all">{{ item.filename }}</div>
    </div>
    <div class="flex gap-1">
      <slot name="actions" :item="item" />
    </div>
  </div>
</template>
