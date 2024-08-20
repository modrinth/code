<script setup>
import { XIcon, DownloadIcon } from '@modrinth/assets'
import { Button, Modal } from '@modrinth/ui'
import { install as pack_install } from '@/helpers/pack'
import { ref } from 'vue'
import { mixpanel_track } from '@/helpers/mixpanel'
import { useTheming } from '@/store/theme.js'
import { handleError } from '@/store/state.js'

const themeStore = useTheming()

const versionId = ref()
const project = ref()
const confirmModal = ref(null)
const installing = ref(false)

let onInstall = ref(() => {})

defineExpose({
  show: (projectVal, versionIdVal, callback) => {
    project.value = projectVal
    versionId.value = versionIdVal
    installing.value = false
    confirmModal.value.show()

    onInstall.value = callback

    mixpanel_track('PackInstallStart')
  },
})

async function install() {
  installing.value = true
  confirmModal.value.hide()

  await pack_install(
    project.value.id,
    versionId.value,
    project.value.title,
    project.value.icon_url,
  ).catch(handleError)
  mixpanel_track('PackInstall', {
    id: project.value.id,
    version_id: versionId.value,
    title: project.value.title,
    source: 'ConfirmModal',
  })

  onInstall.value(versionId.value)
  installing.value = false
}
</script>

<template>
  <Modal
    ref="confirmModal"
    header="Are you sure?"
    :noblur="!themeStore.advancedRendering"
    :on-hide="onInstall"
  >
    <div class="modal-body">
      <p>You already have this modpack installed. Are you sure you want to install it again?</p>
      <div class="input-group push-right">
        <Button @click="() => $refs.confirmModal.hide()"><XIcon />Cancel</Button>
        <Button color="primary" :disabled="installing" @click="install()"
          ><DownloadIcon /> {{ installing ? 'Installing' : 'Install' }}</Button
        >
      </div>
    </div>
  </Modal>
</template>

<style lang="scss" scoped>
.modal-body {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 1rem;
}
</style>
