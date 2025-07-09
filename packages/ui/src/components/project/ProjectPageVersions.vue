<template>
  <div class="mb-3 flex flex-wrap gap-2">
    <VersionFilterControl
      ref="versionFilters"
      :versions="versions"
      :game-versions="gameVersions"
      :base-id="`${baseId}-filter`"
      @update:query="updateQuery"
    />
    <Pagination
      :page="currentPage"
      class="ml-auto mt-auto"
      :count="Math.ceil(filteredVersions.length / pageSize)"
      @switch-page="switchPage"
    />
  </div>
  <div
    v-if="versions.length > 0"
    class="flex flex-col gap-4 rounded-2xl bg-bg-raised px-6 pb-8 pt-4 supports-[grid-template-columns:subgrid]:grid supports-[grid-template-columns:subgrid]:grid-cols-[1fr_min-content] sm:px-8 supports-[grid-template-columns:subgrid]:sm:grid-cols-[min-content_auto_auto_auto_min-content] supports-[grid-template-columns:subgrid]:xl:grid-cols-[min-content_auto_auto_auto_auto_auto_min-content]"
  >
    <div class="versions-grid-row">
      <div class="w-9 max-sm:hidden"></div>
      <div class="text-sm font-bold text-contrast max-sm:hidden">Name</div>
      <div
        class="text-sm font-bold text-contrast max-sm:hidden sm:max-xl:collapse sm:max-xl:hidden"
      >
        Game version
      </div>
      <div
        class="text-sm font-bold text-contrast max-sm:hidden sm:max-xl:collapse sm:max-xl:hidden"
      >
        Platforms
      </div>
      <div
        class="text-sm font-bold text-contrast max-sm:hidden sm:max-xl:collapse sm:max-xl:hidden"
      >
        Published
      </div>
      <div
        class="text-sm font-bold text-contrast max-sm:hidden sm:max-xl:collapse sm:max-xl:hidden"
      >
        Downloads
      </div>
      <div class="text-sm font-bold text-contrast max-sm:hidden xl:collapse xl:hidden">
        Compatibility
      </div>
      <div class="text-sm font-bold text-contrast max-sm:hidden xl:collapse xl:hidden">Stats</div>
      <div class="w-9 max-sm:hidden"></div>
    </div>
    <template v-for="(version, index) in currentVersions" :key="index">
      <!-- Row divider -->
      <div
        class="versions-grid-row h-px w-full bg-button-bg"
        :class="{
          'max-sm:!hidden': index === 0,
        }"
      ></div>
      <div class="versions-grid-row group relative">
        <AutoLink
          v-if="!!versionLink"
          class="absolute inset-[calc(-1rem-2px)_-2rem] before:absolute before:inset-0 before:transition-all before:content-[''] hover:before:backdrop-brightness-110"
          :to="versionLink?.(version)"
        />
        <div class="flex flex-col justify-center gap-2 sm:contents">
          <div class="flex flex-row items-center gap-2 sm:contents">
            <div class="self-center">
              <div class="relative z-[1] cursor-pointer">
                <VersionChannelIndicator
                  v-tooltip="`Toggle filter for ${version.version_type}`"
                  :channel="version.version_type"
                  @click="versionFilters?.toggleFilter('channel', version.version_type)"
                />
              </div>
            </div>
            <div
              class="pointer-events-none relative z-[1] flex flex-col justify-center"
              :class="{
                'group-hover:underline': !!versionLink,
              }"
            >
              <div class="font-bold text-contrast">{{ version.version_number }}</div>
              <div class="text-xs font-medium">{{ version.name }}</div>
            </div>
          </div>
          <div class="flex flex-col justify-center gap-2 sm:contents">
            <div class="flex flex-row flex-wrap items-center gap-1 xl:contents">
              <div class="flex items-center">
                <div class="flex flex-wrap gap-1">
                  <TagItem
                    v-for="gameVersion in formatVersionsForDisplay(
                      version.game_versions,
                      gameVersions,
                    )"
                    :key="`version-tag-${gameVersion}`"
                    v-tooltip="`Toggle filter for ${gameVersion}`"
                    class="z-[1]"
                    :action="
                      () => versionFilters?.toggleFilters('gameVersion', version.game_versions)
                    "
                  >
                    {{ gameVersion }}
                  </TagItem>
                </div>
              </div>
              <div class="flex items-center">
                <div class="flex flex-wrap gap-1">
                  <TagItem
                    v-for="platform in version.loaders"
                    :key="`platform-tag-${platform}`"
                    v-tooltip="`Toggle filter for ${platform}`"
                    class="z-[1]"
                    :style="`--_color: var(--color-platform-${platform})`"
                    :action="() => versionFilters?.toggleFilter('platform', platform)"
                  >
                    <!-- eslint-disable-next-line vue/no-v-html -->
                    <svg v-html="loaders.find((x) => x.name === platform)?.icon"></svg>
                    {{ formatCategory(platform) }}
                  </TagItem>
                </div>
              </div>
            </div>
            <div
              class="flex flex-col justify-center gap-1 max-sm:flex-row max-sm:justify-start max-sm:gap-3 xl:contents"
            >
              <div
                v-tooltip="
                  formatMessage(commonMessages.dateAtTimeTooltip, {
                    date: new Date(version.date_published),
                    time: new Date(version.date_published),
                  })
                "
                class="z-[1] flex cursor-help items-center gap-1 text-nowrap font-medium xl:self-center"
              >
                <CalendarIcon class="xl:hidden" />
                {{ formatRelativeTime(version.date_published) }}
              </div>
              <div
                class="pointer-events-none z-[1] flex items-center gap-1 font-medium xl:self-center"
              >
                <DownloadIcon class="xl:hidden" />
                {{ formatNumber(version.downloads) }}
              </div>
            </div>
          </div>
        </div>
        <div class="flex items-start justify-end gap-1 sm:items-center z-[1]">
          <slot name="actions" :version="version"></slot>
        </div>
        <div v-if="showFiles" class="tag-list pointer-events-none relative z-[1] col-span-full">
          <div
            v-for="(file, fileIdx) in version.files"
            :key="`platform-tag-${fileIdx}`"
            :class="`flex items-center gap-1 text-wrap rounded-full bg-button-bg px-2 py-0.5 text-xs font-medium ${file.primary || fileIdx === 0 ? 'bg-brand-highlight text-contrast' : 'text-primary'}`"
          >
            <StarIcon v-if="file.primary || fileIdx === 0" class="shrink-0" />
            {{ file.filename }} - {{ formatBytes(file.size) }}
          </div>
        </div>
      </div>
    </template>
  </div>
  <div class="flex mt-3">
    <Pagination
      :page="currentPage"
      class="ml-auto"
      :count="Math.ceil(filteredVersions.length / pageSize)"
      @switch-page="switchPage"
    />
  </div>
