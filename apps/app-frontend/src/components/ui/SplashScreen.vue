<template>
  <Transition name="splash-fade" @after-leave="onAfterLeave">
    <div v-if="!doneLoading" class="splash-screen dark">
      <div class="app-logo-wrapper" data-tauri-drag-region>
        <img :src="icarusLogo" class="app-logo" />
        <ProgressBar
          class="loading-bar"
          :progress="Math.min(loadingProgress, 100)"
        />
        <span v-if="message">{{ message }}</span>
      </div>
      <div class="gradient-bg" data-tauri-drag-region></div>
      <div class="cube-bg"></div>
      <div class="base-bg"></div>
    </div>
  </Transition>
</template>

<script setup>
import { injectLoadingState } from '@icarus/ui'
import { ref, watch } from 'vue'

import icarusLogo from '@/assets/icarus_logo.png'
import ProgressBar from '@/components/ui/ProgressBar.vue'
import { loading_listener } from '@/helpers/events.js'

const doneLoading = ref(false)
const loadingProgress = ref(0)
const message = ref()

const MIN_DISPLAY_MS = 500
const mountedAt = Date.now()

const loading = injectLoadingState()

function onAfterLeave() {
  loading.setEnabled(true)
}

watch(
  [loading.barEnabled, loading.pending],
  ([barEnabled, pending]) => {
    if (barEnabled) {
      return
    }

    if (pending) {
      loadingProgress.value = 0
      fakeLoadingIncrease()
      return
    }

    const elapsed = Date.now() - mountedAt
    const delay = Math.max(0, MIN_DISPLAY_MS - elapsed)

    setTimeout(() => {
      if (loading.pending.value) {
        return
      }
      doneLoading.value = true
    }, delay)
  },
  { immediate: true },
)

function fakeLoadingIncrease() {
  if (loadingProgress.value < 95) {
    setTimeout(() => {
      loadingProgress.value += 1
      fakeLoadingIncrease()
    }, 5)
  }
}

loading_listener(async (e) => {
  if (e.event.type === 'directory_move') {
    loadingProgress.value = 100 * (e.fraction ?? 1)
    message.value = 'Updating app directory...'
  } else if (e.event.type === 'checking_for_updates') {
    loadingProgress.value = 100 * (e.fraction ?? 1)
    message.value = 'Checking for updates...'
  }
})
</script>

<style scoped lang="scss">
.splash-screen {
  position: fixed;
  inset: 0;
  z-index: 10000;
}

.splash-fade-leave-active {
  transition: opacity 0.3s ease-in-out;
}

.splash-fade-leave-to {
  opacity: 0;
}

.app-logo-wrapper {
  position: absolute;
  height: 100vh;
  width: 100%;

  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;

  gap: 1rem;

  z-index: 9998;
}

.app-logo {
  height: 6rem;
  width: 6rem;
  object-fit: contain;
}

.loading-bar {
  max-width: 20rem;
}

.gradient-bg {
  position: absolute;
  height: 100vh;
  width: 100vw;
  background:
    linear-gradient(
      180deg,
      rgba(142, 50, 243, 0.275) 0%,
      rgba(17, 35, 43, 0.5) 97.29%
    ),
    linear-gradient(0deg, rgba(22, 24, 28, 0.64), rgba(22, 24, 28, 0.64));
  z-index: 9997;
}

.cube-bg {
  position: absolute;

  left: 50%;
  top: 50%;
  transform: translate(-50%, -50%);

  width: 1920vw;
  height: 1080vh;
  opacity: 0.8;
  background: #16181c url('@/assets/loading/tax-fluid.png') center no-repeat;
  background-size: contain;

  z-index: 9996;
}

.base-bg {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: var(--color-bg);
  z-index: 9995;
}
</style>
