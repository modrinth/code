<template>
  <p
    class="filter"
    :class="{ 'filter-active': activeFilters.includes(facetName) }"
    @click="toggle"
  >
    <slot></slot>
    {{ displayName }}
  </p>
</template>

<script>
export default {
  name: 'SearchFilter',
  props: {
    facetName: {
      type: String,
      default: '',
    },
    displayName: {
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

<style lang="scss">
.filter {
  display: flex;
  align-items: center;
  cursor: pointer;
  padding: 2px 2px 2px 20px;
  margin: 0 0 0 5px;
  border-left: 4px solid var(--color-grey-3);
  border-radius: 0 0.25rem 0.25rem 0;
  color: var(--color-grey-5);
  font-size: 1rem;
  letter-spacing: 0.02rem;

  svg {
    margin-right: 5px;
    height: 1rem;
    flex-shrink: 0;
  }

  &:hover,
  &:focus {
    background-color: var(--color-grey-1);
    color: var(--color-text);
  }
}

.filter-active {
  background-color: var(--color-grey-1);
  color: var(--color-text);
  border-left: 4px solid var(--color-brand);
}
</style>
