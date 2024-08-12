<template>
  <div
    :class="`flex items-center gap-2 rounded-full px-2 py-1 ${
      isOnline ? 'bg-green-400/10 text-green-500' : 'bg-button-border'
    }`"
  >
    <span class="text-sm font-semibold">
      {{ isOnline ? "Online" : "Offline" }}
    </span>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  isOnline: boolean;
}>();
</script>
