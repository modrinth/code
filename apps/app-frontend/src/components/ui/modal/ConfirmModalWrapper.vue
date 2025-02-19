<script setup lang="ts">
import { ref } from 'vue'
import { ConfirmModal } from '@modrinth/ui'
import { show_ads_window, hide_ads_window } from '@/helpers/ads.js'
import { useTheming } from '@/store/theme.js'

const themeStore = useTheming()

const props = defineProps({
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
  proceedIcon: {
    type: Object,
    default: undefined,
  },
  proceedLabel: {
    type: String,
    default: 'Proceed',
  },
  danger: {
    type: Boolean,
    default: true,
  },
  showAdOnClose: {
    type: Boolean,
    default: true,
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
  if (props.showAdOnClose) {
    show_ads_window()
  }
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
    :proceed-icon="proceedIcon"
    :proceed-label="proceedLabel"
    :on-hide="onModalHide"
    :noblur="!themeStore.advancedRendering"
    :danger="danger"
    @proceed="proceed"
  />
</template>
