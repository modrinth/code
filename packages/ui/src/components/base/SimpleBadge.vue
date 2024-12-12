<template>
  <span class="inline-flex items-center gap-1 font-semibold text-secondary">
    <component :is="icon" v-if="icon" :aria-hidden="true" class="shrink-0" />
    {{ formattedName }}
  </span>
</template>

<script setup lang="ts">
import type { Component } from 'vue'

defineProps<{
  icon?: Component
  formattedName: string
  color?: 'brand' | 'green' | 'blue' | 'purple' | 'orange' | 'red'
}>()
</script>
