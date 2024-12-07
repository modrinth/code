<script setup lang="ts">
import { computed, nextTick, ref, shallowRef, watch } from 'vue'
import type { Ref } from 'vue'
import { GameIcon, LeftArrowIcon, SearchIcon, XIcon } from '@modrinth/assets'
import type { Category, GameVersion, Platform, ProjectType, Tags } from '@modrinth/ui'
import {
  SearchSidebarFilter,
  Avatar,
  Button,
  ButtonStyled,
  Checkbox,
  DropdownSelect,
  LoadingIndicator,
  Pagination,
  useSearch,
} from '@modrinth/ui'
import { formatCategory } from '@modrinth/utils'
import { handleError } from '@/store/state'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import { get_categories, get_game_versions, get_loaders } from '@/helpers/tags'
import { useRoute, useRouter } from 'vue-router'
import SearchCard from '@/components/ui/SearchCard.vue'
import { get as getInstance, get_projects as getInstanceProjects } from '@/helpers/profile.js'
import { convertFileSrc } from '@tauri-apps/api/core'
import { get_search_results } from '@/helpers/cache.js'
import { debounce } from '@/helpers/utils.js'
import NavTabs from '@/components/ui/NavTabs.vue'

const router = useRouter()
const route = useRoute()

const projectTypes = computed(() => {
  return [route.params.projectType as ProjectType]
})

const [categories, loaders, availableGameVersions] = await Promise.all([
  get_categories().catch(handleError).then(ref),
  get_loaders().catch(handleError).then(ref),
  get_game_versions().catch(handleError).then(ref),
])

const tags: Ref<Tags> = computed(() => ({
  gameVersions: availableGameVersions.value as GameVersion[],
  loaders: loaders.value as Platform[],
  categories: categories.value as Category[],
}))

const instance = ref(null)
const instanceProjects = ref(null)
const instanceHideInstalled = ref(false)
const newlyInstalled = ref([])

const PERSISTENT_QUERY_PARAMS = ['i', 'ai']

if (route.query.i) {
  ;[instance.value, instanceProjects.value] = await Promise.all([
    getInstance(route.query.i).catch(handleError),
    getInstanceProjects(route.query.i).catch(handleError),
  ])
  newlyInstalled.value = []
}

if (route.query.ai && !(projectTypes.value.length === 1 && projectTypes.value[0] === 'modpack')) {
  instanceHideInstalled.value = route.query.ai === 'true'
}

const instanceFilters = computed(() => {
  const filters = []

  if (instance.value) {
    const gameVersion = instance.value.game_version
    if (gameVersion) {
      filters.push({
        type: 'game_version',
        option: gameVersion,
      })
    }

    const platform = instance.value.loader

    const supportedModLoaders = ['fabric', 'forge', 'quilt', 'neoforge']

    if (platform && projectTypes.value.includes('mod') && supportedModLoaders.includes(platform)) {
      filters.push({
        type: 'mod_loader',
        option: platform,
      })
    }

    if (instanceHideInstalled.value) {
      const installedMods = Object.values(instanceProjects.value)
        .filter((x) => x.metadata)
        .map((x) => x.metadata.project_id)

      installedMods.push(...newlyInstalled.value)

      installedMods
        ?.map((x) => ({
          type: 'project_id',
          option: `project_id:${x}`,
          negative: true,
        }))
        .forEach((x) => filters.push(x))
    }
  }

  return filters
})

const {
  // Selections
  query,
  currentSortType,
  currentFilters,
  toggledGroups,
  maxResults,
  currentPage,
  overriddenProvidedFilterTypes,

  // Lists
  filters,
  sortTypes,

  // Computed
  requestParams,

  // Functions
  createPageParams,
} = useSearch(projectTypes, tags, instanceFilters)

const offline = ref(!navigator.onLine)
window.addEventListener('offline', () => {
  offline.value = true
})
window.addEventListener('online', () => {
  offline.value = false
})

const breadcrumbs = useBreadcrumbs()
breadcrumbs.setContext({ name: 'Discover content', link: route.path, query: route.query })

const loading = ref(false)

const projectType = ref(route.params.projectType)

const results = shallowRef([])
const pageCount = computed(() =>
  results.value ? Math.ceil(results.value.total_hits / results.value.limit) : 1,
)

watch(requestParams, () => {
  refreshSearch()
})

