<template>
  <div :class="['site-banner', `site-banner--${variant}`, '[&>*]:z-[6]']">
    <div class="site-banner__title">
      <IssuesIcon aria-hidden="true" v-if="variant == 'warning' || variant == 'error'" />
      <InfoIcon aria-hidden="true" v-if="variant == 'info'" />
      <slot name="title" />
    </div>
    <div class="site-banner__description">
      <slot name="description" />
    </div>
    <div v-if="$slots.actions" class="site-banner__actions">
      <slot name="actions" />
    </div>
    <div v-if="$slots.actions_right" class="site-banner__actions_right">
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
.site-banner {
  display: grid;
  gap: 0.5rem;
  grid-template: "title actions_right" "description actions_right" "actions actions_right";
  padding-block: var(--gap-xl);
  padding-inline: max(calc((100% - 80rem) / 2 + var(--gap-md)), var(--gap-xl));
  position: relative;
  z-index: 4;

  .site-banner__title {
    grid-area: title;
    display: flex;
    gap: 0.5rem;
    align-items: center;
    font-weight: bold;
    font-size: var(--font-size-md);
    color: var(--color-contrast);

    svg {
      width: 1.5rem;
      height: 1.5rem;
      flex-shrink: 0;
    }
  }

  .site-banner__description {
    grid-area: description;
    display: flex;
    flex-direction: column;
    gap: var(--gap-md);
  }

  .site-banner__actions {
    grid-area: actions;
  }

  a {
    color: inherit;
  }

  &--error {
    background-color: var(--color-bg);
    border-bottom: 2px solid var(--color-red);

    &::before {
      content: "";
      position: absolute;
      inset: 0;
      background-color: var(--color-red-bg);
      z-index: 5;
    }

    .site-banner__title svg {
      color: var(--color-red);
    }

    a {
      color: var(--color-red);
    }
  }

  &--warning {
    background-color: var(--color-bg);
    border-bottom: 2px solid var(--color-orange);

    &::before {
      content: "";
      position: absolute;
      inset: 0;
      background-color: var(--color-orange-bg);
      z-index: 5;
    }

    .site-banner__title svg {
      color: var(--color-orange);
    }

    a {
      color: var(--color-orange);
    }
  }

  &--info {
    background-color: var(--color-bg);
    border-bottom: 2px solid var(--color-blue);

    &::before {
      content: "";
      position: absolute;
      inset: 0;
      background-color: var(--color-blue-bg);
      z-index: 5;
    }

    .site-banner__title svg {
      color: var(--color-blue);
    }

    a {
      color: var(--color-blue);
    }
  }
}
</style>
