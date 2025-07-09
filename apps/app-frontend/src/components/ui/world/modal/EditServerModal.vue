<script setup lang="ts">
import { SaveIcon, XIcon } from '@modrinth/assets'
import { ButtonStyled, commonMessages } from '@modrinth/ui'
import { computed, ref } from 'vue'
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import type { GameInstance } from '@/helpers/types'
import {
  type ServerPackStatus,
  edit_server_in_profile,
  type ServerWorld,
  set_world_display_status,
  type DisplayStatus,
} from '@/helpers/worlds.ts'
import { defineMessage, useVIntl } from '@vintl/vintl'
import { handleError } from '@/store/notifications'
import ServerModalBody from '@/components/ui/world/modal/ServerModalBody.vue'
import HideFromHomeOption from '@/components/ui/world/modal/HideFromHomeOption.vue'

const { formatMessage } = useVIntl()

const emit = defineEmits<{
  submit: [server: ServerWorld]
}>()

const props = defineProps<{
  instance: GameInstance
}>()

const modal = ref()

const name = ref<string>('')
const address = ref<string>('')
const resourcePack = ref<ServerPackStatus>('enabled')
const index = ref<number>(0)
const displayStatus = ref<DisplayStatus>('normal')
const hideFromHome = ref(false)

const newDisplayStatus = computed(() => (hideFromHome.value ? 'hidden' : 'normal'))

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

  if (newDisplayStatus.value !== displayStatus.value) {
    await set_world_display_status(
      props.instance.path,
      'server',
      address.value,
      newDisplayStatus.value,
    ).catch(handleError)
  }

  emit('submit', {
    name: serverName,
    type: 'server',
    index: index.value,
    address: address.value,
    pack_status: resourcePackStatus,
    display_status: newDisplayStatus.value,
  })
  hide()
}

function show(server: ServerWorld) {
  name.value = server.name
  address.value = server.address
  resourcePack.value = server.pack_status
  index.value = server.index
  displayStatus.value = server.display_status
  hideFromHome.value = server.display_status === 'hidden'
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
    <HideFromHomeOption v-model="hideFromHome" class="mt-3" />
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