async function refreshSearch() {
  let rawResults = await get_search_results(requestParams.value)
  if (!rawResults) {
    rawResults = {
      result: {
        hits: [],
        total_hits: 0,
        limit: 1,
      },
    }
  }
  if (instance.value) {
    for (const val of rawResults.result.hits) {
      val.installed =
        newlyInstalled.value.includes(val.project_id) ||
        Object.values(instanceProjects.value).some(
          (x) => x.metadata && x.metadata.project_id === val.project_id,
        )
    }
  }
  results.value = rawResults.result
}

async function updateSearchResults() {
  if (query.value === null) {
    return
  }

  await refreshSearch()

  if (import.meta.client) {
    const persistentParams = {}

    for (const [key, value] of Object.entries(route.query)) {
      if (PERSISTENT_QUERY_PARAMS.includes(key)) {
        persistentParams[key] = value
      }
    }

    if (serverHideInstalled.value) {
      persistentParams.ai = 'true'
    } else {
      delete persistentParams.ai
    }

    const params = {
      ...persistentParams,
      ...createPageParams(),
    }

    router.replace({ path: route.path, query: params })
    breadcrumbs.setContext({ name: 'Discover content', link: route.path, query: params })
  }
}

const debouncedSearchChange = debounce(() => updateSearchResults(1), 200)

const searchWrapper = ref(null)
async function onSearchChangeToTop(newPageNumber) {
  await updateSearchResults(newPageNumber)
  await nextTick()
  searchWrapper.value.scrollTo({ top: 0, behavior: 'smooth' })
}

async function clearSearch() {
  query.value = ''
  await updateSearchResults(1)
}

function getSearchUrl(offset, useObj) {
  const queryItems = []
  const obj = {}

  if (offset > 0) {
    queryItems.push(`o=${offset}`)
  }

  let url = `${route.path}`

  if (queryItems.length > 0) {
    url += `?${queryItems[0]}`

    for (let i = 1; i < queryItems.length; i++) {
      url += `&${queryItems[i]}`
    }
  }

  return useObj ? obj : url
}

async function clearFilters() {}

watch(
  () => route.params.projectType,
  async (newType) => {
    // Check if the newType is not the same as the current value
    if (!newType || newType === projectType.value) return

    projectType.value = newType
    breadcrumbs.setContext({ name: 'Discover content', link: `/browse/${projectType.value}` })

    currentSortType.value = { display: 'Relevance', name: 'relevance' }
    query.value = ''

    loading.value = true
    await clearFilters()
    loading.value = false
  },
)

const selectableProjectTypes = computed(() => {
  let dataPacks = false,
    mods = false,
    modpacks = false

  if (instance.value) {
    if (
      availableGameVersions.value.findIndex((x) => x.version === instance.value.game_version) <=
      availableGameVersions.value.findIndex((x) => x.version === '1.13')
    ) {
      dataPacks = true
    }

    if (instance.value.loader !== 'vanilla') {
      mods = true
    }
  } else {
    dataPacks = true
    mods = true
    modpacks = true
  }

  return [
    { label: 'Modpacks', href: `/browse/modpack`, shown: modpacks },
    { label: 'Mods', href: `/browse/mod`, shown: mods },
    { label: 'Resource Packs', href: `/browse/resourcepack` },
    { label: 'Data Packs', href: `/browse/datapack`, shown: dataPacks },
    { label: 'Shaders', href: `/browse/shader` },
  ]
})

await refreshSearch()
</script>

