<template>
  <ModalWrapper ref="modal" :header="formatMessage(messages.header)">
    <div>{{ formatMessage(messages.body, { version: update!.version }) }}</div>
    <div class="mt-4 flex flex-wrap gap-2">
      <ButtonStyled color="green">
        <button>
          <RefreshCwIcon />
          {{ formatMessage(messages.restartNow) }}
        </button>
      </ButtonStyled>
      <ButtonStyled>
        <button @click="hide()">
          {{ formatMessage(messages.later) }}
        </button>
      </ButtonStyled>
      <ButtonStyled>
        <button>
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
import type { Update } from '@tauri-apps/plugin-updater'
import { ButtonStyled } from '@modrinth/ui'
import { RefreshCwIcon } from '@modrinth/assets'

const { formatMessage } = useVIntl()

const messages = defineMessages({
  header: {
    id: 'app.update.modal-header',
    defaultMessage: 'An update is available!',
  },
  body: {
    id: 'app.update.modal-body',
    defaultMessage: 'Version {version} of the Modrinth App is available for installation.',
  },
  restartNow: {
    id: 'app.update.restart',
    defaultMessage: 'Restart Now',
  },
  later: {
    id: 'app.update.later',
    defaultMessage: 'Later',
  },
  skip: {
    id: 'app.update.skip',
    defaultMessage: 'Skip This Update',
  },
})

const update = ref<Update>()

const modal = useTemplateRef('modal')
const isOpen = ref(false)

function show(newUpdate: Update) {
  update.value = newUpdate
  modal.value!.show()
  isOpen.value = true
}

function hide() {
  modal.value!.hide()
  isOpen.value = false
}

defineExpose({ show, isOpen })
</script>

<style scoped lang="scss"></style>
