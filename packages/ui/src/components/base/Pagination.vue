<template>
  <div v-if="count > 1" class="paginates">
    <a
      :class="{ disabled: page === 1 }"
      :tabindex="page === 1 ? -1 : 0"
      class="left-arrow paginate has-icon"
      aria-label="Previous Page"
      :href="linkFunction(page - 1)"
      @click.prevent="page !== 1 ? switchPage(page - 1) : null"
    >
      <LeftArrowIcon />
    </a>
    <div
      v-for="(item, index) in pages"
      :key="'page-' + item + '-' + index"
      :class="{
        'page-number': page !== item,
        shrink: item !== '-' && item > 99,
      }"
      class="page-number-container"
    >
      <div v-if="item === '-'" class="has-icon">
        <GapIcon />
      </div>
      <a
        v-else
        :class="{
          'page-number current': page === item,
          shrink: item > 99,
        }"
        :href="linkFunction(item)"
        @click.prevent="page !== item ? switchPage(item) : null"
      >
        {{ item }}
      </a>
    </div>

    <a
      :class="{
        disabled: page === pages[pages.length - 1],
      }"
      :tabindex="page === pages[pages.length - 1] ? -1 : 0"
      class="right-arrow paginate has-icon"
      aria-label="Next Page"
      :href="linkFunction(page + 1)"
      @click.prevent="page !== pages[pages.length - 1] ? switchPage(page + 1) : null"
    >
      <RightArrowIcon />
    </a>
  </div>
</template>
<script setup lang="ts">
import { computed } from 'vue'
import { GapIcon, LeftArrowIcon, RightArrowIcon } from '@modrinth/assets'

const emit = defineEmits<{
  'switch-page': [page: number]
}>()

const props = withDefaults(
  defineProps<{
    page: number
    count: number
    linkFunction: (page: number) => string | undefined
  }>(),
  {
    page: 1,
    count: 1,
    linkFunction: (page: number) => void page,
  },
)

const pages = computed(() => {
  let pages: ('-' | number)[] = []

  if (props.count > 7) {
    if (props.page + 3 >= props.count) {
      pages = [
        1,
        '-',
        props.count - 4,
        props.count - 3,
        props.count - 2,
        props.count - 1,
        props.count,
      ]
    } else if (props.page > 5) {
      pages = [1, '-', props.page - 1, props.page, props.page + 1, '-', props.count]
    } else {
      pages = [1, 2, 3, 4, 5, '-', props.count]
    }
  } else {
    pages = Array.from({ length: props.count }, (_, i) => i + 1)
  }

  return pages
})

function switchPage(newPage: number) {
  emit('switch-page', Math.min(Math.max(newPage, 1), props.count))
}
</script>

<style lang="scss" scoped>
.paginates {
  display: flex;
}

a {
  color: var(--color-contrast);
  box-shadow: var(--shadow-raised), var(--shadow-inset);

  padding: 0.5rem 1rem;
  margin: 0;
  border-radius: 2rem;
  background: var(--color-raised-bg);
  cursor: pointer;

  transition:
    opacity 0.5s ease-in-out,
    filter 0.2s ease-in-out,
    transform 0.05s ease-in-out,
    outline 0.2s ease-in-out;

  @media (prefers-reduced-motion) {
    transition: none !important;
  }

  &:hover {
    color: inherit;
    text-decoration: none;
  }

  &.page-number.current {
    background: var(--color-brand);
    color: var(--color-accent-contrast);
    cursor: default;
  }

  &.paginate.disabled {
    background-color: transparent;
    cursor: not-allowed;
    filter: grayscale(50%);
    opacity: 0.5;
  }

  &:hover:not(&:disabled) {
    filter: brightness(0.85);
  }

  &:active:not(&:disabled) {
    transform: scale(0.95);
    filter: brightness(0.8);
  }
}

.has-icon {
  display: flex;
  align-items: center;
  svg {
    width: 1em;
  }
}

.page-number-container,
a,
.has-icon {
  display: flex;
  justify-content: center;
  align-items: center;
}

.paginates {
  height: 2em;
  margin: 0.5rem 0;
  > div,
  .has-icon {
    margin: 0 0.3em;
  }
}

.left-arrow {
  margin-left: auto !important;
}

.right-arrow {
  margin-right: auto !important;
}

@media screen and (max-width: 400px) {
  .paginates {
    font-size: 80%;
  }
}

@media screen and (max-width: 530px) {
  a {
    width: 2.5rem;
    padding: 0.5rem 0;
  }
}
</style>
