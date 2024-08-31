<script setup lang="ts">
import { ref } from 'vue'
import { ConfirmModal } from '@modrinth/ui'
import { show_ads_window, hide_ads_window } from '@/helpers/ads.js'
import { useTheming } from '@/store/theme.js'

const themeStore = useTheming()

defineProps({
  confirmationText: {
    type: String,
    default: '',
  },
  hasToType: {
    type: Boolean,
    default: false,
  },
  title: {
    type: String,
    default: 'No title defined',
    required: true,
  },
  description: {
    type: String,
    default: 'No description defined',
    required: true,
  },
  proceedLabel: {
    type: String,
    default: 'Proceed',
  },
})

const emit = defineEmits(['proceed'])
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
}

function proceed() {
  emit('proceed')
}
</script>

<template>
  <ConfirmModal
    ref="modal"
    :confirmation-text="confirmationText"
    :has-to-type="hasToType"
    :title="title"
    :description="description"
    :proceed-label="proceedLabel"
    :on-hide="onModalHide"
    :noblur="!themeStore.advancedRendering"
    @proceed="proceed"
  />
</template>
