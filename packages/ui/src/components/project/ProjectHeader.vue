<template>
  <ContentPageHeader>
    <template #icon>
      <Avatar :src="project.icon_url" :alt="project.title" size="96px" />
    </template>
    <template #title>
      {{ project.title }}
    </template>
    <template #title-suffix>
      <ProjectStatusBadge v-if="member || project.status !== 'approved'" :status="project.status" />
    </template>
    <template #summary>
      {{ project.description }}
    </template>
    <template #stats>
      <div
        v-tooltip="
          `${formatNumber(project.downloads, false)} download${project.downloads !== 1 ? 's' : ''}`
        "
        class="flex items-center gap-2 border-0 border-r border-solid border-divider pr-4 font-semibold cursor-help"
      >
        <DownloadIcon class="h-6 w-6 text-secondary" />
        {{ formatNumber(project.downloads) }}
      </div>
      <div
        v-tooltip="
          `${formatNumber(project.followers, false)} follower${project.downloads !== 1 ? 's' : ''}`
        "
        class="flex items-center gap-2 border-0 border-solid border-divider pr-4 cursor-help"
        :class="{ 'md:border-r': project.categories.length > 0 }"
      >
        <HeartIcon class="h-6 w-6 text-secondary" />
        <span class="font-semibold">
          {{ formatNumber(project.followers) }}
        </span>
      </div>
      <div v-if="project.categories.length > 0" class="hidden items-center gap-2 md:flex">
        <TagsIcon class="h-6 w-6 text-secondary" />
        <div class="flex flex-wrap gap-2">
          <TagItem
            v-for="(category, index) in project.categories"
            :key="index"
            :action="() => router.push(`/${project.project_type}s?f=categories:${category}`)"
          >
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
import Avatar from '../base/Avatar.vue'
import ContentPageHeader from '../base/ContentPageHeader.vue'
import { formatCategory, formatNumber, type Project } from '@modrinth/utils'
import { useRouter } from 'vue-router'
import TagItem from '../base/TagItem.vue'
import ProjectStatusBadge from './ProjectStatusBadge.vue'

const router = useRouter()

withDefaults(
  defineProps<{
    project: Project
    member?: boolean
  }>(),
  {
    member: false,
  },
)
</script>
