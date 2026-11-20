<script setup lang="ts">
import { SaveIcon, XIcon } from '@modrinth/assets'
import { Button, commonMessages, defineMessage, NewModal, useVIntl } from '@modrinth/ui'
import { ref, shallowRef } from 'vue'

import type { ServerPackStatus, ServerWorld } from '@/platform/app-lib/worlds/types'

import type { EditServerValues } from '../types'
import HideFromHomeOption from './HideFromHomeOption.vue'
import ServerForm from './ServerForm.vue'

const emit = defineEmits<{
	submit: [values: EditServerValues]
}>()

const { formatMessage } = useVIntl()

const titleMessage = defineMessage({
	id: 'instance.edit-server.title',
	defaultMessage: 'Edit server',
})

const modal = ref<InstanceType<typeof NewModal> | null>(null)

const server = shallowRef<ServerWorld | null>(null)
const name = ref('')
const address = ref('')
const resourcePack = ref<ServerPackStatus>('enabled')
const hideFromHome = ref(false)

function submit() {
	if (!server.value) return
	emit('submit', {
		server: server.value,
		name: name.value || address.value,
		address: address.value,
		packStatus: resourcePack.value,
		displayStatus: hideFromHome.value ? 'hidden' : 'normal',
	})
	hide()
}

function show(value: ServerWorld, event?: MouseEvent) {
	server.value = value
	name.value = value.name
	address.value = value.address
	resourcePack.value = value.pack_status
	hideFromHome.value = value.display_status === 'hidden'
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
		:header="formatMessage(titleMessage)"
		width="min(500px, calc(95vw - 10rem))"
	>
		<div class="flex flex-col gap-3">
			<ServerForm
				v-model:name="name"
				v-model:address="address"
				v-model:resource-pack="resourcePack"
			/>
			<HideFromHomeOption v-model="hideFromHome" />
		</div>
		<template #actions>
			<div class="flex justify-end gap-2">
				<Button type="outlined" @click="hide">
					<XIcon aria-hidden="true" />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button type="colored" color="brand" :disabled="!address" @click="submit">
					<SaveIcon aria-hidden="true" />
					{{ formatMessage(commonMessages.saveChangesButton) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>
