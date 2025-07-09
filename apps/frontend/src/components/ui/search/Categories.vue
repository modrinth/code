<template>
  <div class="categories">
    <slot />
    <span
      v-for="category in categoriesFiltered"
      :key="category.name"
      v-html="category.icon + formatCategory(category.name)"
    />
  </div>
</template>

<script>
import { formatCategory } from "@modrinth/utils";

export default {
  props: {
    categories: {
      type: Array,
      default() {
        return [];
      },
    },
    type: {
      type: String,
      required: true,
    },
  },
  setup() {
    const tags = useTags();

    return { tags };
  },
  computed: {
    categoriesFiltered() {
      return this.tags.categories
        .concat(this.tags.loaders)
        .filter(
          (x) =>
            this.categories.includes(x.name) && (!x.project_type || x.project_type === this.type),
        );
    },
  },
  methods: { formatCategory },
};
</script>

<style lang="scss" scoped>
.categories {
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;

  :deep(span) {
    display: flex;
    align-items: center;
    flex-direction: row;

    &:not(:last-child) {
      margin-right: var(--spacing-card-md);
    }

    &:not(.badge) {
      color: var(--color-icon);
    }

    svg {
      width: 1rem;
      margin-right: 0.2rem;
    }
  }
}
</style>
