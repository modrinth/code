<template>
  <ModalWrapper
    ref="modal"
    :header="formatMessage(messages.header)"
    :on-hide="onHide"
    :closable="!updatingImmediately"
  >
    <div>{{ formatMessage(messages.bodyVersion, { version: update!.version }) }}</div>
    <div v-if="updateSize">
      {{ formatMessage(messages.bodySize, { size: formatBytes(updateSize) }) }}
    </div>
    <div>
      <a href="https://modrinth.com/news/changelog?filter=app">{{
        formatMessage(messages.bodyChangelog)
      }}</a>
    </div>
    <ProgressBar class="mt-4" :progress="downloadProgress" />
    <div class="mt-4 flex flex-wrap gap-2">
      <ButtonStyled color="green">
        <button :disabled="updatingImmediately" @click="installUpdateNow">
          <RefreshCwIcon />
          {{ formatMessage(messages.restartNow) }}
        </button>
      </ButtonStyled>
      <ButtonStyled>
        <button
          :disabled="updatingImmediately || downloadInProgress || update!.version == enqueuedUpdate"
          @click="updateAtNextExit"
        >
          <RightArrowIcon />
          {{ formatMessage(messages.later) }}
        </button>
      </ButtonStyled>
      <ButtonStyled color="red">
        <button :disabled="updatingImmediately || downloadInProgress" @click="skipUpdate">
          <XIcon />
          {{ formatMessage(messages.skip) }}
        </button>
      </ButtonStyled>
    </div>
  </ModalWrapper>
</template>

<script setup lang="ts">
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { useTemplateRef, ref } from 'vue'
import { ButtonStyled } from '@modrinth/ui'
import { RefreshCwIcon, XIcon, RightArrowIcon } from '@modrinth/assets'
import { enqueueUpdateForInstallation, getUpdateSize, removeEnqueuedUpdate } from '@/helpers/utils'
import { formatBytes } from '@modrinth/utils'
import { handleError } from '@/store/notifications'
import ProgressBar from '@/components/ui/ProgressBar.vue'
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
    defaultMessage: 'An update is available!',
  },
  bodyVersion: {
    id: 'app.update.modal-body-version',
    defaultMessage: 'Version {version} of the Modrinth App is available for installation.',
  },
  bodySize: {
    id: 'app.update.modal-body-size',
    defaultMessage: 'The download is {size} in size.',
  },
  bodyChangelog: {
    id: 'app.update.modal-body-changelog',
    defaultMessage: 'Click here to view the changelog.',
  },
  restartNow: {
    id: 'app.update.restart',
    defaultMessage: 'Update Now',
  },
  later: {
    id: 'app.update.later',
    defaultMessage: 'Update on Next Restart',
  },
  skip: {
    id: 'app.update.skip',
    defaultMessage: 'Skip This Update',
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

const enqueuedUpdate = ref<string | null>(null)

const modal = useTemplateRef('modal')
const isOpen = ref(false)

async function show(newUpdate: UpdateData) {
  const oldVersion = update.value?.version

  update.value = newUpdate
  updateSize.value = await getUpdateSize(newUpdate.rid).catch(handleError)

  if (oldVersion !== update.value?.version) {
    downloadProgress.value = 0
  }

  modal.value!.show()
  isOpen.value = true
}

function onHide() {
  isOpen.value = false
}

function hide() {
  modal.value!.hide()
}

defineExpose({ show, hide, isOpen })

loading_listener((event) => {
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
  const versionToDownload = update.value!.version

  downloadInProgress.value = true
  try {
    await enqueueUpdateForInstallation(update.value!.rid)
  } catch (e) {
    downloadInProgress.value = false

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
