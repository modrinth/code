<template>
  <div
      ref="wrapperContainer"
      class="group rounded-lg relative overflow-hidden shadow-md w-full text-contrast"
      @mouseenter="isHovered = true"
      @mouseleave="isHovered = false"
  >
    <div
        v-if="loaded"
        class="absolute top-2 right-2 flex gap-1 transition-opacity duration-200 z-10"
        :class="{ 'opacity-0': !isHovered, 'opacity-100': isHovered }"
    >
      <Button v-tooltip="'Copy'" icon-only title="Copy" @click="copyImageToClipboard">
        <ClipboardCopyIcon />
      </Button>
      <Button v-tooltip="'View in folder'" icon-only title="View in folder" @click="viewInFolder">
        <ExternalIcon />
      </Button>
      <Button v-tooltip="'Delete'" color="red" icon-only title="Delete" @click="deleteScreenshot">
        <TrashIcon />
      </Button>
    </div>

    <div class="aspect-video bg-bg-raised overflow-hidden">
      <div v-if="!loaded" class="absolute inset-0 skeleton"></div>
      <img
          v-else
          :alt="getScreenshotFileName(screenshot.path)"
          :src="blobUrl"
          class="w-full h-full object-cover transition-opacity duration-700"
          :class="{ 'opacity-0': !loaded, 'opacity-100': loaded }"
          @load="onLoad"
          @click="
          imagePreviewModal.show(
            blobUrl,
            getScreenshotFileName(screenshot.path),
            {
              ...screenshot,
              title: getScreenshotFileName(screenshot.path),
              description: `Taken on ${dayjs(screenshot.creation_date).format('MMMM Do, YYYY')}`,
            }
          )
        "
      />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { ClipboardCopyIcon, TrashIcon, ExternalIcon } from '@modrinth/assets'
import type { ImagePreviewModal } from '@modrinth/ui'
import { Button } from '@modrinth/ui'
import {
  type Screenshot,
  deleteProfileScreenshot,
  openProfileScreenshot,
  getScreenshotData,
  getScreenshotFileName,
} from '@/helpers/screenshots'
import { useNotifications } from '@/store/state'
import dayjs from 'dayjs'

const props = defineProps<{
  screenshot: Screenshot
  profilePath: string
  imagePreviewModal: typeof ImagePreviewModal
}>()
const emit = defineEmits(['deleted'])
const notifications = useNotifications()

const loaded = ref(false)
const blobUrl = ref<string>('')
const isHovered = ref(false)
const wrapperContainer = ref<HTMLElement | null>(null)
let observer: IntersectionObserver | null = null

function onLoad() {
  loaded.value = true
}

async function loadData(): Promise<void> {
  try {
    const base64 = await getScreenshotData(props.profilePath, props.screenshot)
    if (!base64) {
      notifications.addNotification({ title: 'Failed to load screenshot:', text: props.screenshot.path, type: 'error' })
      return
    }

    const binary = atob(base64)
    const bytes = Uint8Array.from(binary, (c) => c.charCodeAt(0))
    const blob = new Blob([bytes], { type: 'image/png' })

    blobUrl.value = URL.createObjectURL(blob)
    loaded.value = true;
  } catch (err: any) {
    notifications.addNotification({
      title: 'Error fetching screenshot',
      text: err.message,
      type: 'error',
    })
  }
}

onMounted(() => {
  observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) {
            loadData()
            if (observer && wrapperContainer.value) {
              observer.unobserve(wrapperContainer.value)
            }
          }
        }
      },
      { rootMargin: '100px', threshold: 0.1 }
  )
  if (wrapperContainer.value) observer.observe(wrapperContainer.value)
})

onBeforeUnmount(() => {
  if (observer && wrapperContainer.value) {
    observer.unobserve(wrapperContainer.value)
  }

  if (blobUrl.value) {
    URL.revokeObjectURL(blobUrl.value)
  }
})

async function copyImageToClipboard() {
  try {
    const resp = await fetch(blobUrl.value)
    const blob = await resp.blob()
    await navigator.clipboard.write([new ClipboardItem({ [blob.type]: blob })])
    notifications.addNotification({
      title: 'Copied to clipboard',
      text: 'The screenshot has been copied successfully.',
      type: 'success',
    })
  } catch (error: any) {
    notifications.addNotification({ title: 'Copy failed', text: error.message, type: 'error' })
  }
}

async function deleteScreenshot() {
  try {
    const ok = await deleteProfileScreenshot(props.profilePath, props.screenshot)
    if (!ok) throw new Error('Delete returned false')
    notifications.addNotification({ title: 'Successfully deleted screenshot', type: 'success' })
    emit('deleted')
  } catch (err: any) {
    notifications.addNotification({
      title: 'Error deleting screenshot',
      text: err.message,
      type: 'error',
    })
  }
}

async function viewInFolder() {
  const ok = await openProfileScreenshot(props.profilePath, props.screenshot)
  if (!ok) {
    notifications.addNotification({ title: 'Unable to open screenshot in folder.', type: 'error' })
  }
}
</script>

<style scoped>
.skeleton {
  background: linear-gradient(
      90deg,
      var(--color-bg) 25%,
      var(--color-raised-bg) 50%,
      var(--color-bg) 75%
  );
  background-size: 200% 100%;
  animation: wave 1500ms infinite linear;
}

@keyframes wave {
  0% {
    background-position: -200% 0;
  }
  100% {
    background-position: 200% 0;
  }
}
</style>
