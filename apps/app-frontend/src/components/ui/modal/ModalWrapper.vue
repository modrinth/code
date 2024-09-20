<script setup lang="ts">
import { ref } from 'vue'
import { Modal } from '@modrinth/ui'
import { show_ads_window, hide_ads_window } from '@/helpers/ads.js'
import { useTheming } from '@/store/theme.js'

const themeStore = useTheming()

const props = defineProps({
  header: {
    type: String,
    default: null,
  },
  closable: {
    type: Boolean,
    default: true,
  },
  onHide: {
    type: Function,
    default() {
      return () => {}
    },
  },
})

const modal = ref(null)

defineExpose({
  show: () => {
    hide_ads_window()
    modal.value.show()
  },
  hide: () => {
    onModalHide()
    modal.value.hide()
  },
})

function onModalHide() {
  show_ads_window()
  props.onHide()
}
</script>

<template>
  <Modal ref="modal" :header="header" :noblur="!themeStore.advancedRendering" @hide="onModalHide">
    <slot />
  </Modal>
</template>
