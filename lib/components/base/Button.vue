<script setup>
import { ExternalIcon, UnknownIcon } from '@/components'

import { computed } from 'vue'

const props = defineProps({
  link: {
    type: String,
    default: null,
  },
  external: {
    type: Boolean,
    default: false,
  },
  action: {
    type: Function,
    default: null,
  },
  design: {
    type: String,
    default: 'default',
  },
  color: {
    type: String,
    default: 'default',
  },
  iconOnly: {
    type: Boolean,
    default: false,
  },
})

const defaultDesign = computed(() => props.design === 'default')
const accentedButton = computed(
  () => defaultDesign.value && ['danger', 'primary'].includes(props.color)
)
</script>

<template>
  <!--  <nuxt-link-->
  <!--    v-if="link && link.startsWith('/')"-->
  <!--    class="omorphia__button button-base button-color-base padding-block-sm padding-inline-lg radius-sm"-->
  <!--    :class="{-->
  <!--      'icon-only': props.iconOnly,-->
  <!--      'bg-raised': raised,-->
  <!--      'bg-red': danger,-->
  <!--      'bg-brand': primary,-->
  <!--      'color-accent-contrast': danger || primary,-->
  <!--    }"-->
  <!--    :to="link"-->
  <!--    :target="external ? '_blank' : '_self'"-->
  <!--  >-->
  <!--    <slot />-->
  <!--    <Icon v-if="external && !props.iconOnly" class="external-icon" icon="external" />-->
  <!--    <Icon v-if="!$slots.default" icon="unknown" />-->
  <!--  </nuxt-link>-->
  <a
    v-if="link"
    class="omorphia__button button-base padding-block-sm padding-inline-lg radius-md"
    :class="{
      'standard-button': defaultDesign,
      'icon-only': props.iconOnly,
      'bg-raised': defaultDesign && color === 'raised',
      'bg-red': defaultDesign && color === 'danger',
      'bg-brand': defaultDesign && color === 'primary',
      'color-accent-contrast': accentedButton,
    }"
    :href="link"
    :target="external ? '_blank' : '_self'"
  >
    <slot />
    <ExternalIcon v-if="external && !iconOnly" class="external-icon" />
    <UnknownIcon v-if="!$slots.default" />
  </a>
  <button
    v-else
    class="omorphia__button button-base padding-block-sm padding-inline-lg radius-md"
    :class="{
      'standard-button': defaultDesign,
      'icon-only': props.iconOnly,
      'bg-raised': defaultDesign && color === 'raised',
      'bg-red': defaultDesign && color === 'danger',
      'bg-brand': defaultDesign && color === 'primary',
      'color-accent-contrast': accentedButton,
    }"
    @click="action"
  >
    <slot />
    <UnknownIcon v-if="!$slots.default" />
  </button>
</template>

<style lang="scss" scoped>
:where(button) {
  background: none;
  color: var(--color-base);
}

.omorphia__button {
  display: flex;
  align-items: center;
  cursor: pointer;
  width: fit-content;
  height: fit-content;
  text-decoration: none;

  :deep(svg) {
    width: 1.1rem;
    height: 1.1rem;
    margin-right: 0.5rem;
  }

  :deep(.external-icon) {
    width: 0.75rem;
    height: 0.75rem;
    margin-bottom: auto;
    margin-left: 0.25rem;
    margin-right: 0;
  }

  &.icon-only {
    padding: 0;
    height: 2.25rem;
    width: 2.25rem;

    :deep(svg) {
      min-width: 1.25rem;
      max-width: 1.25rem;
      min-height: 1.25rem;
      max-height: 1.25rem;
      margin: auto;
    }
  }
}
</style>
