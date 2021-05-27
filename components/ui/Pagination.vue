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
      :key="'page-' + item + '-' + index"
      :class="{
        'page-number': currentPage !== item,
        shrink: item > 99,
      }"
      class="page-number-container"
    >
      <div v-if="item == '-'" class="has-icon">
        <GapIcon />
      </div>
      <button
        v-else
        :class="{
          'page-number current': currentPage === item,
          shrink: item > 99,
        }"
        @click="currentPage !== item ? switchPage(item) : null"
      >
        {{ item }}
      </button>
    </div>

    <button
      :class="{
        disabled: currentPage === pages[pages.length - 1],
      }"
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
  padding: 0;
  margin: 0;
  width: 2em;
  height: 2em;
  border-radius: 2em;
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
  height: 2em;
  svg {
    width: 1em;
  }
}

.page-number-container,
button,
.has-icon {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 2em;
  width: 2em;
}

.paginates {
  height: 2em;
  margin: 0.5rem 0;
  > div {
    margin: 0 0.1em;
  }
  font-size: 80%;
  @media screen and (min-width: 350px) {
    font-size: 100%;
  }
}

.shrink {
  font-size: 0.9rem;
  height: 2.225em;
  width: 2.225em;
}
</style>
