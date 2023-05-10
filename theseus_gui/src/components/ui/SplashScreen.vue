<template>
  <div class="page-loading" :class="{ 'app-loading': appLoading }">
    <AnimatedLogo class="initializing-icon" />
  </div>
</template>

<script setup>
import { AnimatedLogo } from 'omorphia'

defineProps({
  appLoading: Boolean,
})
</script>

<style scoped lang="scss">
.page-loading {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  height: 100%;
  width: 100%;

  &.app-loading {
    background-color: #16181c;
    height: 100vh;
  }
}

.initializing-icon {
  width: 12rem;
  height: 12rem;

  :deep(svg),
  svg {
    width: 12rem;
    height: 12rem;
  }
}
</style>
