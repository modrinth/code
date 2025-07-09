<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    progress: number
    max?: number
    color?: 'brand' | 'green' | 'red' | 'orange' | 'blue' | 'purple' | 'gray'
    waiting?: boolean
  }>(),
  {
    max: 1,
    color: 'brand',
    waiting: false,
  },
)

const colors = {
  brand: {
    fg: 'bg-brand',
    bg: 'bg-brand-highlight',
  },
  green: {
    fg: 'bg-green',
    bg: 'bg-bg-green',
  },
  red: {
    fg: 'bg-red',
    bg: 'bg-bg-red',
  },
  orange: {
    fg: 'bg-orange',
    bg: 'bg-bg-orange',
  },
  blue: {
    fg: 'bg-blue',
    bg: 'bg-bg-blue',
  },
  purple: {
    fg: 'bg-purple',
    bg: 'bg-bg-purple',
  },
  gray: {
    fg: 'bg-gray',
    bg: 'bg-bg-gray',
  },
}

const percent = computed(() => props.progress / props.max)
</script>
<template>
  <div
    class="flex w-full max-w-[15rem] h-1 rounded-full overflow-hidden"
    :class="colors[props.color].bg"
  >
    <div
      class="rounded-full progress-bar"
      :class="[colors[props.color].fg, { 'progress-bar--waiting': waiting }]"
      :style="!waiting ? { width: `${percent * 100}%` } : {}"
    ></div>
  </div>
</template>
<style scoped lang="scss">
.progress-bar {
  transition: width 0.2s ease-in-out;
}

.progress-bar--waiting {
  animation: progress-bar-waiting 1s linear infinite;
  position: relative;
}

@keyframes progress-bar-waiting {
  0% {
    left: -50%;
    width: 20%;
  }
  50% {
    width: 60%;
  }
  100% {
    left: 100%;
    width: 20%;
  }
}
</style>
