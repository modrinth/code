<template>
  <ModalWrapper
    ref="modal"
    :header="formatMessage(messages.title)"
    :on-hide="() => setEnabled(true)"
  >
    <div class="flex flex-col gap-4">
      <div class="text-base leading-normal">
        {{ formatMessage(messages.body) }}
      </div>
      <div class="flex flex-wrap gap-2">
        <ButtonStyled>
          <button @click="() => setEnabled(false)">
            {{ formatMessage(messages.disable) }}
          </button>
        </ButtonStyled>
        <ButtonStyled>
          <button @click="() => setEnabled(true)">
            {{ formatMessage(messages.keepEnabled) }}
          </button>
        </ButtonStyled>
      </div>
    </div>
  </ModalWrapper>
</template>

<script setup lang="ts">
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { useTemplateRef } from 'vue'
import { ButtonStyled } from '@modrinth/ui'
import { get as getSettings, set as setSettings } from '@/helpers/settings.ts'
import type UpdateModal from '@/components/ui/modal/UpdateModal.vue'

const props = defineProps<{
  updateModal: InstanceType<typeof UpdateModal>
}>()

const { formatMessage } = useVIntl()
const messages = defineMessages({
  title: {
    id: 'app.meteredNetwork.title',
    defaultMessage: 'Metered Network Detected',
  },
  body: {
    id: 'app.meteredNetwork.body',
    defaultMessage:
      'You appear to be on a metered network. Would you like to disable automatic file downloads?',
  },
  disable: {
    id: 'app.meteredNetwork.disable',
    defaultMessage: 'Disable',
  },
  keepEnabled: {
    id: 'app.meteredNetwork.keepEnabled',
    defaultMessage: 'Keep Enabled',
  },
})

const modal = useTemplateRef('modal')
async function setEnabled(enabled: boolean) {
  const settings = await getSettings()
  if (settings.auto_download_updates !== null) return

  settings.auto_download_updates = enabled
  await setSettings(settings)
  modal.value!.hide()
  if (enabled) {
    props.updateModal.updateAtNextExit()
  }
}

defineExpose({
  show: () => modal.value!.show(),
})
</script>

<style scoped lang="scss"></style>
