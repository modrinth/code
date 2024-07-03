<script setup>
import { Button, Modal, XIcon, DownloadIcon } from 'omorphia'
import { install as pack_install } from '@/helpers/pack'
import { ref } from 'vue'
import { mixpanel_track } from '@/helpers/mixpanel'
import { useTheming } from '@/store/theme.js'
import { handleError } from '@/store/state.js'

const themeStore = useTheming()

const version = ref('')
const title = ref('')
const projectId = ref('')
const icon = ref('')
const confirmModal = ref(null)
const installing = ref(false)

defineExpose({
  show: (projectIdVal, versionId, projectTitle, projectIcon) => {
    projectId.value = projectIdVal
    version.value = versionId
    title.value = projectTitle
    icon.value = projectIcon
    installing.value = false
    confirmModal.value.show()

    mixpanel_track('PackInstallStart')
  },
})

async function install() {
  installing.value = true
  console.log(`Installing ${projectId.value} ${version.value} ${title.value} ${icon.value}`)
  confirmModal.value.hide()
  await pack_install(
    projectId.value,
    version.value,
    title.value,
    icon.value ? icon.value : null,
  ).catch(handleError)
  mixpanel_track('PackInstall', {
    id: projectId.value,
    version_id: version.value,
    title: title.value,
    source: 'ConfirmModal',
  })
}
</script>

<template>
  <Modal ref="confirmModal" header="Are you sure?" :noblur="!themeStore.advancedRendering">
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
