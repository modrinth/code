<script setup>
import { Button, Modal, XIcon, DownloadIcon } from 'omorphia'
import { useRouter } from 'vue-router'
import { install as pack_install } from '@/helpers/pack'
import { ref } from 'vue'

const router = useRouter()

const version = ref('')
const confirmModal = ref(null)

defineExpose({
  show: (id) => {
    version.value = id
    confirmModal.value.show()
  },
})

async function install() {
  let id = await pack_install(version.value)
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
        <Button color="primary" @click="install()"><DownloadIcon /> Install</Button>
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
