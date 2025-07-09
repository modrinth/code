<template>
  <div class="button-base p-4 bg-bg-raised rounded-xl flex gap-3 group">
    <div class="icon">
      <Avatar :src="project.icon_url" size="96px" class="search-icon" />
    </div>
    <div class="flex flex-col gap-2 overflow-hidden">
      <div class="gap-2 overflow-hidden no-wrap text-ellipsis">
        <span class="text-lg font-extrabold text-contrast m-0 leading-none">{{
          project.title
        }}</span>
        <span v-if="project.author" class="text-secondary"> by {{ project.author }}</span>
      </div>
      <div class="m-0 line-clamp-2">
        {{ project.description }}
      </div>
      <div class="mt-auto flex items-center gap-1 no-wrap">
        <TagsIcon class="h-4 w-4 shrink-0" />
        <div
          v-for="tag in categories"
          :key="tag"
          class="text-sm font-semibold text-secondary flex gap-1 px-[0.375rem] py-0.5 bg-button-bg rounded-full"
        >
          {{ formatCategory(tag) }}
        </div>
      </div>
    </div>
    <div class="flex flex-col gap-2 items-end shrink-0 ml-auto">
      <div class="flex items-center gap-2">
        <DownloadIcon class="shrink-0" />
        <span>
          {{ formatNumber(project.downloads) }}
          <span class="text-secondary">downloads</span>
        </span>
      </div>
      <div class="flex items-center gap-2">
        <HeartIcon class="shrink-0" />
        <span>
          {{ formatNumber(project.follows ?? project.followers) }}
          <span class="text-secondary">followers</span>
        </span>
      </div>
      <div class="mt-auto relative">
        <div
          :class="{
            'group-hover:-translate-y-3 group-hover:opacity-0 group-focus-within:opacity-0 group-hover:scale-95 group-focus-within:scale-95 transition-all':
              $slots.actions,
          }"
          class="flex items-center gap-2"
        >
          <HistoryIcon class="shrink-0" />
          <span>
            <span class="text-secondary">Updated</span>
            {{ formatRelativeTime(project.date_modified ?? project.updated) }}
          </span>
        </div>
        <div
          class="opacity-0 scale-95 translate-y-3 group-hover:translate-y-0 group-hover:scale-100 group-hover:opacity-100 group-focus-within:opacity-100 group-focus-within:scale-100 absolute bottom-0 right-0 transition-all w-fit"
        >
          <slot name="actions" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { TagsIcon, DownloadIcon, HeartIcon, HistoryIcon } from '@modrinth/assets'
import Avatar from '../base/Avatar.vue'
import { formatNumber, formatCategory } from '@modrinth/utils'
import { useRelativeTime } from '../../composables'

const formatRelativeTime = useRelativeTime()

defineProps({
  project: {
    type: Object,
    required: true,
  },
  categories: {
    type: Array,
    required: true,
  },
})
</script>
