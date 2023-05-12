<script setup>
import { Button, Modal, XIcon, DownloadIcon } from 'omorphia'
import { install as pack_install } from '@/helpers/pack'
import { ref } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()

const version = ref('')
const title = ref('')
const icon = ref('')
const confirmModal = ref(null)
const installing = ref(false)

defineExpose({
  show: (id, projectTitle, projectIcon) => {
    version.value = id
    title.value = projectTitle
    icon.value = projectIcon
    confirmModal.value.show()
  },
})

async function install() {
  installing.value = true
  let id = await pack_install(version.value)
  await pack_install(version.value, title.value, icon.value ? icon.value : null)
  await router.push({ path: `/instance/${encodeURIComponent(id)}` })
  confirmModal.value.hide()
}
</script>

<template>
  <Modal ref="confirmModal" header="Are you sure?">
    <div class="modal-body">
      <p>
        This project is already installed on your system. Are you sure you want to install it again?
      </p>
      <div class="button-group">
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

.button-group {
  display: flex;
  flex-direction: row;
  gap: 0.5rem;
  justify-content: flex-end;
}
</style>
