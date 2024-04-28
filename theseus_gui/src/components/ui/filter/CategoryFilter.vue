<script setup>
import { computed, ref } from 'vue'
import { get_categories, sortByNameOrNumber } from '@/helpers/tags.js'
import { handleError } from '@/store/notifications.js'
import { formatCategory, formatCategoryHeader, SearchFilter } from 'omorphia'

const props = defineProps({
  projectTypes: {
    type: Object,
    required: true,
  },
  facets: {
    type: Object,
    required: true,
  },
})

const emits = defineEmits(['toggleFacet'])

function onToggleFacet(elementName) {
  emits('toggleFacet', elementName)
}

const projectTypes = computed(() => {
  return props.projectTypes.map((type) => {
    return type === 'datapack' ? 'mod' : type
  })
})

const sortedCategories = computed(() => {
  const values = new Map()

  console.log(projectTypes.value)
  console.log('hi')

  for (const category of categories.value.filter((cat) => {
    return projectTypes.value.includes(cat.project_type)
  })) {
    if (!values.has(category.header)) {
      values.set(category.header, [])
    }
    values.get(category.header).push(category)
  }
  return values
})

const categories = await get_categories()
  .catch(handleError)
  .then((s) => sortByNameOrNumber(s, ['header', 'name']))
  .then(ref)
</script>

<template>
  <div
    v-for="categoryList in Array.from(sortedCategories)"
    :key="categoryList[0]"
    class="categories"
  >
    <h2>{{ formatCategoryHeader(categoryList[0]) }}</h2>
    <div v-for="category in categoryList[1]" :key="category.name">
      <SearchFilter
        :active-filters="facets"
        :icon="category.icon"
        :display-name="formatCategory(category.name)"
        :facet-name="`categories:${encodeURIComponent(category.name)}`"
        class="filter-checkbox"
        @toggle="onToggleFacet"
      />
    </div>
  </div>
</template>

<style scoped lang="scss"></style>
