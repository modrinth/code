<template>
  <ModalWrapper ref="modal" hide-header :closable="false" :on-hide="onHide">
    <div class="flex flex-col gap-4">
      <div class="w-[500px]">
        <div class="font-extrabold text-contrast text-xl">
          {{ formatMessage(messages.header) }} Modrinth App v{{ update!.version }}
        </div>
        <template v-if="!downloadInProgress && !downloadError">
          <div class="mb-1 leading-tight">{{ formatMessage(messages.bodyVersion) }}</div>
          <div class="text-sm text-secondary mb-2">
            {{ formatMessage(messages.downloadSize, { size: formatBytes(updateSize) }) }}
          </div>
        </template>
        <AppearingProgressBar
          v-if="!downloadError"
          :max-value="shouldShowProgress ? updateSize || 0 : 0"
          :current-value="shouldShowProgress ? downloadedBytes : 0"
          color="green"
          class="w-full mb-4 mt-2"
        />
        <div v-if="downloadError" class="leading-tight">
          <div class="text-red font-medium mb-4">
            {{ formatMessage(messages.downloadError) }}
          </div>
          <div class="flex flex-wrap gap-2">
            <ButtonStyled color="brand">
              <button @click="installUpdateNow">
                <DownloadIcon />
                {{ formatMessage(messages.tryAgain) }}
              </button>
            </ButtonStyled>
            <ButtonStyled>
              <button @click="copyError">
                <ClipboardCopyIcon />
                {{
                  copiedError
                    ? formatMessage(messages.copiedError)
                    : formatMessage(messages.copyError)
                }}
              </button>
            </ButtonStyled>
            <ButtonStyled>
              <a href="https://support.modrinth.com"><ChatIcon /> Get support</a>
            </ButtonStyled>
          </div>
        </div>
        <div v-if="!downloadError" class="flex flex-wrap gap-2 w-full">
          <JoinedButtons
            :actions="installActions"
            :disabled="updatingImmediately || downloadInProgress"
            color="brand"
          />
          <div>
            <ButtonStyled>
              <button @click="() => openUrl('https://modrinth.com/news/changelog?filter=app')">
                <ExternalIcon /> {{ formatMessage(messages.changelog) }}
              </button>
            </ButtonStyled>
          </div>
        </div>
      </div>
    </div>
  </ModalWrapper>
</template>

<script setup lang="ts">
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { useTemplateRef, ref, computed } from 'vue'
import { AppearingProgressBar, ButtonStyled, JoinedButtons } from '@modrinth/ui'
import type { JoinedButtonAction } from '@modrinth/ui'
import { ExternalIcon, DownloadIcon, RedoIcon, ClipboardCopyIcon, XIcon } from '@modrinth/assets'
import { enqueueUpdateForInstallation, getUpdateSize } from '@/helpers/utils'
import { formatBytes } from '@modrinth/utils'
import { handleError } from '@/store/notifications'
import { loading_listener } from '@/helpers/events'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { openUrl } from '@tauri-apps/plugin-opener'
import { ChatIcon } from '@/assets/icons'

const emit = defineEmits<{
  (e: 'updateEnqueuedForLater', version: string | null): Promise<void>
  (e: 'modalHidden'): void
}>()

const { formatMessage } = useVIntl()
const messages = defineMessages({
  header: {
    id: 'app.update.modal-header',
    defaultMessage: 'Update available - ',
  },
  copiedError: {
    id: 'app.update.copied-error',
    defaultMessage: 'Copied to clipboard!',
  },
  bodyVersion: {
    id: 'app.update.modal-body-version',
    defaultMessage:
      'We recommend updating as soon as possible so you can enjoy the latest features, fixes, and improvements.',
  },
  downloadSize: {
    id: 'app.update.download-size',
    defaultMessage: 'The update is {size}.',
  },
  changelog: {
    id: 'app.update.changelog',
    defaultMessage: 'View changelog',
  },
  restartNow: {
    id: 'app.update.restart',
    defaultMessage: 'Update now',
  },
  later: {
    id: 'app.update.later',
    defaultMessage: 'Update on exit',
  },
  downloadError: {
    id: 'app.update.download-error',
    defaultMessage:
      'An error occurred while downloading the update. Please try again later. Contact support if the issue persists.',
  },
  copyError: {
    id: 'app.update.copy-error',
    defaultMessage: 'Copy error',
  },
  tryAgain: {
    id: 'app.update.try-again',
    defaultMessage: 'Try again',
  },
  hide: {
    id: 'app.update.hide',
    defaultMessage: 'Hide update reminder',
  },
})