</template>
<script setup lang="ts">
import {
  formatBytes,
  formatCategory,
  formatNumber,
  formatVersionsForDisplay,
  type GameVersionTag,
  type PlatformTag,
  type Version,
} from '@modrinth/utils'

import { commonMessages } from '../../utils/common-messages'
import { CalendarIcon, DownloadIcon, StarIcon } from '@modrinth/assets'
import { Pagination, VersionChannelIndicator, VersionFilterControl } from '../index'
import { useVIntl } from '@vintl/vintl'
import { type Ref, ref, computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import AutoLink from '../base/AutoLink.vue'
import TagItem from '../base/TagItem.vue'
import { useRelativeTime } from '../../composables'

const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime()

type VersionWithDisplayUrlEnding = Version & {
  displayUrlEnding: string
}

const props = withDefaults(
  defineProps<{
    baseId?: string
    project: {
      project_type: string
      slug?: string
      id: string
    }
    versions: VersionWithDisplayUrlEnding[]
    showFiles?: boolean
    currentMember?: boolean
    loaders: PlatformTag[]
    gameVersions: GameVersionTag[]
    versionLink?: (version: Version) => string
  }>(),
  {
    baseId: undefined,
    showFiles: false,
    currentMember: false,
    versionLink: undefined,
  },
)

const currentPage: Ref<number> = ref(1)
const pageSize: Ref<number> = ref(20)
const versionFilters: Ref<InstanceType<typeof VersionFilterControl> | null> = ref(null)

const selectedGameVersions: Ref<string[]> = computed(
  () => versionFilters.value?.selectedGameVersions ?? [],
)
const selectedPlatforms: Ref<string[]> = computed(
  () => versionFilters.value?.selectedPlatforms ?? [],
)
const selectedChannels: Ref<string[]> = computed(() => versionFilters.value?.selectedChannels ?? [])

const filteredVersions = computed(() => {
  return props.versions.filter(
    (version) =>
      hasAnySelected(version.game_versions, selectedGameVersions.value) &&
      hasAnySelected(version.loaders, selectedPlatforms.value) &&
      isAnySelected(version.version_type, selectedChannels.value),
  )
})

function hasAnySelected(values: string[], selected: string[]) {
  return selected.length === 0 || selected.some((value) => values.includes(value))
}

function isAnySelected(value: string, selected: string[]) {
  return selected.length === 0 || selected.includes(value)
}

const currentVersions = computed(() =>
  filteredVersions.value.slice(
    (currentPage.value - 1) * pageSize.value,
    currentPage.value * pageSize.value,
  ),
)

const route = useRoute()
const router = useRouter()

if (route.query.page) {
  currentPage.value = Number(route.query.page) || 1
}

function switchPage(page: number) {
  currentPage.value = page

  router.replace({
    query: {
      ...route.query,
      page: currentPage.value !== 1 ? currentPage.value : undefined,
    },
  })

  window.scrollTo({ top: 0, behavior: 'smooth' })
}

function updateQuery(newQueries: Record<string, string | string[] | undefined | null>) {
  if (newQueries.page) {
    currentPage.value = Number(newQueries.page)
  } else if (newQueries.page === undefined) {
    currentPage.value = 1
  }

  router.replace({
    query: {
      ...route.query,
      ...newQueries,
    },
  })
}
</script>
<style scoped>
.versions-grid-row {
  @apply grid grid-cols-[1fr_min-content] gap-4 supports-[grid-template-columns:subgrid]:col-span-full supports-[grid-template-columns:subgrid]:!grid-cols-subgrid sm:grid-cols-[min-content_1fr_1fr_1fr_min-content] xl:grid-cols-[min-content_1fr_1fr_1fr_1fr_1fr_min-content];
}
</style>
