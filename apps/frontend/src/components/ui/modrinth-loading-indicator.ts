import { computed, defineComponent, h, onBeforeUnmount, ref, watch } from 'vue'
import { startLoading, stopLoading, useNuxtApp } from '#imports'

export default defineComponent({
  name: 'ModrinthLoadingIndicator',
  props: {
    throttle: {
      type: Number,
      default: 50,
    },
    duration: {
      type: Number,
      default: 500,
    },
    height: {
      type: Number,
      default: 3,
    },
    color: {
      type: [String, Boolean],
      default:
        'repeating-linear-gradient(to right, var(--color-brand-green) 0%, var(--landing-green-label) 100%)',
    },
  },
  setup(props, { slots }) {
    const indicator = useLoadingIndicator({
      duration: props.duration,
      throttle: props.throttle,
    })

    const nuxtApp = useNuxtApp()
    nuxtApp.hook('page:start', () => {
      startLoading()
      indicator.start()
    })
    nuxtApp.hook('page:finish', () => {
      stopLoading()
      indicator.finish()
    })
    onBeforeUnmount(() => indicator.clear)

    const loading = useLoading()

    watch(loading, (newValue) => {
      if (newValue) {
        indicator.start()
      } else {
        indicator.finish()
      }
    })

    return () =>
      h(
        'div',
        {
          class: 'nuxt-loading-indicator',
          style: {
            position: 'fixed',
            top: 0,
            right: 0,
            left: 0,
            pointerEvents: 'none',
            width: `${indicator.progress.value}%`,
            height: `${props.height}px`,
            opacity: indicator.isLoading.value ? 1 : 0,
            background: props.color || undefined,
            backgroundSize: `${(100 / indicator.progress.value) * 100}% auto`,
            transition: 'width 0.1s, height 0.4s, opacity 0.4s',
            zIndex: 999999,
          },
        },
        slots
      )
  },
})

function useLoadingIndicator(opts: { duration: number; throttle: number }) {
  const progress = ref(0)
  const isLoading = ref(false)
  const step = computed(() => 10000 / opts.duration)

  let _timer: any = null
  let _throttle: any = null

  function start() {
    clear()
    progress.value = 0
    if (opts.throttle && process.client) {
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

  function _increase(num: number) {
    progress.value = Math.min(100, progress.value + num)
  }

  function _hide() {
    clear()
    if (process.client) {
      setTimeout(() => {
        isLoading.value = false
        setTimeout(() => {
          progress.value = 0
        }, 400)
      }, 500)
    }
  }

  function _startTimer() {
    if (process.client) {
      _timer = setInterval(() => {
        _increase(step.value)
      }, 100)
    }
  }

  return {
    progress,
    isLoading,
    start,
    finish,
    clear,
  }
}
