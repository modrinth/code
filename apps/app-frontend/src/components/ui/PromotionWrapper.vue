<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { get as getCreds } from '@/helpers/mr_auth.js'
import { handleError } from '@/store/notifications.js'
import { get_user } from '@/helpers/cache.js'
import { ChevronRightIcon } from '@modrinth/assets'
import { init_ads_window, open_ads_link, record_ads_click } from '@/helpers/ads.js'
import { listen } from '@tauri-apps/api/event'

const showAd = ref(true)

defineExpose({
  scroll() {
    updateAdPosition()
  },
})

const creds = await getCreds().catch(handleError)
if (creds && creds.user_id) {
  const user = await get_user(creds.user_id).catch(handleError)

  const MIDAS_BITFLAG = 1 << 0
  if (user && (user.badges & MIDAS_BITFLAG) === MIDAS_BITFLAG) {
    showAd.value = false
  }
}

const adsWrapper = ref(null)
let resizeObserver
let scrollHandler
let intersectionObserver
let mutationObserver
onMounted(() => {
  if (showAd.value) {
    updateAdPosition(true)

    resizeObserver = new ResizeObserver(() => updateAdPosition())
    resizeObserver.observe(adsWrapper.value)

    intersectionObserver = new IntersectionObserver(() => updateAdPosition())
    intersectionObserver.observe(adsWrapper.value)

    mutationObserver = new MutationObserver(() => updateAdPosition())
    mutationObserver.observe(adsWrapper.value, { attributes: true, childList: true, subtree: true })

    // Add scroll event listener
    scrollHandler = () => {
      requestAnimationFrame(() => updateAdPosition())
    }
    window.addEventListener('scroll', scrollHandler, { passive: true })
  }
})

function updateAdPosition(overrideShown = false) {
  if (adsWrapper.value) {
    const rect = adsWrapper.value.getBoundingClientRect()

    let y = rect.top + window.scrollY
    let height = rect.bottom - rect.top

    // Prevent ad from overlaying the app bar
    if (y <= 52) {
      y = 52
      height = rect.bottom - 52

      if (height < 0) {
        height = 0
        y = -1000
      }
    }

    init_ads_window(rect.left + window.scrollX, y, rect.right - rect.left, height, overrideShown)
  }
}

async function openPlusLink() {
  await record_ads_click()
  await open_ads_link('https://modrinth.com/plus', 'https://modrinth.com')
}

const unlisten = await listen('ads-scroll', (event) => {
  if (adsWrapper.value) {
    adsWrapper.value.parentNode.scrollTop += event.payload.scroll
    updateAdPosition()
  }
})

onUnmounted(() => {
  if (resizeObserver) {
    resizeObserver.disconnect()
  }
  if (intersectionObserver) {
    intersectionObserver.disconnect()
  }
  if (mutationObserver) {
    mutationObserver.disconnect()
  }
  if (scrollHandler) {
    window.removeEventListener('scroll', scrollHandler)
  }

  unlisten()
})
</script>

<template>
  <div
    v-if="showAd"
    ref="adsWrapper"
    class="ad-parent relative mb-3 flex w-full justify-center rounded-2xl bg-bg-raised cursor-pointer"
  >
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
