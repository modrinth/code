<script setup lang="ts">
import { PlayIcon, PlusIcon, XIcon } from '@modrinth/assets'
import { Button, commonMessages, defineMessages, NewModal, useVIntl } from '@modrinth/ui'
import { ref } from 'vue'

import type { ServerPackStatus } from '@/platform/app-lib/worlds/types'

import type { AddServerValues } from '../types'
import ServerForm from './ServerForm.vue'

const emit = defineEmits<{
	submit: [values: AddServerValues]
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'instance.add-server.title',
		defaultMessage: 'Add a server',
	},
	addServer: {
		id: 'instance.add-server.add-server',
		defaultMessage: 'Add server',
	},
	addAndPlay: {
		id: 'instance.add-server.add-and-play',
		defaultMessage: 'Add and play',
	},
})

const modal = ref<InstanceType<typeof NewModal> | null>(null)

const name = ref('')
const address = ref('')
const resourcePack = ref<ServerPackStatus>('enabled')

function submit(play: boolean) {
	emit('submit', {
		name: name.value || address.value,
		address: address.value,
		packStatus: resourcePack.value,
		play,
	})
	hide()
}

function show(event?: MouseEvent) {
	name.value = ''
	address.value = ''
	resourcePack.value = 'enabled'
	modal.value?.show(event)
}

function hide() {
	modal.value?.hide()
}

defineExpose({ show, hide })
</script>

<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.title)"
		width="min(500px, calc(95vw - 10rem))"
	>
		<ServerForm
			v-model:name="name"
			v-model:address="address"
			v-model:resource-pack="resourcePack"
		/>
		<template #actions>
			<div class="flex justify-end gap-2">
				<Button type="outlined" @click="hide">
					<XIcon aria-hidden="true" />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button :disabled="!address" @click="submit(false)">
					<PlusIcon aria-hidden="true" />
					{{ formatMessage(messages.addServer) }}
				</Button>
				<Button type="colored" color="brand" :disabled="!address" @click="submit(true)">
					<PlayIcon aria-hidden="true" />
					{{ formatMessage(messages.addAndPlay) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>
