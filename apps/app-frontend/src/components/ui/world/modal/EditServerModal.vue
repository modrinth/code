<script setup lang="ts">
import { SaveIcon, XIcon } from '@modrinth/assets'
import { ButtonStyled, commonMessages } from '@modrinth/ui'
import { ref } from 'vue'
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import type { GameInstance } from '@/helpers/types'
import { edit_server_in_profile, type ServerWorld } from '@/helpers/worlds.ts'
import { defineMessage, useVIntl } from '@vintl/vintl'
import { handleError } from '@/store/notifications'
import ServerModalBody from '@/components/ui/world/modal/ServerModalBody.vue'

const { formatMessage } = useVIntl()

const emit = defineEmits<{
  submit: [server: ServerWorld]
}>()

const props = defineProps<{
  instance: GameInstance
}>()

const modal = ref()

const name = ref()
const address = ref()
const resourcePack = ref('enabled')
const index = ref()

async function saveServer() {
  const serverName = name.value ? name.value : address.value
  const resourcePackStatus = resourcePack.value
  await edit_server_in_profile(
    props.instance.path,
    index.value,
    serverName,
    address.value,
    resourcePackStatus,
  ).catch(handleError)
  emit('submit', {
    name: serverName,
    type: 'server',
    index: index.value,
    address: address.value,
    pack_status: resourcePackStatus,
  })
  hide()
}

function show(server: ServerWorld) {
  name.value = server.name
  address.value = server.address
  resourcePack.value = server.pack_status
  index.value = server.index
  modal.value.show()
}

function hide() {
  modal.value.hide()
}

defineExpose({ show })

const titleMessage = defineMessage({
  id: 'instance.edit-server.title',
  defaultMessage: 'Edit server',
})
</script>
<template>
  <ModalWrapper ref="modal">
    <template #title>
      <span class="font-extrabold text-lg text-contrast">{{ formatMessage(titleMessage) }}</span>
    </template>
    <ServerModalBody
      v-model:name="name"
      v-model:address="address"
      v-model:resource-pack="resourcePack"
    />
    <div class="flex gap-2 mt-4">
      <ButtonStyled color="brand">
        <button :disabled="!address" @click="saveServer">
          <SaveIcon />
          {{ formatMessage(commonMessages.saveChangesButton) }}
        </button>
      </ButtonStyled>
      <ButtonStyled>
        <button @click="hide()">
          <XIcon />
          {{ formatMessage(commonMessages.cancelButton) }}
        </button>
      </ButtonStyled>
    </div>
  </ModalWrapper>
</template>
