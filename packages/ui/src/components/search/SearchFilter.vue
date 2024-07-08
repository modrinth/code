<template>
  <Checkbox
    class="filter"
    :model-value="isActive"
    :description="displayName"
    @update:model-value="toggle"
  >
    <div class="filter-text">
      <div v-if="props.icon" aria-hidden="true" class="icon" v-html="props.icon" />
      <div v-else class="icon">
        <slot />
      </div>
      <span aria-hidden="true"> {{ props.displayName }}</span>
    </div>
  </Checkbox>
</template>

<script setup>
import { computed } from 'vue'
import Checkbox from '../base/Checkbox.vue'

const props = defineProps({
  facetName: {
    type: String,
    default: '',
  },
  displayName: {
    type: String,
    default: '',
  },
  icon: {
    type: String,
    default: '',
  },
  activeFilters: {
    type: Array,
    default() {
      return []
    },
  },
})

const isActive = computed(() => props.activeFilters.includes(props.facetName))
const emit = defineEmits(['toggle'])

const toggle = () => {
  emit('toggle', props.facetName)
}
</script>

<style lang="scss" scoped>
.filter {
  margin-bottom: 0.5rem;

  :deep(.filter-text) {
    display: flex;
    align-items: center;

    .icon {
      height: 1rem;

      svg {
        margin-right: 0.25rem;
        width: 1rem;
        height: 1rem;
      }
    }
  }

  span {
    user-select: none;
  }
}
</style>
