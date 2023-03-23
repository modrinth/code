<script setup>
import { ref } from 'vue'
import { Pagination } from 'omorphia'
import SearchPanel from '@/components/SearchPanel.vue'
import Instance from '@/components/ui/Instance.vue'
import { useInstances } from '@/store/state'

const currentPage = ref(1)

const instanceStore = useInstances()
await instanceStore.searchInstances()

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
      <Instance
        v-for="instance in instanceStore.instances"
        :id="instance.project_id"
        :instance="instance"
        display="project"
      />
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
}
</style>
