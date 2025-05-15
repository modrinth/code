<script setup lang="ts">
const emit = defineEmits<{
  (e: 'click'): void
}>()

const props = withDefaults(
  defineProps<{
    tooltip: string
    highlighted?: boolean
  }>(),
  {
    highlighted: false,
  },
)
</script>

<template>
  <button
    v-tooltip="tooltip"
    class="block border-0 m-0 p-0 bg-transparent group cursor-pointer"
    :aria-label="tooltip"
    @click="emit('click')"
  >
    <span
      :class="
        highlighted
          ? `bg-brand highlighted-outer-glow`
          : `bg-button-bg opacity-75 group-hover:opacity-100`
      "
      class="block p-[3px] rounded-lg border-0 group-active:scale-95 transition-all"
    >
      <span
        class="flex flex-col items-center justify-center aspect-[10/16] w-[60px] min-h-[96px] rounded-[5px] bg-black/10 relative overflow-hidden"
        :class="{ 'highlighted-inner-shadow': highlighted }"
      >
        <div class="mb-1">
          <slot name="icon"></slot>
        </div>

        <span class="text-xs text-white/80 group-hover:text-white">
          <slot name="default"></slot>
        </span>
      </span>
    </span>
  </button>
</template>

<style lang="scss" scoped>
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
