<script setup lang="ts" generic="T">
import AutoLink from '../base/AutoLink.vue'
import Avatar from '../base/Avatar.vue'
import Checkbox from '../base/Checkbox.vue'
import type { RouteLocationRaw } from 'vue-router'

export interface ContentCreator {
  name: string
  type: 'user' | 'organization'
  id: string
  link?: string | RouteLocationRaw
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  linkProps?: any
}

export interface ContentProject {
  id: string
  link?: string | RouteLocationRaw
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  linkProps?: any
}

export interface ContentItem<T> {
  path: string
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

withDefaults(
  defineProps<{
    item: ContentItem<T>
    last?: boolean
  }>(),
  {
    last: false,
  },
)

const model = defineModel<boolean>()
</script>
<template>
  <div
    class="grid grid-cols-[min-content,4fr,3fr,2fr] gap-3 items-center p-2 h-[64px] border-solid border-0 border-b-button-bg relative"
    :class="{ 'border-b-[1px]': !last }"
  >
    <Checkbox v-model="model" :description="``" class="select-checkbox" />
    <div
      class="flex items-center gap-2 text-contrast font-medium"
      :class="{ 'opacity-50': item.disabled }"
    >
      <AutoLink :to="item.project?.link ?? ''" tabindex="-1" v-bind="item.project?.linkProps ?? {}">
        <Avatar :src="item.icon ?? ''" :class="{ grayscale: item.disabled }" size="48px" />
      </AutoLink>
      <div class="flex flex-col">
        <AutoLink :to="item.project?.link ?? ''" v-bind="item.project?.linkProps ?? {}">
          <div class="text-contrast line-clamp-1" :class="{ 'line-through': item.disabled }">
            {{ item.title ?? item.filename }}
          </div>
        </AutoLink>
        <AutoLink :to="item.creator?.link ?? ''" v-bind="item.creator?.linkProps ?? {}">
          <div class="line-clamp-1 break-all" :class="{ 'opacity-50': item.disabled }">
            <slot v-if="item.creator && item.creator.name" :item="item">
              <span class="text-secondary"> by {{ item.creator.name }} </span>
            </slot>
          </div>
        </AutoLink>
      </div>
    </div>
    <div class="flex flex-col max-w-60" :class="{ 'opacity-50': item.disabled }">
      <div v-if="item.version" class="line-clamp-1 break-all">
        <slot :creator="item.creator">
          {{ item.version }}
        </slot>
      </div>
      <div class="text-secondary text-xs line-clamp-1 break-all">{{ item.filename }}</div>
    </div>
    <div class="flex justify-end gap-1 items-center">
      <slot name="actions" :item="item" />
    </div>
  </div>
</template>
