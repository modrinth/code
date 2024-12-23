<script setup>
import { ref, onMounted } from 'vue'
import { ChevronRightIcon } from '@modrinth/assets'
import { init_ads_window, open_ads_link, record_ads_click } from '@/helpers/ads.js'

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

async function openPlusLink() {
  await record_ads_click()
  await open_ads_link('https://modrinth.com/plus', 'https://modrinth.com')
}
</script>

<template>
  <div ref="adsWrapper" class="ad-parent relative flex w-full justify-center cursor-pointer bg-bg">
    <div class="flex max-h-[250px] min-h-[250px] min-w-[300px] max-w-[300px] flex-col gap-4 p-6">
      <p class="m-0 text-2xl font-bold text-contrast">75% of ad revenue goes to creators</p>
      <button
        class="mt-auto items-center gap-1 text-purple hover:underline bg-transparent border-none text-left cursor-pointer outline-none"
        @click="openPlusLink"
      >
        <span>
          Support creators and Modrinth ad-free with
          <span class="font-bold">Modrinth+</span>
        </span>
        <ChevronRightIcon class="relative top-[3px] h-5 w-5" />
      </button>
    </div>
  </div>
</template>
