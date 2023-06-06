<script setup>
import Instance from '@/components/ui/Instance.vue'
import { computed, ref } from 'vue'
import { SearchIcon, DropdownSelect, Card, formatCategoryHeader } from 'omorphia'
import dayjs from 'dayjs'

const props = defineProps({
  instances: {
    type: Array,
    default() {
      return []
    },
  },
  label: {
    type: String,
    default: '',
  },
})

const search = ref('')
const group = ref('Category')
const filters = ref('All profiles')
const sortBy = ref('Name')

const filteredResults = computed(() => {
  let instances = props.instances.filter((instance) => {
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
      return dayjs(b.metadata.last_played ?? 0).diff(dayjs(a.metadata.last_played ?? 0))
    })
  }

  if (sortBy.value === 'Date created') {
    instances.sort((a, b) => {
      return dayjs(b.metadata.date_created).diff(dayjs(a.metadata.date_created))
    })
  }

  if (sortBy.value === 'Date modified') {
    instances.sort((a, b) => {
      return dayjs(b.metadata.date_modified).diff(dayjs(a.metadata.date_modified))
    })
  }

  if (filters.value === 'Custom instances') {
    instances = instances.filter((instance) => {
      return !instance.metadata?.linked_data
    })
  } else if (filters.value === 'Downloaded modpacks') {
    instances = instances.filter((instance) => {
      return instance.metadata?.linked_data
    })
  }

  const instanceMap = new Map()

  if (group.value === 'Loader') {
    instances.forEach((instance) => {
      const loader = formatCategoryHeader(instance.metadata.loader)
      if (!instanceMap.has(loader)) {
        instanceMap.set(loader, [])
      }

      instanceMap.get(loader).push(instance)
    })
  } else if (group.value === 'Game version') {
    instances.forEach((instance) => {
      if (!instanceMap.has(instance.metadata.game_version)) {
        instanceMap.set(instance.metadata.game_version, [])
      }

      instanceMap.get(instance.metadata.game_version).push(instance)
    })
  } else if (group.value === 'Category') {
    instances.forEach((instance) => {
      if (instance.metadata.groups.length === 0) {
        instance.metadata.groups.push('None')
      }

      for (const category of instance.metadata.groups) {
        if (!instanceMap.has(category)) {
          instanceMap.set(category, [])
        }

        instanceMap.get(category).push(instance)
      }
    })
  } else {
    return instanceMap.set('None', instances)
  }

  return instanceMap
})
</script>
<template>
  <Card class="header">
    <div class="iconified-input">
      <SearchIcon />
      <input v-model="search" type="text" placeholder="Search" class="search-input" />
    </div>
    <div class="labeled_button">
      <span>Sort by</span>
      <DropdownSelect
        v-model="sortBy"
        class="sort-dropdown"
        :options="['Name', 'Last played', 'Date created', 'Date modified', 'Game version']"
        placeholder="Select..."
      />
    </div>
    <div class="labeled_button">
      <span>Filter by</span>
      <DropdownSelect
        v-model="filters"
        class="filter-dropdown"
        :options="['All profiles', 'Custom instances', 'Downloaded modpacks']"
        placeholder="Select..."
      />
    </div>
    <div class="labeled_button">
      <span>Group by</span>
      <DropdownSelect
        v-model="group"
        class="group-dropdown"
        :options="['Category', 'Loader', 'Game version', 'None']"
        placeholder="Select..."
      />
    </div>
  </Card>
  <div
    v-for="instanceSection in Array.from(filteredResults, ([key, value]) => ({ key, value }))"
    :key="instanceSection.key"
    class="row"
  >
    <div v-if="instanceSection.key !== 'None'" class="divider">
      <p>{{ instanceSection.key }}</p>
      <hr aria-hidden="true" />
    </div>
    <section class="instances">
      <Instance
        v-for="instance in instanceSection.value"
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
  align-items: flex-start;
  width: 100%;
  padding: 1rem;

  .divider {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
    gap: 1rem;
    margin-bottom: 1rem;

    p {
      margin: 0;
      font-size: 1rem;
      white-space: nowrap;
      color: var(--color-contrast);
    }

    hr {
      background-color: var(--color-gray);
      height: 1px;
      width: 100%;
      border: none;
    }
  }
}

.header {
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
  justify-content: space-between;
  gap: 1rem;
  align-items: inherit;
  margin: 1rem 1rem 0 !important;
  padding: 1rem;
  width: calc(100% - 2rem);

  .iconified-input {
    flex-grow: 1;
  }

  .sort-dropdown {
    width: 10rem;
  }

  .filter-dropdown {
    width: 15rem;
  }

  .group-dropdown {
    width: 10rem;
  }

  .labeled_button {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 0.5rem;
    white-space: nowrap;
  }
}

.instances {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(10rem, 1fr));
  width: 100%;
  gap: 1rem;
  margin-right: auto;
  scroll-behavior: smooth;
  overflow-y: auto;
}
</style>
