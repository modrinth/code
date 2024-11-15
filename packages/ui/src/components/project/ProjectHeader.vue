<template>
  <ContentPageHeader>
    <template #icon>
      <Avatar :src="project.icon_url" :alt="project.title" size="96px" />
    </template>
    <template #title>
      {{ project.title }}
    </template>
    <template #title-suffix>
      <Badge v-if="member || project.status !== 'approved'" type="project-status" :value="project.status" class="status-badge" />
    </template>
    <template #summary>
      {{ project.description }}
    </template>
    <template #stats>
      <div
        v-tooltip="`${formatNumber(project.downloads, false)} download${project.downloads !== 1 ? 's' : ''}`"
        class="flex items-center gap-2 border-0 border-r border-solid border-button-bg pr-4 font-semibold cursor-help"
      >
        <DownloadIcon class="h-6 w-6 text-secondary" />
        {{ formatNumber(project.downloads) }}
      </div>
      <div v-tooltip="`${formatNumber(project.followers, false)} follower${project.downloads !== 1 ? 's' : ''}`" class="flex items-center gap-2 border-0 border-solid border-button-bg pr-4 md:border-r cursor-help">
        <HeartIcon class="h-6 w-6 text-secondary" />
        <span class="font-semibold">
          {{ formatNumber(project.followers) }}
        </span>
      </div>
      <div class="hidden items-center gap-2 md:flex">
        <TagsIcon class="h-6 w-6 text-secondary" />
        <div class="flex flex-wrap gap-2">
          <TagItem v-for="(category, index) in project.categories" :key="index" class="tag-list__item">
            {{ formatCategory(category) }}
          </TagItem>
        </div>
      </div>
    </template>
    <template #actions>
      <slot name="actions" />
    </template>
  </ContentPageHeader>
</template>
<script setup lang="ts">
import { DownloadIcon, HeartIcon, TagsIcon } from '@modrinth/assets'
import Badge from '../base/Badge.vue'
import Avatar from '../base/Avatar.vue'
import ContentPageHeader from '../base/ContentPageHeader.vue'
import { formatCategory, formatNumber } from '@modrinth/utils'
import TagItem from '../base/TagItem.vue'

const props = withDefaults(defineProps<{
  project: {
    icon_url: string,
    title: string,
    status: string,
    description: string,
    downloads: number,
    followers: number,
    categories: string[],
  }
  member?: boolean,
}>(), {
  member: false,
})
</script>
