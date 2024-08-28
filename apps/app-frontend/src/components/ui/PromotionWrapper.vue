<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { get as getCreds } from '@/helpers/mr_auth.js'
import { handleError } from '@/store/notifications.js'
import { get_user } from '@/helpers/cache.js'
import { ChevronRightIcon } from '@modrinth/assets'
import { init_ads_window } from '@/helpers/ads.js'

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
    updateAdPosition()

    resizeObserver = new ResizeObserver(updateAdPosition)
    resizeObserver.observe(adsWrapper.value)

    intersectionObserver = new IntersectionObserver(updateAdPosition)
    intersectionObserver.observe(adsWrapper.value)

    mutationObserver = new MutationObserver(updateAdPosition)
    mutationObserver.observe(adsWrapper.value, { attributes: true, childList: true, subtree: true })

    // Add scroll event listener
    scrollHandler = () => {
      requestAnimationFrame(updateAdPosition)
    }
    window.addEventListener('scroll', scrollHandler, { passive: true })
  }
})

function updateAdPosition() {
  if (adsWrapper.value) {
    const rect = adsWrapper.value.getBoundingClientRect()

    init_ads_window(
      rect.left + window.scrollX,
      rect.top + window.scrollY,
      rect.right - rect.left,
      rect.bottom - rect.top,
    )
  }
}

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
})
</script>

<template>
  <div
    v-if="showAd"
    ref="adsWrapper"
    class="ad-parent relative mb-3 flex w-full justify-center rounded-2xl bg-bg-raised"
  >
    <div class="flex max-h-[250px] min-h-[250px] min-w-[300px] max-w-[300px] flex-col gap-4 p-6">
      <p class="m-0 text-2xl font-bold text-contrast">90% of ad revenue goes to creators</p>
      <a
        href="https://modrinth.com/plus"
        class="mt-auto items-center gap-1 text-purple hover:underline"
      >
        <span>
          Support creators and Modrinth ad-free with
          <span class="font-bold">Modrinth+</span>
        </span>
        <ChevronRightIcon class="relative top-[3px] h-5 w-5" />
      </a>
    </div>
  </div>
</template>
