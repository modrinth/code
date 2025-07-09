<template>
  <ModalWrapper
    ref="modal"
    :header="formatMessage(messages.header)"
    :on-hide="onHide"
    :closable="!updateInProgress"
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
        <button :disabled="updateInProgress" @click="installUpdateNow">
          <RefreshCwIcon />
          {{ formatMessage(messages.restartNow) }}
        </button>
      </ButtonStyled>
      <ButtonStyled>
        <button :disabled="updateInProgress">
          <RightArrowIcon />
          {{ formatMessage(messages.later) }}
        </button>
      </ButtonStyled>
      <ButtonStyled color="red">
        <button :disabled="updateInProgress" @click="skipUpdate">
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
import { getUpdateSize } from '@/helpers/utils'
import { formatBytes } from '@modrinth/utils'
import { handleError } from '@/store/notifications'
import ProgressBar from '@/components/ui/ProgressBar.vue'
import { Update } from '@tauri-apps/plugin-updater'

const emit = defineEmits<{
  (e: 'updateSkipped', version: string): void
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
const updateInProgress = ref(false)
const downloadProgress = ref(0)

const modal = useTemplateRef('modal')
const isOpen = ref(false)

async function show(newUpdate: UpdateData) {
  update.value = newUpdate
  updateSize.value = await getUpdateSize(newUpdate.rid).catch(handleError)
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

function installUpdateNow() {
  updateInProgress.value = true
  let totalSize = 0
  let totalDownloaded = 0
  new Update(update.value!).downloadAndInstall((event) => {
    if (event.event === 'Started') {
      totalSize = event.data.contentLength!
    } else if (event.event === 'Progress') {
      totalDownloaded += event.data.chunkLength
    } else if (event.event === 'Finished') {
      totalDownloaded = totalSize
    }
    downloadProgress.value = (totalDownloaded / totalSize) * 100
  })
}

function skipUpdate() {
  hide()
  emit('updateSkipped', update.value!.version)
}
</script>

<style scoped lang="scss"></style>
