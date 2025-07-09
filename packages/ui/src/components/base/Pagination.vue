<template>
  <div v-if="count > 1" class="flex items-center gap-1">
    <ButtonStyled v-if="page > 1" circular type="transparent">
      <a
        v-if="linkFunction"
        aria-label="Previous Page"
        :href="linkFunction(page - 1)"
        @click.prevent="switchPage(page - 1)"
      >
        <ChevronLeftIcon />
      </a>
      <button v-else aria-label="Previous Page" @click="switchPage(page - 1)">
        <ChevronLeftIcon />
      </button>
    </ButtonStyled>
    <div
      v-for="(item, index) in pages"
      :key="'page-' + item + '-' + index"
      :class="{
        'page-number': page !== item,
        shrink: item !== '-' && item > 99,
      }"
      class="page-number-container"
    >
      <div v-if="item === '-'">
        <GapIcon />
      </div>
      <ButtonStyled
        v-else
        circular
        :color="page === item ? 'brand' : 'standard'"
        :type="page === item ? 'standard' : 'transparent'"
      >
        <a
          v-if="linkFunction"
          :href="linkFunction(item)"
          @click.prevent="page !== item ? switchPage(item) : null"
        >
          {{ item }}
        </a>
        <button v-else @click="page !== item ? switchPage(item) : null">{{ item }}</button>
      </ButtonStyled>
    </div>

    <ButtonStyled v-if="page !== pages[pages.length - 1]" circular type="transparent">
      <a
        v-if="linkFunction"
        aria-label="Next Page"
        :href="linkFunction(page + 1)"
        @click.prevent="switchPage(page + 1)"
      >
        <ChevronRightIcon />
      </a>
      <button v-else aria-label="Next Page" @click="switchPage(page + 1)">
        <ChevronRightIcon />
      </button>
    </ButtonStyled>
  </div>
</template>
<script setup lang="ts">
import { computed } from 'vue'
import { GapIcon, ChevronLeftIcon, ChevronRightIcon } from '@modrinth/assets'
import ButtonStyled from './ButtonStyled.vue'

const emit = defineEmits<{
  'switch-page': [page: number]
}>()

const props = withDefaults(
  defineProps<{
    page: number
    count: number
    linkFunction?: (page: number) => string | undefined
  }>(),
  {
    page: 1,
    count: 1,
    linkFunction: (page: number) => void page,
  },
)

const pages = computed(() => {
  const pages: ('-' | number)[] = []

  const first = 1
  const last = props.count
  const current = props.page
  const prev = current - 1
  const next = current + 1
  const gap = '-'

  if (prev > first) {
    pages.push(first)
  }
  if (prev > first + 1) {
    pages.push(gap)
  }
  if (prev >= first) {
    pages.push(prev)
  }
  pages.push(current)
  if (next <= last) {
    pages.push(next)
  }
  if (next < last - 1) {
    pages.push(gap)
  }
  if (next < last) {
    pages.push(last)
  }

  return pages
})

function switchPage(newPage: number) {
  emit('switch-page', Math.min(Math.max(newPage, 1), props.count))
}
</script>
