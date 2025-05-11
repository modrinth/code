<template>
  <div class="rounded-lg overflow-hidden shadow-md w-full text-contrast bg-bg-raised">
    <div class="relative">
      <img
          :alt="getFileName(screenshot.path)"
          :src="`data:image/png;base64,${screenshot.data}`"
          class="w-full h-auto object-contain"
      />
    </div>
    <div class="p-4">
      <div class="flex items-center gap-2">
        <div class="font-medium truncate max-w-[calc(100%-120px)]">
          {{ getFileName(screenshot.path) }}
        </div>
        <div class="flex gap-1 ml-auto">
          <Button icon-only title="Rename" @click="renameScreenshot">
            <EditIcon/>
          </Button>
          <Button icon-only title="Copy" @click="copyImageToClipboard">
            <ClipboardCopyIcon/>
          </Button>
          <Button icon-only title="Share" @click="shareScreenshot">
            <ShareIcon/>
          </Button>
          <Button color="red" icon-only title="Delete" @click="deleteScreenshot">
            <TrashIcon/>
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import {ClipboardCopyIcon, EditIcon, ShareIcon, TrashIcon} from '@modrinth/assets'
import {Button} from '@modrinth/ui'
import type {Screenshot} from '@/helpers/screenshots.ts'
import {useNotifications} from '@/store/state'

const notifications = useNotifications()

const props = defineProps<{
  screenshot: Screenshot
}>()

const getFileName = (path: string | undefined) => {
  if (!path) return 'Untitled'
  return path.split('/').pop()
}

const copyImageToClipboard = async () => {
  try {
    const base64 = props.screenshot.data
    const binary = atob(base64)

    const bytes = Uint8Array.from(binary, (char) => char.charCodeAt(0))

    const blob = new Blob([bytes], {type: `data:image/png`})
    const clipboardItem = new ClipboardItem({'image/png': blob})

    await navigator.clipboard.write([clipboardItem])

    notifications.addNotification({
      title: 'Copied to clipboard',
      text: 'The screenshot has successfully been copied to your clipboard.',
      type: 'success',
    })
    // eslint-disable-next-line
  } catch (error: any) {
    notifications.addNotification({
      title: 'Failed to copy screenshot',
      text: error.message,
      type: 'warn',
    })
  }
}

const renameScreenshot = () => {
}

const deleteScreenshot = () => {
}

const shareScreenshot = () => {
}
</script>
