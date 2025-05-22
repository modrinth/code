<script setup lang="ts">
const emit = defineEmits<{
  (e: 'click'): void
}>()

withDefaults(
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
      :class="[
        'block rounded-lg group-active:scale-95 transition-all border-2 relative',
        highlighted
          ? 'border-brand highlighted-glow'
          : 'border-transparent opacity-75 group-hover:opacity-100',
      ]"
    >
      <span class="block p-[3px] rounded-lg bg-button-bg">
        <span
          class="flex flex-col items-center justify-center aspect-[10/16] w-[60px] min-h-[96px] rounded-[5px] bg-black/10 relative overflow-hidden"
        >
          <div class="mb-1">
            <slot name="icon"></slot>
          </div>
          <span class="text-xs group-hover:text-white" :class="{ 'text-white': highlighted }">
            <slot></slot>
          </span>
        </span>
      </span>
    </span>
  </button>
</template>

<style lang="scss" scoped>
.highlighted-glow::before {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: inherit;
  pointer-events: none;
}

@supports (background-color: color-mix(in srgb, transparent, transparent)) {
  .highlighted-glow::before {
    box-shadow: inset 0 0 2px 2px color-mix(in srgb, var(--color-brand), transparent 10%);
  }
}
</style>
