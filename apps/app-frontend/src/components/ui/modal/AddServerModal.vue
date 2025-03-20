<script setup lang="ts">
import { PlusIcon, XIcon } from '@modrinth/assets'
import { ButtonStyled, TeleportDropdownMenu } from '@modrinth/ui'
import { ref } from 'vue'
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import type { GameInstance } from '@/helpers/types'
import InstanceModalTitlePrefix from '@/components/ui/modal/InstanceModalTitlePrefix.vue'
import { type ServerPackStatus, add_server_to_profile, type ServerWorld } from '@/helpers/worlds'
import { type MessageDescriptor, defineMessage, defineMessages, useVIntl } from '@vintl/vintl'
import { handleError } from '@/store/notifications'

const { formatMessage } = useVIntl()

const emit = defineEmits<{
  'add-server': [server: ServerWorld]
}>()

const props = defineProps<{
  edit?: boolean
  instance: GameInstance
}>()

const modal = ref()

const defaultName = ref('Minecraft Server')
const name = ref()
const address = ref()

type ResourcePackOption = {
  id: ServerPackStatus
  message: MessageDescriptor
}
const resourcePackOptions = ref<ResourcePackOption[]>([
  {
    id: 'enabled',
    message: defineMessage({
      id: 'instance.add-server.resource-pack.enabled',
      defaultMessage: 'Enabled',
    }),
  },
  {
    id: 'prompt',
    message: defineMessage({
      id: 'instance.add-server.resource-pack.prompt',
      defaultMessage: 'Prompt',
    }),
  },
  {
    id: 'disabled',
    message: defineMessage({
      id: 'instance.add-server.resource-pack.disabled',
      defaultMessage: 'Disabled',
    }),
  },
])

const resourcePackOption = ref(resourcePackOptions.value[0])

async function addServer() {
  const serverName = name.value ? name.value : address.value
  const status = resourcePackOption.value.id
  const index = await add_server_to_profile(props.instance.path, serverName, address.value, status).catch(handleError)
  emit('add-server', {
    name: serverName,
    type: 'server',
    index,
    address: address.value,
    pack_status: status,
  })
  hide()
}

function show() {
  name.value = ''
  address.value = ''
  resourcePackOption.value = resourcePackOptions.value[0]
  modal.value.show()
}

function hide() {
  modal.value.hide()
}

defineExpose({ show })

const messages = defineMessages({
  title: {
    id: 'instance.add-server.title',
    defaultMessage: 'Add a server',
  },
  editTitle: {
    id: 'instance.edit-server.title',
    defaultMessage: 'Edit server',
  },
  name: {
    id: 'instance.add-server.name',
    defaultMessage: 'Name',
  },
  address: {
    id: 'instance.add-server.address',
    defaultMessage: 'Address',
  },
  resourcePack: {
    id: 'instance.add-server.resource-pack',
    defaultMessage: 'Resource pack',
  },
})
</script>
<template>
  <ModalWrapper ref="modal">
    <template #title>
      <span class="flex items-center gap-2 text-lg font-semibold text-primary">
        <InstanceModalTitlePrefix :instance="instance" />
        <span class="font-extrabold text-contrast">{{ formatMessage(messages.title) }}</span>
      </span>
    </template>
    <h2 class="text-lg font-extrabold text-contrast mt-0 mb-1">{{ formatMessage(messages.name) }}</h2>
    <input
      v-model="name"
      type="text"
      :placeholder="defaultName"
      class="w-full"
      autocomplete="off"
    />
    <h2 class="text-lg font-extrabold text-contrast mt-3 mb-1">{{ formatMessage(messages.address) }}</h2>
    <input
      v-model="address"
      type="text"
      placeholder="example.modrinth.gg"
      class="w-full"
      autocomplete="off"
    />
    <h2 class="text-lg font-extrabold text-contrast mt-3 mb-1">{{ formatMessage(messages.resourcePack) }}</h2>
    <div>
      <TeleportDropdownMenu
        v-model="resourcePackOption"
        :options="resourcePackOptions"
        name="Time range"
        :display-name="(option: ResourcePackOption) => formatMessage(option.message)"
      />
    </div>
    <div class="flex gap-2 mt-4">
      <ButtonStyled color="brand">
        <button :disabled="!address" @click="addServer">
          <PlusIcon />
          Add server
        </button>
      </ButtonStyled>
      <ButtonStyled>
        <button @click="hide()">
          <XIcon />
          Cancel
        </button>
      </ButtonStyled>
    </div>
  </ModalWrapper>
</template>
