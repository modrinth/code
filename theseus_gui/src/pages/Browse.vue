<script setup>
import { computed, onMounted, ref, watch } from 'vue'
import {
  Pagination,
  Checkbox,
  Button,
  ClearIcon,
  SearchIcon,
  DropdownSelect,
  SearchFilter,
  Card,
  ClientIcon,
  ServerIcon,
  NavRow,
  formatCategoryHeader,
  formatCategory,
  Promotion,
} from 'omorphia'
import Multiselect from 'vue-multiselect'
import { handleError, useSearch } from '@/store/state'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import { get_categories, get_loaders, get_game_versions } from '@/helpers/tags'
import { useRoute } from 'vue-router'
import SearchCard from '@/components/ui/SearchCard.vue'
import InstallConfirmModal from '@/components/ui/InstallConfirmModal.vue'
import InstanceInstallModal from '@/components/ui/InstanceInstallModal.vue'
import SplashScreen from '@/components/ui/SplashScreen.vue'
import Instance from '@/components/ui/Instance.vue'
import IncompatibilityWarningModal from '@/components/ui/IncompatibilityWarningModal.vue'
import { useFetch } from '@/helpers/fetch.js'

const route = useRoute()

const searchStore = useSearch()
searchStore.projectType = route.params.projectType
const showVersions = computed(
  () => searchStore.instanceContext === null || searchStore.ignoreInstance
)
const showLoaders = computed(
  () =>
    searchStore.projectType !== 'datapack' &&
    searchStore.projectType !== 'resourcepack' &&
    searchStore.projectType !== 'shader' &&
    (searchStore.instanceContext === null || searchStore.ignoreInstance)
)
const confirmModal = ref(null)
const modInstallModal = ref(null)
const incompatibilityWarningModal = ref(null)

const breadcrumbs = useBreadcrumbs()

const showSnapshots = ref(false)
const loading = ref(true)

const categories = ref([])
const loaders = ref([])
const availableGameVersions = ref([])

breadcrumbs.setContext({ name: 'Browse', link: route.path })

if (searchStore.projectType === 'modpack') {
  searchStore.instanceContext = null
}

onMounted(async () => {
  ;[categories.value, loaders.value, availableGameVersions.value] = await Promise.all([
    get_categories().catch(handleError),
    get_loaders().catch(handleError),
    get_game_versions().catch(handleError),
  ])
  breadcrumbs.setContext({ name: 'Browse', link: route.path })
  if (searchStore.projectType === 'modpack') {
    searchStore.instanceContext = null
  }
  searchStore.searchInput = ''
  await handleReset()
  loading.value = false
})

const sortedCategories = computed(() => {
  const values = new Map()
  for (const category of categories.value.filter(
    (cat) =>
      cat.project_type ===
      (searchStore.projectType === 'datapack' ? 'mod' : searchStore.projectType)
  )) {
    if (!values.has(category.header)) {
      values.set(category.header, [])
    }
    values.get(category.header).push(category)
  }
  return values
})

const getSearchResults = async () => {
  const queryString = searchStore.getQueryString()
  const response = await useFetch(
    `https://api.modrinth.com/v2/search${queryString}`,
    'search results'
  )
  searchStore.setSearchResults(response)
}

const handleReset = async () => {
  searchStore.currentPage = 1
  searchStore.offset = 0
  searchStore.resetFilters()
  await getSearchResults()
}

const toggleFacet = async (facet) => {
  searchStore.currentPage = 1
  searchStore.offset = 0
  const index = searchStore.facets.indexOf(facet)

  if (index !== -1) searchStore.facets.splice(index, 1)
  else searchStore.facets.push(facet)

  await switchPage(1)
}

const toggleOrFacet = async (orFacet) => {
  const index = searchStore.orFacets.indexOf(orFacet)

  if (index !== -1) searchStore.orFacets.splice(index, 1)
  else searchStore.orFacets.push(orFacet)

  await switchPage(1)
}

const switchPage = async (page) => {
  searchStore.currentPage = parseInt(page)
  if (page === 1) searchStore.offset = 0
  else searchStore.offset = (searchStore.currentPage - 1) * searchStore.limit
  await getSearchResults()
}

watch(
  () => route.params.projectType,
  async (projectType) => {
    if (!projectType) return
    searchStore.projectType = projectType
    breadcrumbs.setContext({ name: 'Browse', link: `/browse/${searchStore.projectType}` })
    await handleReset()
    await switchPage(1)
  }
)

const handleInstanceSwitch = async (value) => {
  searchStore.ignoreInstance = value
  await switchPage(1)
}
</script>

