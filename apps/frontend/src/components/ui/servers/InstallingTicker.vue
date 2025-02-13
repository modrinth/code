<template>
  <div class="ticker-container">
    <div class="ticker-content">
      <div
        v-for="(message, index) in msgs"
        :key="message"
        class="ticker-item text-xs"
        :class="{ active: index === currentIndex % msgs.length }"
      >
        {{ message }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";

const msgs = [
  "Organizing files...",
  "Downloading mods...",
  "Configuring server...",
  "Setting up environment...",
  "Adding Java...",
];

const currentIndex = ref(0);

let intervalId: NodeJS.Timeout | null = null;

onMounted(() => {
  intervalId = setInterval(() => {
    currentIndex.value = (currentIndex.value + 1) % msgs.length;
  }, 3000);
});

onUnmounted(() => {
  if (intervalId) {
    clearInterval(intervalId);
  }
});
</script>

<style scoped>
.ticker-container {
  height: 20px;
  width: 100%;
  position: relative;
}

.ticker-content {
  position: relative;
  width: 100%;
}

.ticker-item {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 20px;
  display: flex;
  align-items: center;
  color: var(--color-secondary-text);
  opacity: 0;
  transform: scale(0.9);
  filter: blur(4px);
  transition: all 0.3s ease-in-out;
}

.ticker-item.active {
  opacity: 1;
  transform: scale(1);
  filter: blur(0);
}
</style>
