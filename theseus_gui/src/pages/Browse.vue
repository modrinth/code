<script setup>
import { ref } from 'vue'
import { storeToRefs } from 'pinia'
import { Pagination, ProjectCard, Checkbox, Button, ClearIcon } from 'omorphia'
import SearchPanel from '@/components/SearchPanel.vue'
import { useInstances } from '@/store/state'

const instanceStore = useInstances()
await instanceStore.searchInstances()
const { getCategoriesByInstanceId } = storeToRefs(instanceStore)
const { getIconByFilter } = storeToRefs(instanceStore)

const currentPage = ref(1)

const switchPage = async (page) => {
  currentPage.value = page
  instanceStore.setCurrentPage(page)
  await instanceStore.searchInstances()
}

const handleCheckbox = async () => {
  await instanceStore.searchInstances()
}

const handleReset = async () => {
  await instanceStore.resetFilters()
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
            <div v-html="getIconByFilter(category)" />
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
            <div v-html="getIconByFilter(env)" />

            {{ env }}
          </Checkbox>
        </div>
      </div>
    </aside>
    <div class="search">
      <SearchPanel />
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

<style lang="scss">
.search-container {
  display: flex;

  .filter-panel {
    min-width: 16rem;
    background: var(--color-raised-bg);
    padding: 1rem;
    border-radius: var(--radius-sm);
    margin: 1rem;
    height: initial;
    max-height: 50rem;

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

      svg {
        display: flex;
        align-self: center;
        justify-self: center;
        margin-right: 0.3rem;
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
</style>
