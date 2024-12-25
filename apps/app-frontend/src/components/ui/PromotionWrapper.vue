<script setup>
import { ref, onMounted } from 'vue'
import { init_ads_window } from '@/helpers/ads.js'

const adsWrapper = ref(null)

let devicePixelRatioWatcher = null

function initDevicePixelRatioWatcher() {
  if (devicePixelRatioWatcher) {
    devicePixelRatioWatcher.removeEventListener('change', updateAdPosition)
  }

  devicePixelRatioWatcher = window.matchMedia(`(resolution: ${window.devicePixelRatio}dppx)`)
  devicePixelRatioWatcher.addEventListener('change', updateAdPosition)
}

onMounted(() => {
  updateAdPosition()

  window.addEventListener('resize', updateAdPosition)
  initDevicePixelRatioWatcher()
})

function updateAdPosition() {
  if (adsWrapper.value) {
    init_ads_window()
    initDevicePixelRatioWatcher()
  }
}
</script>

<template>
  <div ref="adsWrapper" class="ad-parent relative flex w-full justify-center cursor-pointer bg-bg">
    <div class="flex max-h-[250px] min-h-[250px] min-w-[300px] max-w-[300px] flex-col gap-4 p-6">
      <p class="m-0 text-2xl font-bold text-contrast">75% of ad revenue goes to creators</p>
    </div>
  </div>
</template>
