<script setup>
import Instance from '@/components/ui/Instance.vue'
import {computed, ref} from 'vue'
import {SearchIcon, DropdownSelect, Card} from "omorphia";

const props = defineProps({
  instances: {
    type: Array,
    default() {
      return []
    },
  },
  news: {
    type: Array,
    default() {
      return []
    },
  },
  label: {
    type: String,
    default: '',
  },
  canPaginate: Boolean,
})

const modsRow = ref(null)

const search = ref('')
const filters = ref('All profiles')
const sortBy = ref('Name')

const filteredResults = computed(() => {
  let instances =  props.instances.filter((instance) => {
    return instance.metadata.name.toLowerCase().includes(search.value.toLowerCase())
  })

  if (sortBy.value === 'Name') {
    instances.sort((a, b) => {
      return a.metadata.name.localeCompare(b.metadata.name)
    })
  }

  if (sortBy.value === 'Game version') {
    instances.sort((a, b) => {
      return a.metadata.name.localeCompare(b.metadata.game_version)
    })
  }

  if (sortBy.value === 'Last played') {
    instances.sort((a, b) => {
      return b.metadata.last_played - a.metadata.last_played
    })
  }

  if (sortBy.value === 'Date created') {
    instances.sort((a, b) => {
      return b.metadata.date_created - a.metadata.date_created
    })
  }

  if (sortBy.value === 'Date modified') {
    instances.sort((a, b) => {
      return b.metadata.date_modified - a.metadata.date_modified
    })
  }

  if (!filters.value || filters.value === 'All profiles') {
    return instances
  } else if (filters.value === 'Custom instances') {
    return instances.filter((instance) => {
      return !instance.metadata?.linked_data
    })
  } else if (filters.value === 'Downloaded modpacks') {
    return instances.filter((instance) => {
      return instance.metadata?.linked_data
    })
  }

  return instances
})

</script>
<template>
  <div class="row">
    <Card class="header">
      <div class="iconified-input">
        <SearchIcon/>
        <input
          v-model="search"
          type="text"
          placeholder="Search"
          class="search-input"
        />
      </div>
      <div class="labeled_button">
        <span>Sort by</span>
        <DropdownSelect
          v-model="sortBy"
          class="sort-dropdown"
          :options="[
            'Name',
            'Last played',
            'Date created',
            'Date modified',
            'Game version',
          ]"
          placeholder="Select..."
        />
      </div>
      <div class="labeled_button">
        <span>Filter by</span>
        <DropdownSelect
          v-model="filters"
          class="filter-dropdown"
          :options="[
            'All profiles',
            'Custom instances',
            'Downloaded modpacks'
          ]"
          placeholder="Select..."
        />
      </div>
    </Card>
    <section ref="modsRow" class="instances">
      <Instance
        v-for="instance in filteredResults"
        :key="instance.id"
        display="card"
        :instance="instance"
      />
    </section>
  </div>
</template>
<style lang="scss" scoped>
.row {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 100%;
  padding: 1rem;

  .header {
    display: flex;
    gap: 0.5rem;
    align-items: inherit;
    width: 100%;
    margin-bottom: 0 !important;
    padding: 1rem;

    .iconified-input {
      flex-grow: 1;
    }

    .sort-dropdown {
      width: 10rem;
    }

    .filter-dropdown {
      width: 15rem;
    }

    .labeled_button {
      display: flex;
      flex-direction: row;
      align-items: center;
      gap: 0.5rem;
      white-space: nowrap;
    }
  }

  section {
    display: flex;
    align-items: inherit;
    transition: all ease-in-out 0.4s;
    gap: 1rem;
  }

  .instances {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(10rem, 1fr));
    width: 100%;
    gap: 1rem;
    margin-right: auto;
    margin-top: 0.8rem;
    scroll-behavior: smooth;
    overflow-y: auto;
  }
}

.dark-mode {
  .row {
    &:nth-child(odd) {
      background-color: rgb(30, 31, 34);
    }
  }
}
</style>
