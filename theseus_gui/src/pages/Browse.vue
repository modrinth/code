<script setup>
import { ref } from 'vue'
import { storeToRefs } from 'pinia'
import { Pagination, ProjectCard } from 'omorphia'
import SearchPanel from '@/components/SearchPanel.vue'
import { useInstances } from '@/store/state'

const currentPage = ref(1)

const instanceStore = useInstances()
await instanceStore.searchInstances()
const { getCategoriesByInstanceId } = storeToRefs(instanceStore)

const switchPage = async (page) => {
  currentPage.value = page
  instanceStore.setCurrentPage(page)
  await instanceStore.searchInstances()
}
</script>

<template>
  <div class="search-container">
    <SearchPanel />
    <Pagination :page="currentPage" :count="instanceStore.pageCount" @switch-page="switchPage" />
    <section class="project-list display-mode--list instance-results">
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
</template>

<style lang="scss" scoped>
.search-container {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;

  .instance-results {
    width: 90%;
  }

  .instance-project-item {
    width: 100%;
    height: auto;
    margin: 0.75rem auto;
    cursor: pointer;
  }
}
</style>
