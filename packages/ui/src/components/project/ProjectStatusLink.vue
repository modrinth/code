<template>
  <AutoLink
    :to="`/project/${project.slug}`"
    :target="newTab ? '_blank' : undefined"
    class="group inline-flex min-w-0 max-w-full items-center gap-1"
    :style="{
      color: `var(--color-${getProjectStatusColor(getProjectDisplayStatus(project))})`,
    }"
  >
    <component
      :is="getProjectStatusIcon(getProjectDisplayStatus(project))"
      class="size-4 shrink-0"
    />
    <span class="min-w-0 truncate font-medium group-hover:underline">{{
      project.name
    }}</span>
  </AutoLink>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'

import {
  getProjectDisplayStatus,
  getProjectStatusColor,
  getProjectStatusIcon,
} from '../../utils'
import AutoLink from '../base/AutoLink.vue'

withDefaults(
  defineProps<{
    project: Pick<
      Labrinth.Projects.v3.Project,
      'name' | 'slug' | 'visibility' | 'in_review' | 'review_outcome'
    >
    newTab?: boolean
  }>(),
  {
    newTab: false,
  },
)
</script>
