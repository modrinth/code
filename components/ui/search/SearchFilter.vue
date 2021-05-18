<template>
  <p
    class="filter"
    :class="{
      'filter-active': activeFilters.includes(facetName),
      cursed: displayName == 'FlameAnvil',
    }"
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
  padding: 0.4rem 0.3rem;
  margin: 3px 0 0 0.5rem;
  font-size: 1rem;
  letter-spacing: 0.02rem;
  @extend %transparent-clickable;

  @media screen and (min-width: 1024px) {
    padding: 0.2rem 0.3rem;
  }

  svg {
    color: var(--color-icon);
    margin-right: 5px;
    height: 1rem;
    flex-shrink: 0;
  }
}

.filter-active {
  @extend %transparent-clickable, .selected;
  svg {
    color: var(--color-brand-light);
  }
}
</style>
