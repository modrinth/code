<template>
  <div v-if="pages.length > 1" class="columns paginates">
    <button
      :class="{ disabled: currentPage === 1 }"
      class="paginate has-icon"
      aria-label="Previous Page"
      @click="currentPage !== 1 ? switchPage(currentPage - 1) : null"
    >
      <LeftArrowIcon />
    </button>
    <div
      v-for="(item, index) in pages"
      :key="'page-' + item"
      :class="{
        'page-number': currentPage !== item,
      }"
      class="page-number-container"
    >
      <div v-if="pages[index - 1] + 1 !== item && item !== 1" class="has-icon">
        <GapIcon />
      </div>
      <button
        :class="{ 'page-number current': currentPage === item }"
        @click="currentPage !== item ? switchPage(item) : null"
      >
        {{ item }}
      </button>
    </div>

    <button
      :class="{ disabled: currentPage === pages[pages.length - 1] }"
      class="paginate has-icon"
      aria-label="Next Page"
      @click="
        currentPage !== pages[pages.length - 1]
          ? switchPage(currentPage + 1)
          : null
      "
    >
      <RightArrowIcon />
    </button>
  </div>
</template>

<script>
import GapIcon from '~/assets/images/utils/gap.svg?inline'
import LeftArrowIcon from '~/assets/images/utils/left-arrow.svg?inline'
import RightArrowIcon from '~/assets/images/utils/right-arrow.svg?inline'

export default {
  name: 'Pagination',
  components: {
    GapIcon,
    LeftArrowIcon,
    RightArrowIcon,
  },
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

<style scoped lang="scss">
button {
  min-width: 2rem;
  padding: 0 0.5rem;
  height: 2rem;
  border-radius: 2rem;
  background: transparent;
  &.page-number.current {
    background: var(--color-button-bg-hover);
    color: var(--color-button-text-hover);
    cursor: default;
  }
  &.paginate.disabled {
    background: none;
    color: var(--color-button-text-disabled);
    cursor: default;
  }
  &:hover {
    background: var(--color-button-bg-active);
    color: var(--color-button-text-active);
  }
}

.has-icon {
  display: flex;
  align-items: center;
  padding: 0 0.5rem;
  height: 2rem;
  svg {
    width: 1rem;
  }
}

.page-number-container {
  display: flex;
  max-height: 2rem;
}
</style>
