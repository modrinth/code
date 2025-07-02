<script setup lang="ts">
import { computed } from 'vue'

const emit = defineEmits<{
  (e: 'select'): void
}>()

const props = withDefaults(
  defineProps<{
    name: string | undefined
    id: string
    texture: string
    isEquipped?: boolean
    selected?: boolean
    faded?: boolean
  }>(),
  {
    isEquipped: false,
    selected: undefined,
    faded: false,
  },
)

console.log(props)

const highlighted = computed(() => props.selected ?? props.isEquipped)
</script>

<template>
  <button
    v-tooltip="name"
    class="block border-0 m-0 p-0 bg-transparent group cursor-pointer"
    :aria-label="name"
    @click="emit('select')"
  >
    <span
      :class="
        highlighted
          ? `bg-brand highlighted-outer-glow`
          : `bg-button-bg brightness-95 group-hover:brightness-100`
      "
      class="relative block p-[3px] rounded-lg border-0 group-active:scale-95 transition-all"
    >
      <span
        class="block magical-cape-transform rounded-[5px]"
        :class="{
          'highlighted-inner-shadow': highlighted,
          'brightness-[0.3] contrast-[0.8]': faded,
        }"
      >
        <img :src="texture" alt="" />
      </span>
      <span
        v-if="$slots.default || $slots.icon"
        class="p-4 absolute inset-0 flex items-center justify-center text-primary font-medium"
      >
        <span class="mb-1">
          <slot name="icon"></slot>
        </span>
        <span class="text-xs">
          <slot></slot>
        </span>
      </span>
    </span>
  </button>
</template>
<style lang="scss" scoped>
.magical-cape-transform {
  aspect-ratio: 10 / 16;
  position: relative;
  overflow: hidden;
  box-sizing: content-box;
  width: 60px;
  min-height: 96px;
}

.magical-cape-transform img {
  position: absolute;
  object-fit: cover;
  image-rendering: pixelated;

  // scales image up so that the target area of the texture (10x16) is 100% of the container
  width: calc(64 / 10 * 100%);
  height: calc(32 / 16 * 100%);

  // offsets the image so that the target area is in the container
  left: calc(1 / 10 * -100%);
  top: calc(1 / 16 * -100%);

  // scale the image up a little bit to avoid edges from the surrounding texture due to rounding
  scale: 1.01;
  transform-origin: calc(10 / 2 / 64 * 100%) calc(16 / 2 / 32 * 100%);
}

.highlighted-inner-shadow::before {
  content: '';
  position: absolute;
  inset: 0;
  box-shadow: inset 0 0 4px 4px rgba(0, 0, 0, 0.4);
  z-index: 2;
}

@supports (background-color: color-mix(in srgb, transparent, transparent)) {
  .highlighted-glow::before {
    box-shadow: inset 0 0 2px 4px color-mix(in srgb, var(--color-brand), transparent 10%);
  }
}
</style>
