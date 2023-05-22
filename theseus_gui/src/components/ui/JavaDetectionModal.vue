<template>
  <Modal ref="detectJavaModal" header="Select java version">
    <div class="auto-detect-modal">
      <div class="table">
        <div class="table-row table-head">
          <div class="table-cell table-text">Version</div>
          <div class="table-cell table-text">Path</div>
          <div class="table-cell table-text">Actions</div>
        </div>
        <div v-for="javaInstall in chosenInstallOptions" :key="javaInstall.path" class="table-row">
          <div class="table-cell table-text">
            <span>{{ javaInstall.version }}</span>
          </div>
          <div class="table-cell table-text">
            <span>{{ javaInstall.path }}</span>
          </div>
          <div class="table-cell table-text manage">
            <Button v-if="currentSelected.path === javaInstall.path" disabled
              ><CheckIcon /> Selected</Button
            >
            <Button v-else @click="setJavaInstall(javaInstall)"><PlusIcon /> Select</Button>
          </div>
        </div>
        <div v-if="chosenInstallOptions.length === 0" class="table-row entire-row">
          <div class="table-cell table-text">No JARS Found!</div>
        </div>
      </div>
      <div class="button-group">
        <Button @click="$refs.detectJavaModal.hide()">
          <XIcon />
          Cancel
        </Button>
      </div>
    </div>
  </Modal>
</template>
<script setup>
import { Modal, PlusIcon, CheckIcon, Button, XIcon } from 'omorphia'
import { ref } from 'vue'
import {
  find_jre_17_jres,
  find_jre_18plus_jres,
  find_jre_8_jres,
  get_all_jre,
} from '@/helpers/jre.js'
import { handleError } from '@/store/notifications.js'

const chosenInstallOptions = ref([])
const detectJavaModal = ref(null)
const currentSelected = ref({})

defineExpose({
  show: async (version, currentSelectedJava) => {
    if (version <= 8 && !!version) {
      console.log(version)
      chosenInstallOptions.value = await find_jre_8_jres().catch(handleError)
    } else if (version >= 18) {
      chosenInstallOptions.value = await find_jre_18plus_jres().catch(handleError)
    } else if (version) {
      chosenInstallOptions.value = await find_jre_17_jres().catch(handleError)
    } else {
      console.log('get all')
      chosenInstallOptions.value = await get_all_jre().catch(handleError)
    }

    currentSelected.value = currentSelectedJava
    if (!currentSelected.value) {
      currentSelected.value = { path: '', version: '' }
    }

    detectJavaModal.value.show()
  },
})

const emit = defineEmits(['submit'])

function setJavaInstall(javaInstall) {
  emit('submit', javaInstall)
  detectJavaModal.value.hide()
}
</script>
<style lang="scss" scoped>
.auto-detect-modal {
  padding: 1rem;

  .table {
    .table-row {
      grid-template-columns: 1fr 4fr 1.5fr;
    }

    span {
      display: inherit;
      align-items: center;
      justify-content: center;
    }
  }
}

.button-group {
  margin-top: 1rem;
  display: flex;
  gap: 0.5rem;
  justify-content: flex-end;
}

.manage {
  display: flex;
  gap: 0.5rem;
}
</style>
