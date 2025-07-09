<script setup>
import { ExternalIcon, UnknownIcon } from '@modrinth/assets'
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
  transparent: {
    type: Boolean,
    default: false,
  },
  hoverFilled: {
    type: Boolean,
    default: false,
  },
  hoverFilledOnly: {
    type: Boolean,
    default: false,
  },
})

const accentedButton = computed(() =>
  ['danger', 'primary', 'red', 'orange', 'green', 'blue', 'purple', 'gray'].includes(props.color),
)

const classes = computed(() => {
  const color = props.color
  return {
    'icon-only': props.iconOnly,
    'btn-large': props.large,
    'btn-danger': color === 'danger',
    'btn-primary': color === 'primary',
    'btn-secondary': color === 'secondary',
    'btn-highlight': color === 'highlight',
    'btn-red': color === 'red',
    'btn-orange': color === 'orange',
    'btn-green': color === 'green',
    'btn-blue': color === 'blue',
    'btn-purple': color === 'purple',
    'btn-gray': color === 'gray',
    'btn-transparent': props.transparent,
    'btn-hover-filled': props.hoverFilled,
    'btn-hover-filled-only': props.hoverFilledOnly,
    'btn-outline': props.outline,
    'color-accent-contrast': accentedButton,
  }
})
</script>

<template>
  <router-link
    v-if="link && link.startsWith('/')"
    class="btn"
    :class="classes"
    :to="link"
    :target="external ? '_blank' : '_self'"
    @click="
      (event) => {
        if (action) {
          action(event)
        }
      }
    "
  >
    <slot />
    <ExternalIcon v-if="external && !iconOnly" class="external-icon" />
    <UnknownIcon v-if="!$slots.default" />
  </router-link>
  <a
    v-else-if="link"
    class="btn"
    :class="classes"
    :href="link"
    :target="external ? '_blank' : '_self'"
    @click="
      (event) => {
        if (action) {
          action(event)
        }
      }
    "
  >
    <slot />
    <ExternalIcon v-if="external && !iconOnly" class="external-icon" />
    <UnknownIcon v-if="!$slots.default" />
  </a>
  <button v-else class="btn" :class="classes" @click="action">
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
