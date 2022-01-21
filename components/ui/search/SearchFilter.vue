<template>
  <Checkbox
    class="filter"
    :value="activeFilters.includes(facetName)"
    :description="displayName"
    @input="toggle()"
  >
    <div class="filter-text">
      <div v-if="icon" aria-hidden="true" class="icon" v-html="icon"></div>
      <div v-else class="icon"><slot /></div>
      <span aria-hidden="true"> {{ displayName }}</span>
    </div>
  </Checkbox>
</template>

<script>
import Checkbox from '~/components/ui/Checkbox'

export default {
  name: 'SearchFilter',
  components: {
    Checkbox,
  },
  props: {
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
  },
  methods: {
    toggle() {
      this.$emit('toggle', this.facetName)
    },
  },
}
</script>

<style lang="scss" scoped>
.filter ::v-deep {
  margin-bottom: 0.5rem;

  .filter-text {
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
    text-transform: capitalize;
    user-select: none;
  }
}
</style>
