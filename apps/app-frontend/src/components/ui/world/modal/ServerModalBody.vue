<script setup lang="ts">
import {
	Combobox,
	defineMessages,
	type MessageDescriptor,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'

import type { ServerPackStatus } from '@/helpers/worlds.ts'

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
	placeholderAddress: {
		id: 'app.world.server-modal.placeholder-address',
		defaultMessage: 'example.modrinth.gg',
	},
	selectAnOption: {
		id: 'app.world.server-modal.select-an-option',
		defaultMessage: 'Select an option',
	},
})

defineExpose({ resourcePackOptions })
</script>
<template>
	<div class="w-[450px]">
		<h2 class="text-lg font-extrabold text-contrast mt-0 mb-1">
			{{ formatMessage(messages.name) }}
		</h2>
		<StyledInput
			v-model="name"
			:placeholder="formatMessage(messages.placeholderName)"
			autocomplete="off"
			wrapper-class="w-full"
		/>
		<h2 class="text-lg font-extrabold text-contrast mt-3 mb-1">
			{{ formatMessage(messages.address) }}
		</h2>
		<StyledInput
			v-model="address"
			:placeholder="formatMessage(messages.placeholderAddress)"
			autocomplete="off"
			wrapper-class="w-full"
		/>
		<h2 class="text-lg font-extrabold text-contrast mt-3 mb-1">
			{{ formatMessage(messages.resourcePack) }}
		</h2>
		<div>
			<Combobox
				v-model="resourcePack"
				:options="
					resourcePackOptions.map((o) => ({
						value: o,
						label: formatMessage(resourcePackOptionMessages[o]),
					}))
				"
				name="Server resource pack"
				:display-value="
					resourcePack
						? formatMessage(resourcePackOptionMessages[resourcePack])
						: formatMessage(messages.selectAnOption)
				"
			/>
		</div>
	</div>
</template>
