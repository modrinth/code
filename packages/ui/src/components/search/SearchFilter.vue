<template>
  <Checkbox
    v-if="!threePhase"
    class="filter"
    :model-value="isActive || isDisabled"
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
  <ThreeStateCheckbox
    v-else
    class="filter"
    :model-value="isActive ? 'POSITIVE' : isDisabled ? 'NEGATIVE' : 'NEUTRAL'"
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
  </ThreeStateCheckbox>
</template>

<script setup>
import { computed } from 'vue'
import Checkbox from '../base/Checkbox.vue'
import ThreeStateCheckbox from '../base/ThreeStateCheckbox.vue'

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
  threePhase: {
    type: Boolean,
    default: false,
  },
  disabledFilters: {
    type: Array,
    default() {
      return []
    },
  },
})

const isActive = computed(() => props.activeFilters.includes(props.facetName))
const isDisabled = computed(() => props.disabledFilters.includes(props.facetName))
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
