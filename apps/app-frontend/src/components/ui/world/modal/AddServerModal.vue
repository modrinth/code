<script setup lang="ts">
import { PlusIcon, XIcon } from '@modrinth/assets'
import { ButtonStyled, commonMessages } from '@modrinth/ui'
import { ref } from 'vue'
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import type { GameInstance } from '@/helpers/types'
import InstanceModalTitlePrefix from '@/components/ui/modal/InstanceModalTitlePrefix.vue'
import { add_server_to_profile, type ServerWorld } from '@/helpers/worlds.ts'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { handleError } from '@/store/notifications'
import ServerModalBody from '@/components/ui/world/modal/ServerModalBody.vue'

const { formatMessage } = useVIntl()

const emit = defineEmits<{
  'submit': [server: ServerWorld]
}>()

const props = defineProps<{
  instance: GameInstance
}>()

const modal = ref()

const name = ref()
const address = ref()
const resourcePack = ref('enabled')

async function addServer() {
  const serverName = name.value ? name.value : address.value
  const resourcePackStatus = resourcePack.value
  const index = await add_server_to_profile(
    props.instance.path,
    serverName,
    address.value,
    resourcePackStatus,
  ).catch(handleError)
  emit('submit', {
    name: serverName,
    type: 'server',
    index,
    address: address.value,
    pack_status: resourcePackStatus,
  })
  hide()
}

function show() {
  name.value = ''
  address.value = ''
  resourcePack.value = 'enabled'
  modal.value.show()
}

function hide() {
  modal.value.hide()
}

const messages = defineMessages({
  title: {
    id: 'instance.add-server.title',
    defaultMessage: 'Add a server',
  },
  addServer: {
    id: 'instance.add-server.add-server',
    defaultMessage: 'Add server',
  },
})

defineExpose({ show, hide })
</script>
<template>
  <ModalWrapper ref="modal">
    <template #title>
      <span class="flex items-center gap-2 text-lg font-semibold text-primary">
        <InstanceModalTitlePrefix :instance="instance" />
        <span class="font-extrabold text-contrast">{{ formatMessage(messages.title) }}</span>
      </span>
    </template>
    <ServerModalBody
      v-model:name="name"
      v-model:address="address"
      v-model:resource-pack="resourcePack"
    />
    <div class="flex gap-2 mt-4">
      <ButtonStyled color="brand">
        <button :disabled="!address" @click="addServer">
          <PlusIcon />
          {{ formatMessage(messages.addServer) }}
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
