<script setup>
import { computed, onMounted, ref, watch } from 'vue'
import { ofetch } from 'ofetch'
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
  formatCategoryHeader, Avatar, formatCategory,
} from 'omorphia'
import Multiselect from 'vue-multiselect'
import { useSearch } from '@/store/state'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import { get_categories, get_loaders, get_game_versions } from '@/helpers/tags'
import { useRoute } from 'vue-router'
import SearchCard from '@/components/ui/SearchCard.vue'
import InstallConfirmModal from '@/components/ui/InstallConfirmModal.vue'
import InstanceInstallModal from '@/components/ui/InstanceInstallModal.vue'
import SplashScreen from '@/components/ui/SplashScreen.vue'
import {convertFileSrc} from "@tauri-apps/api/tauri";

const route = useRoute()

const searchStore = useSearch()
searchStore.projectType = route.params.projectType
const showVersions = ref(true)
const showLoaders = ref(true)
const confirmModal = ref(null)
const modInstallModal = ref(null)

const breadcrumbs = useBreadcrumbs()

const showSnapshots = ref(false)
const loading = ref(true)

const categories = ref([])
const loaders = ref([])
const availableGameVersions = ref([])

onMounted(async () => {
  [categories.value, loaders.value, availableGameVersions.value] = await Promise.all([
    get_categories(),
    get_loaders(),
    get_game_versions(),
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
  if (searchStore.instanceContext) {
    showVersions.value = false
    showLoaders.value = !(
      searchStore.projectType === 'mod' || searchStore.projectType === 'resourcepack'
    )
  }
  const response = await ofetch(`https://api.modrinth.com/v2/search${queryString}`)
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
    searchStore.projectType = projectType ?? 'modpack'
    breadcrumbs.setContext({ name: 'Browse', link: route.path })
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
      <Card v-if="searchStore.instanceContext" class="instance-small-card button-base" @click="seeInstance">
        <div class="instance-small-card__description">
          <Avatar
            :src="convertFileSrc(searchStore.instanceContext.metadata.icon)"
            :alt="searchStore.instanceContext.metadata.name"
            size="sm"
          />
          <div class="instance-small-card__info">
            <span class="title">{{ searchStore.instanceContext.metadata.name }}</span>
            {{
              searchStore.instanceContext.metadata.loader.charAt(0).toUpperCase() +
              searchStore.instanceContext.metadata.loader.slice(1)
            }}
            {{ searchStore.instanceContext.metadata.game_version }}
          </div>
        </div>
        <Checkbox
          :model-value="searchStore.ignoreInstance"
          :checked="searchStore.ignoreInstance"
          label="Ignore Instance"
          class="filter-checkbox"
          @update:model-value="(value) => handleInstanceSwitch(value)"
        />
      </Card>
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
        <div
          v-if="
            showLoaders &&
            searchStore.projectType !== 'datapack' &&
            searchStore.projectType !== 'resourcepack' &&
            searchStore.projectType !== 'shader'
          "
          class="loaders"
        >
          <h2>Loaders</h2>
          <div
            v-for="loader in loaders.filter((l) =>
              l.supported_project_types?.includes(searchStore.projectType)
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
        <div class="search-panel">
          <div class="iconified-input">
            <SearchIcon aria-hidden="true" />
            <input
              v-model="searchStore.searchInput"
              type="text"
              :placeholder="`Search ${searchStore.projectType}s...`"
              @input="getSearchResults"
            />
          </div>
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
</template>

<style src="vue-multiselect/dist/vue-multiselect.css"></style>
<style lang="scss">
.instance-small-card {
  background-color: var(--color-bg) !important;
  display: flex;
  flex-direction: column;
  min-height: min-content !important;
  gap: 0.5rem;
  align-items: flex-start;

  .instance-small-card__description {
    display: flex;
    flex-direction: row;
    justify-content: center;
    gap: 1rem;
    flex-grow: 1;
  }

  .instance-small-card__info {
    display: flex;
    flex-direction: column;
    justify-content: center;

    .title {
      color: var(--color-contrast);
      font-weight: bolder;
    }
  }

  .cta {
    display: none;
  }
}


.project-type-dropdown {
  width: 100% !important;
}

.project-type-container {
  display: flex;
  flex-direction: column;
  width: 100%;
  margin-top: 1rem;
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
    max-width: 20rem !important;
  }
}

.search-panel-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  width: 100%;
  padding: 0.8rem !important;

  .search-panel {
    display: flex;
    align-items: center;
    justify-content: space-evenly;
    width: 100%;
    gap: 1rem;
    margin: 1rem auto;
    white-space: nowrap;

    .sort-dropdown {
      min-width: 12.18rem;
    }

    .limit-dropdown {
      width: 5rem;
    }

    .iconified-input {
      width: 75%;

      input {
        flex-basis: initial;
      }
    }
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
    width: 19rem;
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
  }

  .search {
    margin: 0 1rem 0 20rem;
    width: calc(100% - 21rem);

    .loading {
      margin: 2rem;
      text-align: center;
    }
  }
}
</style>
