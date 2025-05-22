<template>
  <NewModal ref="modal" @on-hide="hide(true)">
    <template #title>
      <span class="text-lg font-extrabold text-contrast"> Upload skin texture </span>
    </template>
    <div class="relative">
      <div
        class="border-2 border-dashed border-highlight-gray rounded-xl h-[173px] flex flex-col items-center justify-center p-8 cursor-pointer bg-button-bg hover:bg-button-hover transition-colors relative"
        @click="triggerFileInput"
        @drop.prevent="handleFileOperation"
        @dragover.prevent
      >
        <p class="mx-auto mb-0 text-primary text-xl text-center flex items-center gap-2">
          <UploadIcon /> Select skin texture file
        </p>
        <p class="mx-auto mt-0 text-secondary text-sm text-center">
          Drag and drop or click here to browse
        </p>
        <p class="mx-auto mt-0 text-secondary text-xs text-center">
          Only 64x64 PNG files are accepted
        </p>
        <input
          ref="fileInput"
          type="file"
          accept="image/png"
          class="hidden"
          @change="handleFileOperation"
        />
      </div>
    </div>
  </NewModal>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { UploadIcon } from '@modrinth/assets'
import { useNotifications } from '@/store/state'
import { NewModal } from '@modrinth/ui'

const notifications = useNotifications()

const modal = ref()
const fileInput = ref<HTMLInputElement>()

const emit = defineEmits<{
  (e: 'uploaded', file: File): void
  (e: 'canceled'): void
}>()

function show(e?: MouseEvent) {
  modal.value?.show(e)
}
function hide(emitCanceled = false) {
  modal.value?.hide()
  resetState()
  if (emitCanceled) {
    emit('canceled')
  }
}
function resetState() {
  if (fileInput.value) fileInput.value.value = ''
}
function triggerFileInput() {
  fileInput.value?.click()
}

async function validateImageDimensions(file: File): Promise<boolean> {
  return new Promise((resolve) => {
    const img = new Image()
    img.onload = () => {
      URL.revokeObjectURL(img.src)
      resolve(img.width === 64 && img.height === 64)
    }
    img.onerror = () => {
      URL.revokeObjectURL(img.src)
      resolve(false)
    }
    img.src = URL.createObjectURL(file)
  })
}

async function handleFileOperation(e: Event | DragEvent) {
  // Get files from either drag event or file input
  const files = (e as DragEvent).dataTransfer?.files || (e.target as HTMLInputElement).files
  if (!files || files.length === 0) {
    return
  }
  const file = files[0]
  if (file.type !== 'image/png') {
    notifications.addNotification({
      title: 'Invalid file type.',
      text: 'Only PNG files are accepted.',
      type: 'error',
    })
    return
  }
  const isValidDimensions = await validateImageDimensions(file)
  if (!isValidDimensions) {
    notifications.addNotification({
      title: 'Invalid dimensions.',
      text: 'Only 64x64 PNG files are accepted.',
      type: 'error',
    })
    return
  }
  emit('uploaded', file)
  hide()
}

defineExpose({ show, hide })
</script>
