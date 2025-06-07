<template>
  <div class="flex items-center gap-2 w-fit px-3 py-1 bg-button-bg rounded-full text-sm">
    <component :is="icon" v-if="icon" class="w-4 h-4" />
    <span class="whitespace-nowrap">
      {{ text }}
    </span>
  </div>
</template>

<script setup lang="ts">
import type { Component } from 'vue'

defineProps<{
  text: string
  icon?: Component
}>()
</script>
