<script setup>
import { ref, computed } from 'vue'
import { ofetch } from 'ofetch'
import {
  Pagination,
  ProjectCard,
  Checkbox,
  Button,
  ClearIcon,
  SearchIcon,
  DropdownSelect,
  SearchFilter,
  Card,
  ClientIcon,
  ServerIcon,
} from 'omorphia'
import Multiselect from 'vue-multiselect'
import { useSearch } from '@/store/state'
import { get_categories, get_loaders, get_game_versions } from '@/helpers/tags'

// Pull search store
const searchStore = useSearch()

const selectedVersions = ref([])
const showSnapshots = ref(false)

// Sets the clear button's disabled attr
const isClearDisabled = computed({
  get() {
    if (searchStore.facets.length > 0) return false
    if (searchStore.orFacets.length > 0) return false

    if (searchStore.environments.server === true || searchStore.environments.client === true)
      return false
    if (searchStore.openSource === true) return false
    if (selectedVersions.value.length > 0) return false
    return true
  },
})

const categories = await get_categories()
const loaders = await get_loaders()
const availableGameVersions = await get_game_versions()

/**
 * Adds or removes facets from state
 * @param {String} facet The facet to commit to state
 */
const toggleFacet = async (facet) => {
  const index = searchStore.facets.indexOf(facet)

  if (index !== -1) searchStore.facets.splice(index, 1)
  else searchStore.facets.push(facet)

  await getSearchResults()
}

/**
 * Adds or removes orFacets from state
 * @param {String} orFacet The orFacet to commit to state
 */
const toggleOrFacet = async (orFacet) => {
  const index = searchStore.orFacets.indexOf(orFacet)

  if (index !== -1) searchStore.orFacets.splice(index, 1)
  else searchStore.orFacets.push(orFacet)

  await getSearchResults()
}

/**
 * Makes the API request to labrinth
 */
const getSearchResults = async () => {
  const queryString = searchStore.getQueryString()
  const response = await ofetch(`https://api.modrinth.com/v2/search${queryString}`)

  searchStore.setSearchResults(response)
}
await getSearchResults()

/**
 * For when user enters input in search bar
 */
const refreshSearch = async () => {
  await getSearchResults()
}

/**
 * For when the user changes the Sort dropdown
 * @param {Object} e Event param to see selected option
 */
const handleSort = async (e) => {
  searchStore.filter = e.option
  await getSearchResults()
}

/**
 * For when user changes Limit dropdown
 * @param {Object} e Event param to see selected option
 */
const handleLimit = async (e) => {
  searchStore.limit = e.option
  await getSearchResults()
}

/**
 * For when user pages results
 * @param {Number} page The new page to display
 */
const switchPage = async (page) => {
  searchStore.currentPage = parseInt(page)
  if (page === 1) searchStore.offset = 0
  else searchStore.offset = searchStore.currentPage * 10 - 10
  await getSearchResults()
}

/**
 * For when a user interacts with version filters
 */
const handleVersionSelect = async () => {
  searchStore.activeVersions = selectedVersions.value.map((ver) => ver)
  await getSearchResults()
}

/**
 * For when user resets all filters
 */
const handleReset = async () => {
  searchStore.resetFilters()
  selectedVersions.value = []
  isClearDisabled.value = true
  await getSearchResults()
}
</script>