type UpdateData = {
  rid: number
  currentVersion: string
  version: string
  date?: string
  body?: string
  rawJson: Record<string, unknown>
}

const update = ref<UpdateData>()
const updateSize = ref<number>()

const updatingImmediately = ref(false)
const downloadInProgress = ref(false)
const downloadProgress = ref(0)
const copiedError = ref(false)
const downloadError = ref<Error | null>(null)

const enqueuedUpdate = ref<string | null>(null)

const installActions = computed<JoinedButtonAction[]>(() => [
  {
    id: 'install-now',
    label: formatMessage(messages.restartNow),
    icon: DownloadIcon,
    action: installUpdateNow,
    color: 'green',
  },
  {
    id: 'install-later',
    label: formatMessage(messages.later),
    icon: RedoIcon,
    action: updateAtNextExit,
  },
  {
    id: 'hide',
    label: formatMessage(messages.hide),
    action: () => {
      hide()
      emit('modalHidden')
    },
    icon: XIcon,
  },
])

const downloadedBytes = computed(() => {
  return updateSize.value ? Math.round((downloadProgress.value / 100) * updateSize.value) : 0
})

const shouldShowProgress = computed(() => {
  return downloadInProgress.value || updatingImmediately.value
})

const modal = useTemplateRef('modal')
const isOpen = ref(false)

async function show(newUpdate: UpdateData) {
  const oldVersion = update.value?.version

  update.value = newUpdate
  updateSize.value = await getUpdateSize(newUpdate.rid).catch(handleError)

  if (oldVersion !== update.value?.version) {
    downloadProgress.value = 0
  }

  modal.value!.show(new MouseEvent('click'))
  isOpen.value = true
}

function onHide() {
  isOpen.value = false
}

function hide() {
  modal.value!.hide()
}

defineExpose({ show, hide, isOpen })

async function copyError() {
  if (downloadError.value) {
    copiedError.value = true
    const errorData = {
      message: downloadError.value.message,
      stack: downloadError.value.stack,
      name: downloadError.value.name,
      timestamp: new Date().toISOString(),
      updateVersion: update.value?.version,
    }

    setTimeout(() => {
      copiedError.value = false
    }, 3000)

    try {
      await navigator.clipboard.writeText(JSON.stringify(errorData, null, 2))
    } catch (e) {
      console.error('Failed to copy error to clipboard:', e)
    }
  }
}

// TODO: Migrate to common events.ts helper when events/listeners are refactored
interface LoadingListenerEvent {
  event: {
    type: 'launcher_update'
    version: string
  }
  fraction?: number
}

loading_listener((event: LoadingListenerEvent) => {
  if (event.event.type === 'launcher_update') {
    if (event.event.version === update.value!.version) {
      downloadProgress.value = (event.fraction ?? 1.0) * 100
    }
  }
})

function installUpdateNow() {
  updatingImmediately.value = true

  if (enqueuedUpdate.value !== update.value!.version) {
    downloadUpdate()
  } else if (!downloadInProgress.value) {
    // Update already downloaded. Simply close the app
    getCurrentWindow().close()
  }
}

function updateAtNextExit() {
  enqueuedUpdate.value = update.value!.version
  emit('updateEnqueuedForLater', update.value!.version)

  downloadUpdate()
  hide()
}

async function downloadUpdate() {
  downloadError.value = null
  downloadProgress.value = 0

  const versionToDownload = update.value!.version

  downloadInProgress.value = true
  try {
    await enqueueUpdateForInstallation(update.value!.rid)
  } catch (e) {
    downloadInProgress.value = false
    downloadError.value = e instanceof Error ? e : new Error(String(e))

    handleError(e)

    enqueuedUpdate.value = null
    updatingImmediately.value = false
    await emit('updateEnqueuedForLater', null)
    return
  }
  downloadInProgress.value = false

  if (updatingImmediately.value && update.value!.version === versionToDownload) {
    await getCurrentWindow().close()
  }
}
</script>

<style scoped lang="scss"></style>
