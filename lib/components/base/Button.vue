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
  color: {
    type: String,
    default: 'default',
  },
  iconOnly: {
    type: Boolean,
    default: false,
  },
  large: {
    type: Boolean,
    default: false,
  },
  outline: {
    type: Boolean,
    default: false,
  },
})

const accentedButton = computed(() => ['danger', 'primary'].includes(props.color))
</script>

<template>
  <router-link
    v-if="link"
    class="btn"
    :class="{
      'icon-only': props.iconOnly,
      'btn-large': props.large,
      'btn-raised': color === 'raised',
      'btn-danger': color === 'danger',
      'btn-primary': color === 'primary',
      'btn-secondary': color === 'secondary',
      'btn-highlight': color === 'highlight',
      'btn-outline': props.outline,
      'color-accent-contrast': accentedButton,
    }"
    :to="link"
    :target="external ? '_blank' : '_self'"
  >
    <slot />
    <ExternalIcon v-if="external && !iconOnly" class="external-icon" />
    <UnknownIcon v-if="!$slots.default" />
  </router-link>
  <button
    v-else
    class="btn"
    :class="{
      'icon-only': props.iconOnly,
      'btn-large': props.large,
      'btn-raised': color === 'raised',
      'btn-danger': color === 'danger',
      'btn-primary': color === 'primary',
      'btn-secondary': color === 'secondary',
      'btn-highlight': color === 'highlight',
      'btn-outline': props.outline,
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
</style>
