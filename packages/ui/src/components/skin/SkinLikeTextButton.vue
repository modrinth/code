<script setup lang="ts">
import { ref } from 'vue'
const pressed = ref(false)
</script>

<template>
  <div
    class="relative border-2 border-transparent rounded-xl overflow-hidden"
    :class="{ 'scale-95': pressed }"
  >
    <button
      class="absolute inset-0 skin-like-text-bg rounded-xl cursor-pointer p-0 border-none"
      @mousedown="pressed = true"
      @mouseup="pressed = false"
      @mouseleave="pressed = false"
    ></button>
    <div
      class="relative w-full h-full flex flex-col items-center justify-center pointer-events-none"
    >
      <div class="mb-2">
        <slot name="icon"></slot>
      </div>
      <span class="text-md text-center px-2">
        <slot></slot>
      </span>
    </div>
  </div>
</template>

<style scoped lang="scss">
.skin-like-text-bg {
  background: linear-gradient(180deg, #3a3d47 0%, #33363d 100%);
  transition:
    filter 200ms ease-in-out,
    box-shadow 200ms ease-in-out;
}
.skin-like-text-bg:hover {
  filter: brightness(var(--hover-brightness));
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}
</style>
