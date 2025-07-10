<template>
  <ModalWrapper
    ref="modal"
    :header="formatMessage(messages.header)"
    :on-hide="onHide"
    :closable="!updatingImmediately && !downloadInProgress"
  >
    <div class="flex flex-col gap-4">
      <AppearingProgressBar
        :max-value="shouldShowProgress ? updateSize || 0 : 0"
        :current-value="shouldShowProgress ? downloadedBytes : 0"
        color="green"
        class="w-full"
      />

      <div class="flex flex-col gap-4 max-w-[500px]">
        <div class="flex items-center gap-2 mx-auto">
          <span
            class="inline-flex items-center px-2 py-0.5 rounded text-lg font-semibold bg-red text-bg-raised border border-red"
          >
            v{{ update!.currentVersion }}
          </span>
          <RightArrowIcon class="size-6 text-secondary" />
          <span
            class="inline-flex items-center px-2 py-0.5 rounded text-lg font-semibold bg-brand text-bg-raised border border-green"
          >
            v{{ update!.version }}
          </span>
        </div>
        <div>{{ formatMessage(messages.bodyVersion) }}</div>
        <div>
          {{ formatMessage(messages.downloadSize, { size: formatBytes(updateSize) }) }}
        </div>
        <div class="flex flex-wrap gap-2 w-full">
          <JoinedButtons
            :actions="installActions"
            :disabled="updatingImmediately || downloadInProgress"
            color="green"
          />
          <div>
            <ButtonStyled>
              <a href="https://modrinth.com/news/changelog?filter=app" target="_blank">
                <ExternalIcon /> {{ formatMessage(messages.changelog) }}
              </a>
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
import {
  ExternalIcon,
  RefreshCwIcon,
  RightArrowIcon,
  TimerIcon,
  XCircleIcon,
} from '@modrinth/assets'
import { enqueueUpdateForInstallation, getUpdateSize, removeEnqueuedUpdate } from '@/helpers/utils'
import { formatBytes } from '@modrinth/utils'
import { handleError } from '@/store/notifications'
import { loading_listener } from '@/helpers/events'
import { getCurrentWindow } from '@tauri-apps/api/window'

const emit = defineEmits<{
  (e: 'updateSkipped', version: string): Promise<void>
  (e: 'updateEnqueuedForLater', version: string | null): Promise<void>
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
  header: {
    id: 'app.update.modal-header',
    defaultMessage: 'A new app update is available!',
  },
  bodyVersion: {
    id: 'app.update.modal-body-version',
    defaultMessage:
      'We recommend updating as soon as possible so you can enjoy the latest features and improvements.',
  },
  downloadSize: {
    id: 'app.update.download-size',
    defaultMessage: 'The download size of the update is {size}.',
  },
  changelog: {
    id: 'app.update.changelog',
    defaultMessage: 'Changelog',
  },
  restartNow: {
    id: 'app.update.restart',
    defaultMessage: 'Update now',
  },
  later: {
    id: 'app.update.later',
    defaultMessage: 'Update on next restart',
  },
  skip: {
    id: 'app.update.skip',
    defaultMessage: 'Skip this update',
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
const downloadError = ref(false)

const enqueuedUpdate = ref<string | null>(null)

const installActions = computed<JoinedButtonAction[]>(() => [
  {
    id: 'install-now',
    label: formatMessage(messages.restartNow),
    icon: RefreshCwIcon,
    action: installUpdateNow,
    color: 'green',
  },
  {
    id: 'install-later',
    label: formatMessage(messages.later),
    icon: TimerIcon,
    action: updateAtNextExit,
  },
  {
    id: 'skip',
    label: formatMessage(messages.skip),
    action: skipUpdate,
    icon: XCircleIcon,
    color: 'red',
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
  downloadError.value = false
  downloadProgress.value = 0

  const versionToDownload = update.value!.version

  downloadInProgress.value = true
  try {
    await enqueueUpdateForInstallation(update.value!.rid)
  } catch (e) {
    downloadInProgress.value = false
    downloadError.value = true

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

function skipUpdate() {
  enqueuedUpdate.value = null
  emit('updateSkipped', update.value!.version)

  removeEnqueuedUpdate()
  hide()
}
</script>

<style scoped lang="scss"></style>
