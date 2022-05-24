<template>
  <div class="categories">
    <span
      v-for="category in categoriesFiltered"
      :key="category.name"
      v-html="
        category.icon +
        (category.name === 'modloader' ? 'ModLoader' : category.name)
      "
    />
  </div>
</template>

<script>
export default {
  name: 'Categories',
  props: {
    categories: {
      type: Array,
      default() {
        return []
      },
    },
    type: {
      type: String,
      required: true,
    },
  },
  computed: {
    categoriesFiltered() {
      return this.$tag.categories
        .concat(this.$tag.loaders)
        .filter(
          (x) =>
            this.categories.includes(x.name) &&
            (!x.project_type || x.project_type === this.type)
        )
    },
  },
}
</script>

<style lang="scss" scoped>
.categories {
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;

  span ::v-deep {
    display: flex;
    align-items: center;
    flex-direction: row;
    color: var(--color-icon);
    margin-right: 1em;
    text-transform: capitalize;

    svg {
      width: 1rem;
      margin-right: 0.125rem;
    }
  }
}
</style>
