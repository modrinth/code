<template>
  <div
    class="group rounded-lg relative overflow-hidden shadow-md w-full text-contrast"
    @mouseenter="isHovered = true"
    @mouseleave="isHovered = false"
  >
    <div
      v-if="loaded"
      class="absolute top-2 right-2 flex gap-1 transition-opacity duration-200 z-10"
      :class="{
        'opacity-0': !isHovered,
        'opacity-100': isHovered,
      }"
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
        :alt="getFileName(screenshot.path)"
        :src="`data:image/png;base64,${imageData}`"
        class="w-full h-full object-cover transition-opacity duration-700"
        :class="{ 'opacity-0': !loaded, 'opacity-100': loaded }"
        @load="onLoad"
      />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { ref, onMounted } from 'vue'
import { ClipboardCopyIcon, TrashIcon, ExternalIcon } from '@modrinth/assets'
import { Button } from '@modrinth/ui'
import {
  type Screenshot,
  deleteProfileScreenshot,
  openProfileScreenshot,
  getScreenshotData,
} from '@/helpers/screenshots'
import { useNotifications } from '@/store/state'

const notifications = useNotifications()

const props = defineProps<{
  screenshot: Screenshot
  profilePath: string
}>()

const emit = defineEmits(['deleted'])

const loaded = ref(false)
const imageData = ref<string>('')

// Note: cant use tailwind group because it's being used in the parent component
const isHovered = ref(false)

const onLoad = () => {
  loaded.value = true
}

const getFileName = (path: string | undefined) => {
  if (!path) return 'Untitled'
  return path.split('/').pop()!
}

onMounted(async () => {
  try {
    const result = await getScreenshotData(props.profilePath, props.screenshot)
    if (result) {
      imageData.value = result
      loaded.value = true
    } else {
      notifications.addNotification({
        title: 'Failed to load image',
        type: 'error',
      })
    }
    // eslint-disable-next-line
  } catch (err: any) {
    notifications.addNotification({
      title: 'Error fetching screenshot',
      text: err.message,
      type: 'error',
    })
  }
})

const copyImageToClipboard = async () => {
  try {
    const binary = atob(imageData.value)
    const bytes = Uint8Array.from(binary, (char) => char.charCodeAt(0))
    const blob = new Blob([bytes], { type: 'image/png' })
    const clipboardItem = new ClipboardItem({ 'image/png': blob })
    await navigator.clipboard.write([clipboardItem])

    notifications.addNotification({
      title: 'Copied to clipboard',
      text: 'The screenshot has been copied successfully.',
      type: 'success',
    })
    // eslint-disable-next-line
  } catch (error: any) {
    notifications.addNotification({
      title: 'Copy failed',
      text: error.message,
      type: 'error',
    })
  }
}

const deleteScreenshot = async () => {
  try {
    const result = await deleteProfileScreenshot(props.profilePath, props.screenshot)
    if (!result) {
      notifications.addNotification({
        title: 'Unable to delete screenshot',
        type: 'error',
      })
    } else {
      notifications.addNotification({
        title: 'Successfully deleted screenshot',
        type: 'success',
      })
      emit('deleted')
    }
    // eslint-disable-next-line
  } catch (err: any) {
    notifications.addNotification({
      title: 'Error deleting screenshot',
      text: err.message,
      type: 'error',
    })
  }
}

const viewInFolder = () => {
  openProfileScreenshot(props.profilePath, props.screenshot)
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
