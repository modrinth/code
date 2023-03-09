<script setup>
import { ref } from 'vue'
import {
  SearchIcon,
  Button,
  BoxIcon,
  BookIcon,
  ContractIcon,
  TransferIcon,
  ServerIcon,
} from 'omorphia'
import { useInstances } from '@/store/state'

const searchText = ref('')

const instanceStore = useInstances()
</script>

<template>
  <div class="search-panel-container">
    <div class="search-panel">
      <div class="iconified-input">
        <SearchIcon />
        <input
          type="text"
          placeholder="Search.."
          v-model="searchText"
          @input="instanceStore.setFilter(searchText)"
        />
      </div>
      <Button>Search by relevance</Button>
    </div>
    <!-- Open a modal onClick to filter these things? -->
    <div class="filter-panel">
      <Button><BoxIcon />Minecraft version</Button>
      <Button><ContractIcon />Categories</Button>
      <Button><ServerIcon />Environment</Button>
      <Button><TransferIcon />Mod loaders</Button>
      <Button><BookIcon />License</Button>
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
  margin: auto;

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
