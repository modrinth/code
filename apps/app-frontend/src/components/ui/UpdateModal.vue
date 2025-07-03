<template>
  <ModalWrapper
    ref="modal"
    :header="header"
  >
  </ModalWrapper>
</template>

<script setup lang="ts">
import ModalWrapper from "@/components/ui/modal/ModalWrapper.vue";
import {defineMessage, useVIntl} from "@vintl/vintl";
import {computed, useTemplateRef, ref} from 'vue';

const {formatMessage} = useVIntl();

const headerMessage = defineMessage({
  id: 'app.update.modal-header',
  defaultMessage: `An update is available!`,
})
const header = computed(() => formatMessage(headerMessage))

const modal = useTemplateRef('modal')
const isOpen = ref(false)
function show() {
  modal.value.show()
  isOpen.value = true
}
function hide() {
  modal.value.hide()
  isOpen.value = false
}

defineExpose({ show, isOpen })
</script>

<style scoped lang="scss">
</style>
