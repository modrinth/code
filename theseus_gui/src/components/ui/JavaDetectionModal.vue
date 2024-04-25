<template>
  <Modal ref="detectJavaModal" header="Select java version" :noblur="!themeStore.advancedRendering">
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
          <div v-tooltip="javaInstall.path" class="table-cell table-text">
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
          <div class="table-cell table-text">No java installations found!</div>
        </div>
      </div>
      <div class="input-group push-right">
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
import { find_filtered_jres } from '@/helpers/jre.js'
import { handleError } from '@/store/notifications.js'
import { mixpanel_track } from '@/helpers/mixpanel'
import { useTheming } from '@/store/theme.js'

const themeStore = useTheming()

const chosenInstallOptions = ref([])
const detectJavaModal = ref(null)
const currentSelected = ref({})

defineExpose({
  show: async (version, currentSelectedJava) => {
    chosenInstallOptions.value = await find_filtered_jres(version).catch(handleError)

    console.log(chosenInstallOptions.value)
    console.log(version)

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
  mixpanel_track('JavaAutoDetect', {
    path: javaInstall.path,
    version: javaInstall.version,
  })
}
</script>
<style lang="scss" scoped>
.auto-detect-modal {
  padding: 1rem;

  .table {
    .table-row {
      grid-template-columns: 1fr 4fr min-content;
    }

    span {
      display: inherit;
      align-items: center;
      justify-content: center;
    }

    padding: 0.5rem;
  }
}

.manage {
  display: flex;
  gap: 0.5rem;
}
</style>
