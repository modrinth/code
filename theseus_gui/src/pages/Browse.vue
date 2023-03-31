<script setup>
import { ref } from 'vue'
import { storeToRefs } from 'pinia'
import {
  Pagination,
  ProjectCard,
  Checkbox,
  Button,
  ClearIcon,
  SearchIcon,
  DropdownSelect,
} from 'omorphia'
import Multiselect from 'vue-multiselect'
import { useInstances } from '@/store/state'
import generated from '@/generated'

const instanceStore = useInstances()
await instanceStore.searchInstances()
const { getCategoriesByInstanceId } = storeToRefs(instanceStore)
const { getIconByFilter } = storeToRefs(instanceStore)

const currentPage = ref(1)
const selectedVersions = ref([])
const showSnapshots = ref(false)
const searchText = ref('')
const sort = ref('Relevance')
const limit = ref(20)

const availableGameVersions = generated.gameVersions

const searchHandler = async () => {
  instanceStore.setSearchInput(searchText.value)
  await instanceStore.searchInstances()
}

const handleSort = async (e) => {
  sort.value = e.option
  instanceStore.setFilter(sort.value)
  await instanceStore.searchInstances()
}

const handleLimit = async (e) => {
  limit.value = e.option
  instanceStore.setLimit(limit.value)
  await instanceStore.searchInstances()
}

const switchPage = async (page) => {
  currentPage.value = page
  instanceStore.setCurrentPage(page)
  await instanceStore.searchInstances()
}

const handleCheckbox = async () => {
  await instanceStore.searchInstances()
}

const handleVersionSelect = async () => {
  instanceStore.setVersions(selectedVersions.value.map((ver) => ver))
  await instanceStore.searchInstances()
}

const handleReset = async () => {
  instanceStore.resetFilters()
  selectedVersions.value = []
  await instanceStore.searchInstances()
}
</script>

<template>
  <div class="search-container">
    <aside class="filter-panel">
      <Button @click="handleReset"><ClearIcon />Clear Filters</Button>
      <div class="categories">
        <h2>Categories</h2>
        <div v-for="(val, category) in instanceStore.categories" :key="category">
          <Checkbox
            v-model="instanceStore.categories[category].enabled"
            @click="handleCheckbox"
            class="filter-checkbox"
          >
            <div class="checkbox-icon" v-html="getIconByFilter(category)" />
            {{ val.label }}
          </Checkbox>
        </div>
      </div>
      <div class="loaders">
        <h2>Loaders</h2>
        <div v-for="(val, loader) in instanceStore.loaders" :key="loader">
          <Checkbox
            v-model="instanceStore.loaders[loader].enabled"
            @click="handleCheckbox"
            class="filter-checkbox"
          >
            <div role="icon" class="checkbox-icon" v-html="getIconByFilter(loader)" />

            {{ val.label }}
          </Checkbox>
        </div>
      </div>
      <div class="environment">
        <h2>Environments</h2>
        <div v-for="(_, env) in instanceStore.environments" :key="env">
          <Checkbox
            v-model="instanceStore.environments[env]"
            @click="handleCheckbox"
            class="filter-checkbox"
          >
            <div class="checkbox-icon" v-html="getIconByFilter(env)" />

            {{ env }}
          </Checkbox>
        </div>
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
        <Checkbox
          v-model="instanceStore.openSource"
          @click="handleCheckbox"
          class="filter-checkbox"
        >
          Open source
        </Checkbox>
      </div>
    </aside>
    <div class="search">
      <div class="search-panel-container">
        <div class="search-panel">
          <div class="iconified-input">
            <SearchIcon />
            <input type="text" placeholder="Search.." v-model="searchText" @input="searchHandler" />
          </div>
          Sort by
          <DropdownSelect
            name="Sort dropdown"
            :options="[
              'Relevance',
              'Download count',
              'Follow count',
              'Recently published',
              'Recently updated',
            ]"
            :defaultValue="sort"
            @change="handleSort"
            :modelValue="sort"
          />
          Show per page
          <DropdownSelect
            name="Limit dropdown"
            :options="[5, 10, 15, 20, 50, 100]"
            :defaultValue="limit"
            @change="handleLimit"
            :modelValue="limit"
          />
        </div>
      </div>
      <Pagination :page="currentPage" :count="instanceStore.pageCount" @switch-page="switchPage" />
      <section class="project-list display-mode--list instance-results" role="list">
        <ProjectCard
          v-for="instance in instanceStore.instances"
          class="instance-project-item"
          :id="instance?.slug"
          :type="instance?.project_type"
          :name="instance?.title"
          :description="instance?.description"
          :iconUrl="instance?.icon_url"
          :downloads="instance?.downloads?.toString()"
          :follows="instance?.follows"
          :createdAt="instance?.date_created"
          :updatedAt="instance?.date_modified"
          :categories="getCategoriesByInstanceId(instance?.project_id)"
          :projectTypeDisplay="instance?.project_type"
          projectTypeUrl="mod"
          :serverSide="instance?.server_side"
          :clientSide="instance?.client_side"
          :showUpdatedDate="false"
          :color="instance?.color"
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

  .search-panel {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    gap: 1.3rem;
    margin: 1rem auto;

    .iconified-input {
      width: 50%;
    }
  }

  .filter-panel {
    display: flex;
    align-items: center;
    gap: 1rem;

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
    min-width: 16rem;
    background: var(--color-raised-bg);
    padding: 1rem;
    border-radius: var(--radius-sm);
    margin: 1rem;
    height: fit-content;
    display: block;

    h2 {
      color: var(--color-contrast);
      margin-top: 1rem;
      margin-bottom: 0.5rem;
      font-size: 0.9rem;
    }

    .filter-checkbox {
      margin-bottom: 0.3rem;
      font-size: 1rem;
      text-transform: capitalize;

      .checkbox-icon {
        margin-right: 0.4rem;
      }

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
    margin: 0 2rem;

    .instance-project-item {
      width: 100%;
      height: auto;
      cursor: pointer;
    }
  }
}

.multiselect {
  color: var(--color-base) !important;
  outline: 2px solid transparent;

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
    padding-top: 10px;

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
