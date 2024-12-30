<template>
  <div class="experimental-styles-within flex flex-col gap-3">
    <div class="flex flex-wrap items-center gap-2">
      <ManySelect
        v-model="selectedPlatforms"
        :options="filterOptions.platform"
        :dropdown-id="`${baseId}-platform`"
        @change="updateFilters"
      >
        <FilterIcon class="h-5 w-5 text-secondary" />
        Platform
        <template #option="{ option }">
          {{ formatCategory(option) }}
        </template>
      </ManySelect>
      <ManySelect
        v-model="selectedGameVersions"
        :options="filterOptions.gameVersion"
        :dropdown-id="`${baseId}-game-version`"
        search
        @change="updateFilters"
      >
        <FilterIcon class="h-5 w-5 text-secondary" />
        Game versions
        <template #footer>
          <Checkbox v-model="showSnapshots" class="mx-1" :label="`Show all versions`" />
        </template>
      </ManySelect>
      <ManySelect
        v-model="selectedChannels"
        :options="filterOptions.channel"
        :dropdown-id="`${baseId}-channel`"
        @change="updateFilters"
      >
        <FilterIcon class="h-5 w-5 text-secondary" />
        Channels
        <template #option="{ option }">
          {{ option === 'release' ? 'Release' : option === 'beta' ? 'Beta' : 'Alpha' }}
        </template>
      </ManySelect>
    </div>
    <div class="flex flex-wrap items-center gap-1 empty:hidden">
      <TagItem
        v-if="selectedChannels.length + selectedGameVersions.length + selectedPlatforms.length > 1"
        class="transition-transform active:scale-[0.95]"
        :action="clearFilters"
      >
        <XCircleIcon />
        Clear all filters
      </TagItem>
      <TagItem
        v-for="channel in selectedChannels"
        :key="`remove-filter-${channel}`"
        :style="`--_color: var(--color-${channel === 'alpha' ? 'red' : channel === 'beta' ? 'orange' : 'green'});--_bg-color: var(--color-${channel === 'alpha' ? 'red' : channel === 'beta' ? 'orange' : 'green'}-highlight)`"
        :action="() => toggleFilter('channel', channel)"
      >
        <XIcon />
        {{ channel.slice(0, 1).toUpperCase() + channel.slice(1) }}
      </TagItem>
      <TagItem
        v-for="version in selectedGameVersions"
        :key="`remove-filter-${version}`"
        :action="() => toggleFilter('gameVersion', version)"
      >
        <XIcon />
        {{ version }}
      </TagItem>
      <TagItem
        v-for="platform in selectedPlatforms"
        :key="`remove-filter-${platform}`"
        :style="`--_color: var(--color-platform-${platform})`"
        :action="() => toggleFilter('platform', platform)"
      >
        <XIcon />
        {{ formatCategory(platform) }}
      </TagItem>
    </div>
  </div>
</template>

<script setup lang="ts">
import { FilterIcon, XCircleIcon, XIcon } from '@modrinth/assets'
import { ManySelect, Checkbox } from '../index'
import { type Version, formatCategory, type GameVersionTag } from '@modrinth/utils'
import { ref, computed } from 'vue'
import { useRoute } from 'vue-router'
import TagItem from '../base/TagItem.vue'

const props = defineProps<{
  versions: Version[]
  gameVersions: GameVersionTag[]
  baseId?: string
}>()

const emit = defineEmits(['update:query'])

const allChannels = ref(['release', 'beta', 'alpha'])

const route = useRoute()

const showSnapshots = ref(false)

type FilterType = 'channel' | 'gameVersion' | 'platform'
type Filter = string

const filterOptions = computed(() => {
  const filters: Record<FilterType, Filter[]> = {
    channel: [],
    gameVersion: [],
    platform: [],
  }

  const platformSet = new Set()
  const gameVersionSet = new Set()
  const channelSet = new Set()

  for (const version of props.versions) {
    for (const loader of version.loaders) {
      platformSet.add(loader)
    }
    for (const gameVersion of version.game_versions) {
      gameVersionSet.add(gameVersion)
    }
    channelSet.add(version.version_type)
  }

  if (channelSet.size > 0) {
    filters.channel = Array.from(channelSet) as Filter[]
    filters.channel.sort((a, b) => allChannels.value.indexOf(a) - allChannels.value.indexOf(b))
  }
  if (gameVersionSet.size > 0) {
    const gameVersions = props.gameVersions.filter((x) => gameVersionSet.has(x.version))

    filters.gameVersion = gameVersions
      .filter((x) => (showSnapshots.value ? true : x.version_type === 'release'))
      .map((x) => x.version)
  }
  if (platformSet.size > 0) {
    filters.platform = Array.from(platformSet) as Filter[]
  }

  return filters
})

const selectedChannels = ref<string[]>([])
const selectedGameVersions = ref<string[]>([])
const selectedPlatforms = ref<string[]>([])

selectedChannels.value = route.query.c ? getArrayOrString(route.query.c) : []
selectedGameVersions.value = route.query.g ? getArrayOrString(route.query.g) : []
selectedPlatforms.value = route.query.l ? getArrayOrString(route.query.l) : []

async function toggleFilters(type: FilterType, filters: Filter[]) {
  for (const filter of filters) {
    await toggleFilter(type, filter, true)
  }

  updateFilters()
}

async function toggleFilter(type: FilterType, filter: Filter, bulk = false) {
  if (type === 'channel') {
    selectedChannels.value = selectedChannels.value.includes(filter)
      ? selectedChannels.value.filter((x) => x !== filter)
      : [...selectedChannels.value, filter]
  } else if (type === 'gameVersion') {
    selectedGameVersions.value = selectedGameVersions.value.includes(filter)
      ? selectedGameVersions.value.filter((x) => x !== filter)
      : [...selectedGameVersions.value, filter]
  } else if (type === 'platform') {
    selectedPlatforms.value = selectedPlatforms.value.includes(filter)
      ? selectedPlatforms.value.filter((x) => x !== filter)
      : [...selectedPlatforms.value, filter]
  }
  if (!bulk) {
    updateFilters()
  }
}

async function clearFilters() {
  selectedChannels.value = []
  selectedGameVersions.value = []
  selectedPlatforms.value = []

  updateFilters()
}

function updateFilters() {
  emit('update:query', {
    c: selectedChannels.value,
    g: selectedGameVersions.value,
    l: selectedPlatforms.value,
    page: undefined,
  })
}

defineExpose({
  toggleFilter,
  toggleFilters,
  selectedChannels,
  selectedGameVersions,
  selectedPlatforms,
})

function getArrayOrString(x: string | string[]): string[] {
  if (typeof x === 'string') {
    return [x]
  } else {
    return x
  }
}
</script>
