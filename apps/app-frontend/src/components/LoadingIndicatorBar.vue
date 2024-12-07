<script setup>
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { useLoading } from '@/store/state.js'

const props = defineProps({
  throttle: {
    type: Number,
    default: 0,
  },
  duration: {
    type: Number,
    default: 1000,
  },
  height: {
    type: Number,
    default: 2,
  },
  color: {
    type: String,
    default: 'var(--loading-bar-gradient)',
  },
})

const indicator = useLoadingIndicator({
  duration: props.duration,
  throttle: props.throttle,
})

onBeforeUnmount(() => indicator.clear)

const loading = useLoading()

watch(loading, (newValue) => {
  if (newValue.barEnabled) {
    if (newValue.loading) {
      indicator.start()
    } else {
      indicator.finish()
    }
  }
})

function useLoadingIndicator(opts) {
  const progress = ref(0)
  const isLoading = ref(false)
  const step = computed(() => 10000 / opts.duration)

  let _timer = null
  let _throttle = null

  function start() {
    clear()
    progress.value = 0
    if (opts.throttle) {
      _throttle = setTimeout(() => {
        isLoading.value = true
        _startTimer()
      }, opts.throttle)
    } else {
      isLoading.value = true
      _startTimer()
    }
  }

  function finish() {
    progress.value = 100
    _hide()
  }

  function clear() {
    clearInterval(_timer)
    clearTimeout(_throttle)
    _timer = null
    _throttle = null
  }

  function _increase(num) {
    progress.value = Math.min(100, progress.value + num)
  }

  function _hide() {
    clear()
    setTimeout(() => {
      isLoading.value = false
      setTimeout(() => {
        progress.value = 0
      }, 400)
    }, 500)
  }

  function _startTimer() {
    _timer = setInterval(() => {
      _increase(step.value)
    }, 100)
  }

  return { progress, isLoading, start, finish, clear }
}
</script>

<template>
  <div
    class="loading-indicator-bar"
    :style="{
      '--_width': `${indicator.progress.value}%`,
      '--_height': `${indicator.isLoading.value ? props.height : 0}px`,
      '--_opacity': `${indicator.isLoading.value ? 1 : 0}`,
      top: `0`,
      right: `0`,
      left: `${props.offsetWidth}`,
      pointerEvents: 'none',
      width: `var(--_width)`,
      height: `var(--_height)`,
      borderRadius: `var(--_height)`,
      // opacity: `var(--_opacity)`,
      background: `${props.color}`,
      backgroundSize: `${(100 / indicator.progress.value) * 100}% auto`,
      transition: 'width 0.1s ease-in-out, height 0.1s ease-out',
      zIndex: 6,
    }"
  >
    <slot />
  </div>
</template>
<style lang="scss" scoped>
.loading-indicator-bar::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  width: var(--_width);
  bottom: 0;
  background-image: radial-gradient(80% 100% at 20% 0%, var(--color-brand) 0%, transparent 80%);
  opacity: calc(var(--_opacity) * 0.1);
  z-index: 5;
  transition:
    width 0.1s ease-in-out,
    opacity 0.1s ease-out;
}
</style>
