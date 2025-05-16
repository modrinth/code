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
      class="absolute inset-0 bg-button-bg rounded-xl cursor-pointer p-0 border-none hover:brightness-125"
      @mousedown="pressed = true"
      @mouseup="pressed = false"
      @mouseleave="pressed = false"
    ></button>
    <div class="relative w-full h-full flex flex-col items-center justify-center pointer-events-none">
      <div class="mb-2">
        <slot name="icon"></slot>
      </div>
      <span class="text-md text-center px-2">
        <slot></slot>
      </span>
    </div>
  </div>
</template>
