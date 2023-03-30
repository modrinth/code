<script setup>
import { ref } from 'vue'
import { SearchIcon, DropdownSelect } from 'omorphia'
import { useInstances } from '@/store/instancesStore'

const instanceStore = useInstances()
const searchText = ref('')
const sort = ref('Relevance')

const searchHandler = async () => {
  instanceStore.setSearchInput(searchText.value)
  await instanceStore.searchInstances()
}

const handleSort = async (e) => {
  sort.value = e.option
  instanceStore.setFilter(sort.value)
  await instanceStore.searchInstances()
}
</script>

<template>
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
    </div>
  </div>
</template>

<style lang="scss" scoped>
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
      width: 60%;
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
</style>
