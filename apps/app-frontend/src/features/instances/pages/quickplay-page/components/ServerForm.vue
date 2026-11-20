<script setup lang="ts">
import { Combobox, defineMessages, Input, type MessageDescriptor, useVIntl } from '@modrinth/ui'
import { computed, useId } from 'vue'

import type { ServerPackStatus } from '@/platform/app-lib/worlds/types'

const { formatMessage } = useVIntl()

const name = defineModel<string>('name', { required: true })
const address = defineModel<string>('address', { required: true })
const resourcePack = defineModel<ServerPackStatus>('resourcePack', { required: true })

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
	placeholderAddress: {
		id: 'app.world.server-modal.placeholder-address',
		defaultMessage: 'example.modrinth.gg',
	},
})

const RESOURCE_PACK_OPTIONS: ServerPackStatus[] = ['enabled', 'prompt', 'disabled']

const resourcePackOptions = computed(() =>
	RESOURCE_PACK_OPTIONS.map((value) => ({
		value,
		label: formatMessage(resourcePackOptionMessages[value]),
	})),
)

const id = useId()
const nameId = `${id}-name`
const addressId = `${id}-address`
</script>

<template>
	<div class="flex w-full flex-col gap-4">
		<div class="flex flex-col gap-2">
			<label :for="nameId" class="font-semibold text-contrast">
				{{ formatMessage(messages.name) }}
			</label>
			<Input
				:id="nameId"
				v-model="name"
				:placeholder="formatMessage(messages.placeholderName)"
				autocomplete="off"
				wrapper-class="w-full"
			/>
		</div>
		<div class="flex flex-col gap-2">
			<label :for="addressId" class="font-semibold text-contrast">
				{{ formatMessage(messages.address) }}
			</label>
			<Input
				:id="addressId"
				v-model="address"
				:placeholder="formatMessage(messages.placeholderAddress)"
				autocomplete="off"
				:spellcheck="false"
				wrapper-class="w-full"
			/>
		</div>
		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">
				{{ formatMessage(messages.resourcePack) }}
			</span>
			<Combobox
				v-model="resourcePack"
				:options="resourcePackOptions"
				:display-value="formatMessage(resourcePackOptionMessages[resourcePack])"
			/>
		</div>
	</div>
</template>
