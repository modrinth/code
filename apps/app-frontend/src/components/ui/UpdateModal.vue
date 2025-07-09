<template>
  <ModalWrapper ref="modal" :header="formatMessage(messages.header)" :on-hide="onHide">
    <div>{{ formatMessage(messages.bodyVersion, { version: update!.version }) }}</div>
    <div v-if="updateSize">
      {{ formatMessage(messages.bodySize, { size: formatBytes(updateSize) }) }}
    </div>
    <div>
      <a href="https://modrinth.com/news/changelog?filter=app">{{
        formatMessage(messages.bodyChangelog)
      }}</a>
    </div>
    <div class="mt-4 flex flex-wrap gap-2">
      <ButtonStyled color="green">
        <button>
          <RefreshCwIcon />
          {{ formatMessage(messages.restartNow) }}
        </button>
      </ButtonStyled>
      <ButtonStyled>
        <button @click="hide">
          {{ formatMessage(messages.later) }}
        </button>
      </ButtonStyled>
      <ButtonStyled color="red">
        <button @click="skipUpdate">
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
import { RefreshCwIcon } from '@modrinth/assets'
import { getUpdateSize } from '@/helpers/utils'
import { formatBytes } from '@modrinth/utils'
import { handleError } from '@/store/notifications'

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
  version: string
}

const update = ref<UpdateData>()
const updateSize = ref<number>()

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

function skipUpdate() {
  hide()
  emit('updateSkipped', update.value!.version)
}
</script>

<style scoped lang="scss"></style>
