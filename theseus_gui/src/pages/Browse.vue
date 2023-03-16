<script setup>
import SearchPanel from '@/components/SearchPanel.vue'
import Instance from '@/components/ui/Instance.vue'
import { useInstances } from '@/store/state'

const instanceStore = useInstances()
console.log(instanceStore.getFilteredInstances)
</script>

<template>
  <div class="search-container">
    <SearchPanel />
    <section class="project-list display-mode--list instance-results">
      <Instance
        v-for="instance in instanceStore.getFilteredInstances"
        :id="instance.id"
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
