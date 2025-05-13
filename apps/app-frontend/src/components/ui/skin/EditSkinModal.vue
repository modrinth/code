<script setup lang="ts">
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import { computed, useTemplateRef } from 'vue'

const modal = useTemplateRef('modal')

const previewSkin = computed(() => `https://vzge.me/full/350/ProspectorDev.png?no=ears`)

function show(e: MouseEvent) {
  modal.value?.show(e)
}

function hide() {
  modal.value?.hide()
}

defineExpose({
  show,
  hide,
})
</script>
<template>
  <ModalWrapper ref="modal">
    <template #title>
      <span class="text-lg font-extrabold text-contrast">Edit skin</span>
    </template>
    <div class="grid grid-cols-[auto_1fr] gap-6">
      <div class="flex">
        <img :src="previewSkin" alt="" class="w-auto my-auto h-60 object-contain" />
      </div>
    </div>
  </ModalWrapper>
</template>
