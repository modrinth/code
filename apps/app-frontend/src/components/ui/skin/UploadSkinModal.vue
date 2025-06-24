<template>
  <ModalWrapper ref="modal" @on-hide="hide(true)">
    <template #title>
      <span class="text-lg font-extrabold text-contrast"> Upload skin texture </span>
    </template>
    <div class="relative">
      <div
        class="border-2 border-dashed border-highlight-gray rounded-xl h-[173px] flex flex-col items-center justify-center p-8 cursor-pointer bg-button-bg hover:bg-button-hover transition-colors relative"
        @click="triggerFileInput"
      >
        <p class="mx-auto mb-0 text-primary font-bold text-lg text-center flex items-center gap-2">
          <UploadIcon /> Select skin texture file
        </p>
        <p class="mx-auto mt-0 text-secondary text-sm text-center">
          Drag and drop or click here to browse
        </p>
        <input
          ref="fileInput"
          type="file"
          accept="image/png"
          class="hidden"
          @change="handleInputFileChange"
        />
      </div>
    </div>
  </ModalWrapper>
</template>

<script setup lang="ts">
import { ref, onBeforeUnmount, watch } from 'vue'
import { UploadIcon } from '@modrinth/assets'
import { useNotifications } from '@/store/state'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { readFile } from '@tauri-apps/plugin-fs'
import ModalWrapper from "@/components/ui/modal/ModalWrapper.vue";

const notifications = useNotifications()

const modal = ref()
const fileInput = ref<HTMLInputElement>()
const unlisten = ref<() => void>()
const modalVisible = ref(false)

const emit = defineEmits<{
  (e: 'uploaded', file: File): void
  (e: 'canceled'): void
}>()

function show(e?: MouseEvent) {
  modal.value?.show(e)
  modalVisible.value = true
  setupDragDropListener()
}

function hide(emitCanceled = false) {
  modal.value?.hide()
  modalVisible.value = false
  cleanupDragDropListener()
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

async function validateImageDimensions(blob: Blob): Promise<boolean> {
  return new Promise((resolve) => {
    const img = new Image()
    img.onload = () => {
      URL.revokeObjectURL(img.src)
      resolve(img.width === 64 && (img.height === 64 || img.height === 32))
    }
    img.onerror = () => {
      URL.revokeObjectURL(img.src)
      resolve(false)
    }
    img.src = URL.createObjectURL(blob)
  })
}

async function handleInputFileChange(e: Event) {
  const files = (e.target as HTMLInputElement).files
  if (!files || files.length === 0) {
    return
  }
  const file = files[0]
  await processFile(file)
}

async function setupDragDropListener() {
  try {
    if (modalVisible.value) {
      await cleanupDragDropListener()
      unlisten.value = await getCurrentWebview().onDragDropEvent(async (event) => {
        if (event.payload.type !== 'drop') {
          return
        }

        if (!event.payload.paths || event.payload.paths.length === 0) {
          return
        }

        const filePath = event.payload.paths[0]

        try {
          const data = await readFile(filePath);

          const fileName = filePath.split('/').pop() || filePath.split('\\').pop() || 'skin.png'
          const fileBlob = new Blob([data], { type: 'image/png' })
          const file = new File([fileBlob], fileName, { type: 'image/png' })

          await processFile(file)
        } catch (error) {
          console.error(error);
          notifications.addNotification({
            title: 'Error processing file',
            text: 'Failed to read the dropped file.',
            type: 'error',
          })
        }
      })
    }
  } catch (error) {
    console.error('Failed to set up drag and drop listener:', error)
  }
}

async function cleanupDragDropListener() {
  if (unlisten.value) {
    unlisten.value()
    unlisten.value = undefined
  }
}

async function processFile(file: File) {
  if (!file.name.toLowerCase().endsWith('.png') && file.type !== 'image/png') {
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
      text: 'Only 64x64 and 64x32 PNG files are accepted.',
      type: 'error',
    })
    return
  }

  emit('uploaded', file)
  hide()
}

watch(modalVisible, (isVisible) => {
  if (isVisible) {
    setupDragDropListener()
  } else {
    cleanupDragDropListener()
  }
})

onBeforeUnmount(() => {
  cleanupDragDropListener()
})

defineExpose({ show, hide })
</script>