<template>
  <div class="search-container">
    <aside class="filter-panel">
      <Instance v-if="searchStore.instanceContext" :instance="searchStore.instanceContext" small>
        <template #content>
          <Checkbox
            :model-value="searchStore.ignoreInstance"
            :checked="searchStore.ignoreInstance"
            label="Unfilter loader & version"
            class="filter-checkbox"
            @update:model-value="(value) => handleInstanceSwitch(value)"
          />
        </template>
      </Instance>
      <Card class="search-panel-card">
        <Button
          role="button"
          :disabled="
            !(
              searchStore.facets.length > 0 ||
              searchStore.orFacets.length > 0 ||
              searchStore.environments.server === true ||
              searchStore.environments.client === true ||
              searchStore.openSource === true ||
              searchStore.activeVersions.length > 0
            )
          "
          @click="handleReset"
          ><ClearIcon />Clear Filters</Button
        >
        <div v-if="showLoaders" class="loaders">
          <h2>Loaders</h2>
          <div
            v-for="loader in loaders.filter(
              (l) =>
                (searchStore.projectType !== 'mod' &&
                  l.supported_project_types?.includes(searchStore.projectType)) ||
                (searchStore.projectType === 'mod' && ['fabric', 'forge', 'quilt'].includes(l.name))
            )"
            :key="loader"
          >
            <SearchFilter
              :active-filters="searchStore.orFacets"
              :icon="loader.icon"
              :display-name="formatCategory(loader.name)"
              :facet-name="`categories:${encodeURIComponent(loader.name)}`"
              class="filter-checkbox"
              @toggle="toggleOrFacet"
            />
          </div>
        </div>
        <div
          v-for="categoryList in Array.from(sortedCategories)"
          :key="categoryList[0]"
          class="categories"
        >
          <h2>{{ formatCategoryHeader(categoryList[0]) }}</h2>
          <div v-for="category in categoryList[1]" :key="category.name">
            <SearchFilter
              :active-filters="searchStore.facets"
              :icon="category.icon"
              :display-name="formatCategory(category.name)"
              :facet-name="`categories:${encodeURIComponent(category.name)}`"
              class="filter-checkbox"
              @toggle="toggleFacet"
            />
          </div>
        </div>
        <div v-if="searchStore.projectType !== 'datapack'" class="environment">
          <h2>Environments</h2>
          <SearchFilter
            v-model="searchStore.environments.client"
            display-name="Client"
            facet-name="client"
            class="filter-checkbox"
            @click="getSearchResults"
          >
            <ClientIcon aria-hidden="true" />
          </SearchFilter>
          <SearchFilter
            v-model="searchStore.environments.server"
            display-name="Server"
            facet-name="server"
            class="filter-checkbox"
            @click="getSearchResults"
          >
            <ServerIcon aria-hidden="true" />
          </SearchFilter>
        </div>
        <div v-if="showVersions" class="versions">
          <h2>Minecraft versions</h2>
          <Checkbox v-model="showSnapshots" class="filter-checkbox" label="Include snapshots" />
          <multiselect
            v-model="searchStore.activeVersions"
            :options="
              showSnapshots
                ? availableGameVersions.map((x) => x.version)
                : availableGameVersions
                    .filter((it) => it.version_type === 'release')
                    .map((x) => x.version)
            "
            :multiple="true"
            :searchable="true"
            :show-no-results="false"
            :close-on-select="false"
            :clear-search-on-select="false"
            :show-labels="false"
            placeholder="Choose versions..."
            @update:model-value="getSearchResults"
          />
        </div>
        <div class="open-source">
          <h2>Open source</h2>
          <Checkbox
            v-model="searchStore.openSource"
            class="filter-checkbox"
            label="Open source"
            @click="getSearchResults"
          />
        </div>
      </Card>
    </aside>
    <div class="search">
      <Promotion class="promotion" />
      <Card class="project-type-container">
        <NavRow
          :links="
            searchStore.instanceContext
              ? [
                  { label: 'Mods', href: `/browse/mod` },
                  { label: 'Datapacks', href: `/browse/datapack` },
                  { label: 'Shaders', href: `/browse/shader` },
                  { label: 'Resource Packs', href: `/browse/resourcepack` },
                ]
              : [
                  { label: 'Modpacks', href: '/browse/modpack' },
                  { label: 'Mods', href: '/browse/mod' },
                  { label: 'Datapacks', href: '/browse/datapack' },
                  { label: 'Shaders', href: '/browse/shader' },
                  { label: 'Resource Packs', href: '/browse/resourcepack' },
                ]
          "
        />
      </Card>
      <Card class="search-panel-container">
        <div class="iconified-input">
          <SearchIcon aria-hidden="true" />
          <input
            v-model="searchStore.searchInput"
            type="text"
            :placeholder="`Search ${searchStore.projectType}s...`"
            @input="getSearchResults"
          />
        </div>
        <div class="inline-option">
          <span>Sort by</span>
          <DropdownSelect
            v-model="searchStore.filter"
            name="Sort dropdown"
            :options="[
              'Relevance',
              'Download count',
              'Follow count',
              'Recently published',
              'Recently updated',
            ]"
            class="sort-dropdown"
            @change="getSearchResults"
          />
        </div>
        <div class="inline-option">
          <span>Show per page</span>
          <DropdownSelect
            v-model="searchStore.limit"
            name="Limit dropdown"
            :options="[5, 10, 15, 20, 50, 100]"
            :default-value="searchStore.limit.toString()"
            :model-value="searchStore.limit.toString()"
            class="limit-dropdown"
            @change="getSearchResults"
          />
        </div>
      </Card>
      <Pagination
        :page="searchStore.currentPage"
        :count="searchStore.pageCount"
        @switch-page="switchPage"
      />
      <SplashScreen v-if="loading" />
      <section v-else class="project-list display-mode--list instance-results" role="list">
        <SearchCard
          v-for="result in searchStore.searchResults"
          :key="result?.project_id"
          :project="result"
          :instance="searchStore.instanceContext"
          :categories="[
            ...categories.filter(
              (cat) =>
                result?.display_categories.includes(cat.name) &&
                cat.project_type === searchStore.projectType
            ),
            ...loaders.filter(
              (loader) =>
                result?.display_categories.includes(loader.name) &&
                loader.supported_project_types?.includes(searchStore.projectType)
            ),
          ]"
          :confirm-modal="confirmModal"
          :mod-install-modal="modInstallModal"
          :incompatibility-warning-modal="incompatibilityWarningModal"
        />
      </section>
      <Pagination
        :page="searchStore.currentPage"
        :count="searchStore.pageCount"
        @switch-page="switchPage"
      />
    </div>
  </div>
  <InstallConfirmModal ref="confirmModal" />
  <InstanceInstallModal ref="modInstallModal" />
  <IncompatibilityWarningModal ref="incompatibilityWarningModal" />
