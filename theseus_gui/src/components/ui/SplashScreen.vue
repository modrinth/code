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

  .splash-text {
    margin-top: 2rem;
    font-size: 1rem;
    font-weight: 500;
    text-align: center;
    width: 15rem;
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
