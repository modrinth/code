<template>
  <div class="progress-bar">
    <div class="progress-bar__fill" :style="{ width: `${progress}%` }"></div>
  </div>
</template>

<script setup>
defineProps({
  progress: {
    type: Number,
    required: true,
    validator(value) {
      return value >= 0 && value <= 100
    },
  },
})
</script>

<style scoped>
.progress-bar {
  width: 100%;
  height: 0.5rem;
  background-color: var(--color-button-bg);
  border-radius: var(--radius-lg);
  overflow: hidden;
}

.progress-bar__fill {
  height: 100%;
  background-color: var(--color-brand);
  transition: width 0.3s;
}
</style>