</template>

<style src="vue-multiselect/dist/vue-multiselect.css"></style>
<style lang="scss">
.filter-checkbox {
  margin-bottom: 0.3rem;
  font-size: 1rem;

  svg {
    display: flex;
    align-self: center;
    justify-self: center;
  }

  button.checkbox {
    border: none;
  }
}
</style>
<style lang="scss" scoped>
.project-type-dropdown {
  width: 100% !important;
}

.promotion {
  margin-top: 1rem;
}

.project-type-container {
  display: flex;
  flex-direction: column;
  width: 100%;
}

.search-panel-card {
  display: flex;
  flex-direction: column;
  background-color: var(--color-bg) !important;
  margin-bottom: 0;
  min-height: min-content !important;
}

.iconified-input {
  input {
    max-width: none !important;
    flex-basis: auto;
  }
}

.search-panel-container {
  display: inline-flex;
  flex-direction: row;
  align-items: center;
  flex-wrap: wrap;
  width: 100%;
  padding: 1rem !important;
  white-space: nowrap;
  gap: 1rem;

  .inline-option {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 0.5rem;

    .sort-dropdown {
      max-width: 12.25rem;
    }

    .limit-dropdown {
      width: 5rem;
    }
  }

  .iconified-input {
    flex-grow: 1;
  }

  .filter-panel {
    button {
      display: flex;
      align-items: center;
      justify-content: space-evenly;

      svg {
        margin-right: 0.4rem;
      }
    }
  }
}

.search-container {
  display: flex;

  .filter-panel {
    position: fixed;
    width: 20rem;
    background: var(--color-raised-bg);
    padding: 1rem;
    display: flex;
    flex-direction: column;
    height: fit-content;
    min-height: calc(100vh - 3.25rem);
    max-height: calc(100vh - 3.25rem);
    overflow-y: auto;

    h2 {
      color: var(--color-contrast);
      margin-top: 1rem;
      margin-bottom: 0.5rem;
      font-size: 1.16rem;
    }
  }

  .search {
    margin: 0 1rem 0 21rem;
    width: calc(100% - 22rem);

    .loading {
      margin: 2rem;
      text-align: center;
    }
  }
}
</style>
