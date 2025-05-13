<script setup lang="ts">
import { computed } from 'vue'
import type { Cape } from '@/helpers/skins.ts'

const emit = defineEmits<{
  (e: 'select'): void
}>()

const highlighted = computed(() => props.selected ?? props.cape.is_equipped)

const props = withDefaults(
  defineProps<{
    cape: Cape
    selected?: boolean
  }>(),
  {
    selected: undefined,
  },
)
</script>

<template>
  <button v-tooltip="cape.name" class="block border-0 m-0 p-0 bg-transparent group cursor-pointer" :aria-label="cape.name" @click="emit('select')">
    <span
      :class="
        highlighted
          ? `bg-brand highlighted-outer-glow`
          : `bg-button-bg opacity-75 group-hover:opacity-100`
      "
      class="block p-[3px] rounded-lg border-0 group-active:scale-95 transition-all"
    >
      <span
        class="block cursed-cape-shit rounded-[5px]"
        :class="{ 'highlighted-inner-shadow': highlighted }"
      >
        <img :src="cape.texture" alt="" />
      </span>
    </span>
  </button>
</template>
<style lang="scss" scoped>
.cursed-cape-shit {
  aspect-ratio: 10 / 16;
  position: relative;
  overflow: hidden;
  box-sizing: content-box;
}

.cursed-cape-shit img {
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
  box-shadow: inset 0 0 4px 4px rgba(0, 0, 0, 0.2);
  z-index: 2;
}

@supports (background-color: color-mix(in srgb, transparent, transparent)) {
  .highlighted-outer-glow {
    box-shadow: 0 0 4px 2px color-mix(in srgb, var(--color-brand), transparent 70%);
  }
}
</style>
