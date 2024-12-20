<template>
  <div class="tooltip-parent flex items-center justify-center">
    <RouterLink
      v-if="typeof to === 'string'"
      :to="to"
      v-bind="$attrs"
      :class="{
        'router-link-active': isPrimary && isPrimary(route),
        'subpage-active': isSubpage && isSubpage(route),
      }"
      class="w-12 h-12 rounded-full flex items-center justify-center text-2xl transition-all bg-transparent hover:bg-button-bg hover:text-contrast"
    >
      <slot />
    </RouterLink>
    <button
      v-else
      v-bind="$attrs"
      class="button-animation border-none text-primary cursor-pointer w-12 h-12 rounded-full flex items-center justify-center text-2xl transition-all bg-transparent hover:bg-button-bg hover:text-contrast"
      @click="to"
    >
      <slot />
    </button>
    <div class="tooltip-label">
      <slot name="label" />
    </div>
  </div>
</template>

<script setup lang="ts">
import type { RouteLocationNormalizedLoaded } from 'vue-router'
import { RouterLink, useRoute } from 'vue-router'

const route = useRoute()

type RouteFunction = (route: RouteLocationNormalizedLoaded) => boolean

defineProps<{
  to: (() => void) | string
  isPrimary?: RouteFunction
  isSubpage?: RouteFunction
  highlightOverride?: boolean
}>()

defineOptions({
  inheritAttrs: false,
})
</script>

<style lang="scss" scoped>
.router-link-active,
.subpage-active {
  svg {
    filter: drop-shadow(0 0 0.5rem black);
  }
}

.router-link-active {
  @apply text-[--color-button-text-selected] bg-[--color-button-bg-selected];
}

.subpage-active {
  @apply text-contrast bg-button-bg;
}

.tooltip-parent {
  position: relative;
  border-radius: var(--radius-max);
}

.tooltip-parent:hover .tooltip-label {
  opacity: 1;
  translate: 0 0;
  scale: 1;
}

.tooltip-label:not(:empty) {
  --_tooltip-bg: black;
  --_tooltip-color: var(--dark-color-contrast);

  position: absolute;
  background-color: var(--_tooltip-bg);
  color: var(--_tooltip-color);
  text-wrap: nowrap;
  padding: 0.5rem 0.5rem;
  border-radius: var(--radius-sm);
  left: calc(100% + 0.5rem);
  font-size: 1rem;
  line-height: 1;
  font-weight: bold;
  filter: drop-shadow(5px 5px 0.8rem rgba(0, 0, 0, 0.35));
  pointer-events: none;
  user-select: none;

  opacity: 0;
  translate: -0.5rem 0;
  scale: 0.9;
  transition: all ease-in-out 0.1s;
}

.tooltip-label:not(:empty)::after {
  content: '';
  position: absolute;
  top: 50%;
  right: 100%; /* To the left of the tooltip */
  margin-top: -5px;
  border-width: 5px;
  border-style: solid;
  border-color: transparent var(--_tooltip-bg) transparent transparent;
}
</style>
