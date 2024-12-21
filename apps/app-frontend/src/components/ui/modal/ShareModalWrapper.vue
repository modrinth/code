<script setup lang="ts">
import { ref } from 'vue'
import { ShareModal } from '@modrinth/ui'
import { show_ads_window, hide_ads_window } from '@/helpers/ads.js'
import { useTheming } from '@/store/theme.js'

const themeStore = useTheming()

defineProps({
  header: {
    type: String,
    default: 'Share',
  },
  shareTitle: {
    type: String,
    default: 'Modrinth',
  },
  shareText: {
    type: String,
    default: null,
  },
  link: {
    type: Boolean,
    default: false,
  },
  openInNewTab: {
    type: Boolean,
    default: true,
  },
})

const modal = ref(null)

defineExpose({
  show: (passedContent) => {
    hide_ads_window()
    modal.value.show(passedContent)
  },
  hide: () => {
    onModalHide()
    modal.value.hide()
  },
})

function onModalHide() {
  show_ads_window()
}
</script>

<template>
  <ShareModal
    ref="modal"
    :header="header"
    :share-title="shareTitle"
    :share-text="shareText"
    :link="link"
    :open-in-new-tab="openInNewTab"
    :on-hide="onModalHide"
    :noblur="!themeStore.advancedRendering"
  />
</template>
