<template>
  <div
    :class="[
      'site-banner relative grid gap-2 py-8 px-4 lg:px-8 z-4',
      `site-banner--${variant}`
    ]"
  >
    <div class="site-banner__title flex items-center gap-2 font-bold text-base text-contrast z-6">
      <IssuesIcon
        v-if="variant === 'warning' || variant === 'error'"
        class="w-6 h-6 flex-shrink-0"
        aria-hidden="true"
      />
      <InfoIcon
        v-if="variant === 'info'"
        class="w-6 h-6 flex-shrink-0"
        aria-hidden="true"
      />
      <slot name="title" />
    </div>

    <div class="site-banner__description flex flex-col gap-4">
      <slot name="description" />
    </div>

    <div v-if="$slots.actions" class="site-banner__actions" >
      <slot name="actions" />
    </div>

    <div v-if="$slots.actions_right" class="site-banner__actions_right" >
      <slot name="actions_right" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { IssuesIcon, InfoIcon } from '@modrinth/assets'

const props = defineProps({
  variant: {
    type: String as () => 'error' | 'warning' | 'info',
    required: true,
    validator: (v: string) => ['error', 'warning', 'info'].includes(v),
  },
})
</script>

<style scoped lang="scss">
$variants: (
  error: (
    border: var(--color-red),
    bg:     var(--color-red-bg)
  ),
  warning: (
    border: var(--color-orange),
    bg:     var(--color-orange-bg)
  ),
  info: (
    border: var(--color-blue),
    bg:     var(--color-blue-bg)
  )
);

.site-banner {
  grid-template-areas:
    "title actions_right"
    "description actions_right"
    "actions actions_right";

  @each $name, $cols in $variants {
    &--#{$name} {
      background-color: var(--color-bg);
      border-bottom: 2px solid map-get($cols, border);

      &::before {
        content: '';
        position: absolute;
        inset: 0;
        background-color: map-get($cols, bg);
        z-index: 5;
      }

      .site-banner__title svg,
      a {
        color: map-get($cols, border);
      }
    }
  }

  .site-banner__title {
    grid-area: title;
  }

  .site-banner__description {
    grid-area: description;
  }

  .site-banner__actions {
    grid-area: actions;
  }

  .site-banner__actions_right {
    grid-area: actions_right;
  }

  a {
    color: inherit;
  }
}
</style>
