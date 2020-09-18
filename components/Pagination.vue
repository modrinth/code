<template>
  <div v-if="pages.length > 1" class="columns paginates">
    <svg
      :class="{ 'disabled-paginate': currentPage === 1 }"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
      @click="currentPage !== 1 ? switchPage(currentPage - 1) : null"
    >
      <polyline points="15 18 9 12 15 6"></polyline>
    </svg>
    <p
      v-for="(item, index) in pages"
      :key="'page-' + item"
      :class="{
        'active-page-number': currentPage !== item,
      }"
      @click="currentPage !== item ? switchPage(item) : null"
    >
      <span v-if="pages[index - 1] + 1 !== item && item !== 1">...</span>
      <span :class="{ 'disabled-page-number': currentPage === item }">{{
        item
      }}</span>
    </p>

    <svg
      :class="{
        'disabled-paginate': currentPage === pages[pages.length - 1],
      }"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
      @click="
        currentPage !== pages[pages.length - 1]
          ? switchPage(currentPage + 1)
          : null
      "
    >
      <polyline points="9 18 15 12 9 6"></polyline>
    </svg>
  </div>
</template>

<script>
export default {
  name: 'Pagination',
  props: {
    currentPage: {
      type: Number,
      default: 1,
    },
    pages: {
      type: Array,
      default() {
        return []
      },
    },
  },
  methods: {
    switchPage(newPage) {
      this.$emit('switch-page', newPage)
    },
  },
}
</script>

<style scoped>
.paginates {
  align-items: center;
}

.paginates p {
  margin-left: 5px;
  margin-right: 5px;
}

.disabled-paginate {
  cursor: default;
  color: var(--color-grey-5);
}

.active-page-number {
  user-select: none;
  cursor: pointer;
}

.disabled-page-number {
  user-select: none;
  cursor: default;
  padding: 2px 3px;
  border-radius: 3px;
  background-color: var(--color-grey-1);
}
</style>
