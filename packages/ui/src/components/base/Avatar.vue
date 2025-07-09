<template>
  <img
    v-if="src"
    ref="img"
    class="`experimental-styles-within avatar"
    :style="`--_size: ${cssSize}`"
    :class="{
      circle: circle,
      'no-shadow': noShadow,
      raised: raised,
      pixelated: pixelated,
    }"
    :src="src"
    :alt="alt"
    :loading="loading"
    @load="updatePixelated"
  />
  <svg
    v-else
    class="`experimental-styles-within avatar"
    :style="`--_size: ${cssSize}${tint ? `;--_tint:oklch(50% 75% ${tint})` : ''}`"
    :class="{
      tint: tint,
      circle: circle,
      'no-shadow': noShadow,
      raised: raised,
    }"
    xml:space="preserve"
    fill-rule="evenodd"
    stroke-linecap="round"
    stroke-linejoin="round"
    stroke-miterlimit="1.5"
    clip-rule="evenodd"
    viewBox="0 0 104 104"
    aria-hidden="true"
  >
    <path fill="none" d="M0 0h103.4v103.4H0z" />
    <path
      fill="none"
      stroke="#9a9a9a"
      stroke-width="5"
      d="M51.7 92.5V51.7L16.4 31.3l35.3 20.4L87 31.3 51.7 11 16.4 31.3v40.8l35.3 20.4L87 72V31.3L51.7 11"
    />
  </svg>
</template>

<script setup>
import { computed, ref } from 'vue'

const pixelated = ref(false)
const img = ref(null)

const props = defineProps({
  src: {
    type: String,
    default: null,
  },
  alt: {
    type: String,
    default: '',
  },
  size: {
    type: String,
    default: '2rem',
  },
  circle: {
    type: Boolean,
    default: false,
  },
  noShadow: {
    type: Boolean,
    default: false,
  },
  loading: {
    type: String,
    default: 'eager',
  },
  raised: {
    type: Boolean,
    default: false,
  },
  tintBy: {
    type: String,
    default: null,
  },
})

const LEGACY_PRESETS = {
  xxs: '1.25rem',
  xs: '2.5rem',
  sm: '3rem',
  md: '6rem',
  lg: '9rem',
}

const cssSize = computed(() => LEGACY_PRESETS[props.size] ?? props.size)

function updatePixelated() {
  if (img.value && img.value.naturalWidth && img.value.naturalWidth < 32) {
    pixelated.value = true
  } else {
    pixelated.value = false
  }
}

const tint = computed(() => {
  if (props.tintBy) {
    return hash(props.tintBy) % 360
  } else {
    return null
  }
})

function hash(str) {
  let hash = 0
  for (let i = 0, len = str.length; i < len; i++) {
    const chr = str.charCodeAt(i)
    hash = (hash << 5) - hash + chr
    hash |= 0
  }
  return hash
}
</script>

<style lang="scss" scoped>
.avatar {
  @apply min-w-[--_size] min-h-[--_size] w-[--_size] h-[--_size];
  --_size: 2rem;

  border: 1px solid var(--color-button-border);
  background-color: var(--color-button-bg);
  object-fit: contain;
  border-radius: calc(16 / 96 * var(--_size));
  position: relative;

  &.circle {
    border-radius: 50%;
  }

  &:not(.no-shadow) {
    box-shadow: var(--shadow-card);
  }

  &.no-shadow {
    box-shadow: none;
  }

  &.pixelated {
    image-rendering: pixelated;
  }

  &.raised {
    background-color: var(--color-raised-bg);
  }

  &.tint {
    background-color: color-mix(in oklch, var(--color-button-bg) 100%, var(--_tint) 5%);
  }
}
</style>
