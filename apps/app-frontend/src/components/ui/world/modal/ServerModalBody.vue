<script setup lang="ts">
import { TeleportDropdownMenu } from '@modrinth/ui'
import type { ServerPackStatus } from '@/helpers/worlds.ts'
import { type MessageDescriptor, defineMessages, useVIntl } from '@vintl/vintl'

const { formatMessage } = useVIntl()

const name = defineModel<string>('name')
const address = defineModel<string>('address')
const resourcePack = defineModel<ServerPackStatus>('resourcePack')

const resourcePackOptions: ServerPackStatus[] = ['enabled', 'prompt', 'disabled']

const resourcePackOptionMessages: Record<ServerPackStatus, MessageDescriptor> = defineMessages({
  enabled: {
    id: 'instance.add-server.resource-pack.enabled',
    defaultMessage: 'Enabled',
  },
  prompt: {
    id: 'instance.add-server.resource-pack.prompt',
    defaultMessage: 'Prompt',
  },
  disabled: {
    id: 'instance.add-server.resource-pack.disabled',
    defaultMessage: 'Disabled',
  },
})

const messages = defineMessages({
  name: {
    id: 'instance.server-modal.name',
    defaultMessage: 'Name',
  },
  address: {
    id: 'instance.server-modal.address',
    defaultMessage: 'Address',
  },
  resourcePack: {
    id: 'instance.server-modal.resource-pack',
    defaultMessage: 'Resource pack',
  },
  placeholderName: {
    id: 'instance.server-modal.placeholder-name',
    defaultMessage: 'Minecraft Server',
  },
})

defineExpose({ resourcePackOptions })
</script>
<template>
  <div class="w-[450px]">
    <h2 class="text-lg font-extrabold text-contrast mt-0 mb-1">
      {{ formatMessage(messages.name) }}
    </h2>
    <input
      v-model="name"
      type="text"
      :placeholder="formatMessage(messages.placeholderName)"
      class="w-full"
      autocomplete="off"
    />
    <h2 class="text-lg font-extrabold text-contrast mt-3 mb-1">
      {{ formatMessage(messages.address) }}
    </h2>
    <input
      v-model="address"
      type="text"
      placeholder="example.modrinth.gg"
      class="w-full"
      autocomplete="off"
    />
    <h2 class="text-lg font-extrabold text-contrast mt-3 mb-1">
      {{ formatMessage(messages.resourcePack) }}
    </h2>
    <div>
      <TeleportDropdownMenu
        v-model="resourcePack"
        :options="resourcePackOptions"
        name="Server resource pack"
        :display-name="
          (option: ServerPackStatus) => formatMessage(resourcePackOptionMessages[option])
        "
      />
    </div>
  </div>
</template>