<template>
  <Teleport v-if="filters" to="#sidebar-teleport-target">
    <div
      v-if="instance"
      class="border-0 border-b-[1px] p-4 last:border-b-0 border-[--brand-gradient-border] border-solid"
    >
      <Checkbox
        v-model="instanceHideInstalled"
        label="Hide installed content"
        class="filter-checkbox"
        @update:model-value="onSearchChangeToTop(1)"
        @click.prevent.stop
      />
    </div>
    <SearchSidebarFilter
      v-for="filter in filters.filter((f) => f.display !== 'none')"
      :key="`filter-${filter.id}`"
      v-model:selected-filters="currentFilters"
      v-model:toggled-groups="toggledGroups"
      v-model:overridden-provided-filter-types="overriddenProvidedFilterTypes"
      :provided-filters="instanceFilters"
      :filter-type="filter"
      class="border-0 border-b-[1px] [&:first-child>button]:pt-4 last:border-b-0 border-[--brand-gradient-border] border-solid"
      button-class="button-animation flex flex-col gap-1 px-4 py-3 w-full bg-transparent cursor-pointer border-none hover:bg-button-bg"
      content-class="mb-3"
      inner-panel-class="ml-2 mr-3"
      :open-by-default="filter.id.startsWith('category') || filter.id === 'environment'"
    >
      <template #header>
        <h3 class="text-lg m-0">{{ filter.formatted_name }}</h3>
      </template>
    </SearchSidebarFilter>
  </Teleport>
  <div ref="searchWrapper" class="flex flex-col gap-3 p-6">
    <template v-if="instance">
      <div
        class="flex justify-between items-center border-0 border-b border-solid border-button-bg pb-4"
      >
        <router-link
          :to="`/instance/${encodeURIComponent(instance.path)}`"
          tabindex="-1"
          class="flex flex-col gap-4 text-primary"
        >
          <span class="flex items-center gap-2">
            <Avatar
              :src="instance.icon_path ? convertFileSrc(instance.icon_path) : null"
              :alt="instance.name"
              size="48px"
            />
            <span class="flex flex-col gap-2">
              <span class="font-extrabold bold text-contrast">
                {{ instance.name }}
              </span>
              <span class="text-secondary flex items-center gap-2 font-semibold">
                <GameIcon class="h-5 w-5 text-secondary" />
                {{ formatCategory(instance.loader) }} {{ instance.game_version }}
              </span>
            </span>
          </span>
        </router-link>
        <ButtonStyled>
          <router-link :to="`/instance/${encodeURIComponent(instance.path)}`">
            <LeftArrowIcon /> Back to instance
          </router-link>
        </ButtonStyled>
      </div>
      <h1 class="m-0 mb-1 text-xl">Install content to instance</h1>
    </template>
    <h1 v-else class="m-0 mb-1 text-2xl">Discover content</h1>
    <NavTabs :links="selectableProjectTypes" />
    <div class="iconified-input">
      <SearchIcon aria-hidden="true" class="text-lg" />
      <input
        v-model="query"
        class="h-12"
        autocomplete="off"
        spellcheck="false"
        type="text"
        :placeholder="`Search ${projectType}s...`"
        @input="debouncedSearchChange()"
      />
      <Button v-if="query" class="r-btn" @click="() => clearSearch()">
        <XIcon />
      </Button>
    </div>
    <div class="flex gap-2">
      <DropdownSelect
        v-slot="{ selected }"
        v-model="currentSortType"
        class="max-w-[16rem]"
        name="Sort by"
        :options="sortTypes"
        :display-name="(option) => option?.display"
        @change="updateSearchResults(1)"
      >
        <span class="font-semibold text-primary">Sort by: </span>
        <span class="font-semibold text-secondary">{{ selected }}</span>
      </DropdownSelect>
      <DropdownSelect
        v-slot="{ selected }"
        v-model="maxResults"
        name="Max results"
        :options="[5, 10, 15, 20, 50, 100]"
        :default-value="maxResults"
        :model-value="maxResults"
        class="max-w-[9rem]"
        @change="updateSearchResults(1)"
      >
        <span class="font-semibold text-primary">View: </span>
        <span class="font-semibold text-secondary">{{ selected }}</span>
      </DropdownSelect>
      <Pagination
        :page="currentPage"
        :count="pageCount"
        :link-function="(x) => getSearchUrl(x <= 1 ? 0 : (x - 1) * maxResults)"
        class="ml-auto"
        @switch-page="updateSearchResults"
      />
    </div>
    <div class="search">
      <section v-if="loading" class="offline">
        <LoadingIndicator />
      </section>
      <section v-else-if="offline && results.total_hits === 0" class="offline">
        You are currently offline. Connect to the internet to browse Modrinth!
      </section>
      <section v-else class="project-list display-mode--list instance-results" role="list">
        <SearchCard
          v-for="result in results.hits"
          :key="result?.project_id"
          :project="result"
          :instance="instance"
          :categories="[
            ...categories.filter(
              (cat) =>
                result?.display_categories.includes(cat.name) && cat.project_type === projectType,
            ),
            ...loaders.filter(
              (loader) =>
                result?.display_categories.includes(loader.name) &&
                loader.supported_project_types?.includes(projectType),
            ),
          ]"
          :installed="result.installed"
          @install="
            (id) => {
              newlyInstalled.push(id)
              refreshSearch()
            }
          "
        />
      </section>
      <div class="flex justify-end">
        <pagination
          :page="currentPage"
          :count="pageCount"
          :link-function="(x) => getSearchUrl(x <= 1 ? 0 : (x - 1) * maxResults)"
          class="pagination-after"
          @switch-page="onSearchChangeToTop"
        />
      </div>
    </div>
  </div>
</template>
