<template>
  <Transition name="fade">
    <div v-if="show" class="fixed bottom-4 left-1/2 z-[9999] -translate-x-1/2 transform-gpu">
      <div
        class="flex items-center gap-3 rounded-2xl bg-[var(--color-tooltip-bg)] p-4 text-sm font-bold text-white shadow-lg"
      >
        <UiServersPanelSpinner />
        <span>Uploading {{ currentFile }}</span>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
defineProps<{
  show: boolean;
  currentFile: string;
}>();
</script>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: all 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: translate(-50%, 20px);
}

.fade-enter-to,
.fade-leave-from {
  transform: translate(-50%, 0);
}
</style>
