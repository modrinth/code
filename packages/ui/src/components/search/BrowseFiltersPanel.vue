<template>
  <div>
    <Accordion
      v-for="filter in filters"
      :key="filter.id"
      v-model="filters"
      v-bind="$attrs"
      :button-class="buttonClass"
      :content-class="contentClass"
      open-by-default
    >
      <template #title>
        <slot name="header" :filter="filter">
          <h2>{{ filter.formatted_name }}</h2>
        </slot>
      </template>
      <template #default>
        <template v-for="option in filter.options" :key="`${filter.id}-${option}`">
          <slot name="option" :filter="filter" :option="option">
            <div>
              {{ option.formatted_name }}
            </div>
          </slot>
        </template>
      </template>
    </Accordion>
  </div>
</template>

<script setup lang="ts">
import Accordion from '../base/Accordion.vue'
import { computed } from 'vue'

interface FilterOption<T> {
  id: string
  formatted_name: string
  data: T
}

interface FilterType<T> {
  id: string
  formatted_name: string
  scrollable?: boolean
  options: FilterOption<T>[]
}

interface GameVersion {
  version: string
  version_type: 'release' | 'snapshot' | 'alpha' | 'beta'
  date: string
  major: boolean
}

type ProjectType = 'mod' | 'modpack' | 'resourcepack' | 'shader' | 'datapack' | 'plugin'

interface Platform {
  name: string
  icon: string
  supported_project_types: ProjectType[]
  default: boolean
  formatted_name: string
}

const props = defineProps<{
  buttonClass?: string
  contentClass?: string
  gameVersions?: GameVersion[]
  platforms: Platform[]
}>()

const filters = computed(() => {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const filters: FilterType<any>[] = [
    {
      id: 'platform',
      formatted_name: 'Platform',
      options:
        props.platforms
          .filter((x) => x.default && x.supported_project_types.includes('modpack'))
          .map((x) => ({
            id: x.name,
            formatted_name: x.formatted_name,
            data: x,
          })) || [],
    },
    {
      id: 'gameVersion',
      formatted_name: 'Game version',
      options:
        props.gameVersions
          ?.filter((x) => x.major && x.version_type === 'release')
          .map((x) => ({
            id: x.version,
            formatted_name: x.version,
            data: x,
          })) || [],
    },
  ]

  return filters
})

defineOptions({
  inheritAttrs: false,
})
</script>
