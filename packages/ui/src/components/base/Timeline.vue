<script setup lang="ts">
withDefaults(
  defineProps<{
    fadeOutStart?: boolean
    fadeOutEnd?: boolean
  }>(),
  {
    fadeOutStart: false,
    fadeOutEnd: false,
  },
)
</script>
<template>
  <div class="relative flex flex-col gap-4 pb-6 isolate">
    <div class="absolute flex h-full w-4 justify-center">
      <div
        class="timeline-indicator"
        :class="{ 'fade-out-start': fadeOutStart, 'fade-out-end': fadeOutEnd }"
      />
    </div>
    <slot />
  </div>
</template>
<style lang="scss" scoped>
.timeline-indicator {
  background-image: linear-gradient(
    to bottom,
    var(--timeline-line-color, var(--color-raised-bg)) 66%,
    rgba(255, 255, 255, 0) 0%
  );
  background-size: 100% 30px;
  background-repeat: repeat-y;
  margin-top: 1rem;

  height: calc(100% - 1rem);
  width: 4px;
  z-index: -1;

  &.fade-out-start {
    mask-image: linear-gradient(to top, black calc(100% - 15rem), transparent 100%);
  }

  &.fade-out-end {
    mask-image: linear-gradient(to bottom, black calc(100% - 15rem), transparent 100%);
  }

  &.fade-out-start.fade-out-end {
    mask-image: linear-gradient(
      to bottom,
      transparent 0%,
      black,
      black calc(100% - 8rem),
      transparent 100%
    );
  }
}
</style>