<template>
  <div class="search-container">
    <aside class="filter-panel">
      <Button role="button" :disabled="isClearDisabled" @click="handleReset"
        ><ClearIcon />Clear Filters</Button
      >
      <div class="categories">
        <h2>Categories</h2>
        <div
          v-for="category in categories.filter((cat) => cat.project_type === 'modpack')"
          :key="category.name"
        >
          <SearchFilter
            :active-filters="searchStore.facets"
            :icon="category.icon"
            :display-name="category.name"
            :facet-name="`categories:${encodeURIComponent(category.name)}`"
            class="filter-checkbox"
            @toggle="toggleFacet"
          />
        </div>
      </div>
      <div class="loaders">
        <h2>Loaders</h2>
        <div
          v-for="loader in loaders.filter((l) => l.supported_project_types?.includes('modpack'))"
          :key="loader"
        >
          <SearchFilter
            :active-filters="searchStore.orFacets"
            :icon="loader.icon"
            :display-name="loader.name"
            :facet-name="`categories:${encodeURIComponent(loader.name)}`"
            class="filter-checkbox"
            @toggle="toggleOrFacet"
          />
        </div>
      </div>
      <div class="environment">
        <h2>Environments</h2>
        <SearchFilter
          v-model="searchStore.environments.client"
          display-name="Client"
          :facet-name="client"
          class="filter-checkbox"
          @click="refreshSearch"
        >
          <ClientIcon aria-hidden="true" />
        </SearchFilter>
        <SearchFilter
          v-model="searchStore.environments.server"
          display-name="Server"
          :facet-name="server"
          class="filter-checkbox"
          @click="refreshSearch"
        >
          <ServerIcon aria-hidden="true" />
        </SearchFilter>
      </div>
      <div class="versions">
        <h2>Minecraft versions</h2>
        <Checkbox v-model="showSnapshots" class="filter-checkbox">Show snapshots</Checkbox>
        <multiselect
          v-model="selectedVersions"
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
          :selectable="() => selectedVersions.length <= 6"
          placeholder="Choose versions..."
          @update:model-value="handleVersionSelect"
        />
      </div>
      <div class="open-source">
        <h2>Open source</h2>
        <Checkbox v-model="searchStore.openSource" class="filter-checkbox" @click="refreshSearch">
          Open source
        </Checkbox>
      </div>
    </aside>
    <div class="search">
      <Card class="search-panel-container">
        <div class="search-panel">
          <div class="iconified-input">
            <SearchIcon aria-hidden="true" />
            <input
              v-model="searchStore.searchInput"
              type="text"
              placeholder="Search.."
              @input="refreshSearch"
            />
          </div>
          <span>Sort by</span>
          <DropdownSelect
            name="Sort dropdown"
            :options="[
              'Relevance',
              'Download count',
              'Follow count',
              'Recently published',
              'Recently updated',
            ]"
            :default-value="searchStore.filter"
            :model-value="searchStore.filter"
            class="sort-dropdown"
            @change="handleSort"
          />
          <span>Show per page</span>
          <DropdownSelect
            name="Limit dropdown"
            :options="['5', '10', '15', '20', '50', '100']"
            :default-value="searchStore.limit.toString()"
            :model-value="searchStore.limit.toString()"
            class="limit-dropdown"
            @change="handleLimit"
          />
        </div>
      </Card>
      <Pagination
        :page="searchStore.currentPage"
        :count="searchStore.pageCount"
        @switch-page="switchPage"
      />
      <section class="project-list display-mode--list instance-results" role="list">
        <ProjectCard
          v-for="result in searchStore.searchResults"
          :id="result?.project_id"
          :key="result?.project_id"
          class="result-project-item"
          :tooltip="result?.slug"
          :type="result?.project_type"
          :name="result?.title"
          :description="result?.description"
          :icon-url="result?.icon_url"
          :downloads="result?.downloads?.toString()"
          :follows="result?.follows?.toString()"
          :created-at="result?.date_created"
          :updated-at="result?.date_modified"
          :categories="[
            ...categories.filter(
              (cat) =>
                result?.display_categories.includes(cat.name) && cat.project_type === 'modpack'
            ),
            ...loaders.filter(
              (loader) =>
                result?.display_categories.includes(loader.name) &&
                loader.supported_project_types?.includes('modpack')
            ),
          ]"
          :project-type-display="result?.project_type"
          project-type-url="instance"
          :server-side="result?.server_side"
          :client-side="result?.client_side"
          :show-updated-date="false"
          :color="result?.color"
        >
        </ProjectCard>
      </section>
    </div>
  </div>
</template>

<style src="vue-multiselect/dist/vue-multiselect.css"></style>
<style lang="scss">
.search-panel-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  width: 100%;
  margin-top: 1rem;
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
    width: 16rem;
    background: var(--color-raised-bg);
    padding: 1rem 1rem 3rem 1rem;
    display: flex;
    flex-direction: column;
    min-height: 100vh;
    height: fit-content;
    max-height: 100%;
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
      text-transform: capitalize;

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
    margin: 0 1rem 0 17rem;
    width: 100%;

    .instance-project-item {
      width: 100%;
      height: auto;
      cursor: pointer;
    }

    .result-project-item {
      a {
        &:hover {
          text-decoration: none !important;
        }
      }
    }
  }
}

.multiselect {
  color: var(--color-base) !important;
  outline: 2px solid transparent;

  .multiselect__input:focus-visible {
    outline: none !important;
    box-shadow: none !important;
    padding: 0 !important;
    min-height: 0 !important;
    font-weight: normal !important;
    margin-left: 0.5rem;
    margin-bottom: 10px;
  }

  input {
    background: transparent;
    box-shadow: none;
    border: none !important;

    &:focus {
      box-shadow: none;
    }
  }

  input::placeholder {
    color: var(--color-base);
  }

  .multiselect__tags {
    border-radius: var(--radius-md);
    background: var(--color-button-bg);
    box-shadow: var(--shadow-inset-sm);
    border: none;
    cursor: pointer;
    padding-left: 0.5rem;
    font-size: 1rem;

    transition: background-color 0.1s ease-in-out;

    &:active {
      filter: brightness(1.25);

      .multiselect__spinner {
        filter: brightness(1.25);
      }
    }

    .multiselect__single {
      background: transparent;
    }

    .multiselect__tag {
      border-radius: var(--radius-md);
      color: var(--color-base);
      background: transparent;
      border: 2px solid var(--color-brand);
    }

    .multiselect__tag-icon {
      background: transparent;

      &:after {
        color: var(--color-contrast);
      }
    }

    .multiselect__placeholder {
      color: var(--color-base);
      margin-left: 0.5rem;
      opacity: 0.6;
      font-size: 1rem;
      line-height: 1.25rem;
    }
  }

  .multiselect__content-wrapper {
    background: var(--color-button-bg);
    border: none;
    overflow-x: hidden;
    box-shadow: var(--shadow-inset-sm), var(--shadow-floating);
    width: 100%;

    .multiselect__element {
      .multiselect__option--highlight {
        background: var(--color-button-bg);
        filter: brightness(1.25);
        color: var(--color-contrast);
      }

      .multiselect__option--selected {
        background: var(--color-brand);
        font-weight: bold;
        color: var(--color-accent-contrast);
      }
    }
  }

  .multiselect__spinner {
    background: var(--color-button-bg);

    &:active {
      filter: brightness(1.25);
    }
  }

  &.multiselect--disabled {
    background: none;

    .multiselect__current,
    .multiselect__select {
      background: none;
    }
  }
}

.multiselect--above .multiselect__content-wrapper {
  border-top: none !important;
  border-top-left-radius: var(--radius-md) !important;
  border-top-right-radius: var(--radius-md) !important;
}
</style>
