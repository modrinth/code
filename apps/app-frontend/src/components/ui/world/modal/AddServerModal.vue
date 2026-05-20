<script setup lang="ts">
import { PlayIcon, PlusIcon, XIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	commonMessages,
	defineMessages,
	injectNotificationManager,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { ref } from 'vue'

import ServerModalBody from '@/components/ui/world/modal/ServerModalBody.vue'
import type { GameInstance } from '@/helpers/types'
import { add_server_to_profile, type ServerPackStatus, type ServerWorld } from '@/helpers/worlds.ts'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()

const emit = defineEmits<{
	submit: [server: ServerWorld, play: boolean]
}>()

const props = defineProps<{
	instance: GameInstance
}>()

const modal = ref<InstanceType<typeof NewModal>>()

const name = ref('')
const address = ref('')
const resourcePack = ref<ServerPackStatus>('enabled')

async function addServer(play: boolean) {
	const serverName = name.value ? name.value : address.value
	const resourcePackStatus = resourcePack.value
	const index =
		(await add_server_to_profile(
			props.instance.path,
			serverName,
			address.value,
			resourcePackStatus,
		).catch(handleError)) ?? 0
	emit(
		'submit',
		{
			name: serverName,
			type: 'server',
			index,
			address: address.value,
			pack_status: resourcePackStatus,
		},
		play,
	)
	hide()
}

function show() {
	name.value = ''
	address.value = ''
	resourcePack.value = 'enabled'
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
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
	addAndPlay: {
		id: 'instance.add-server.add-and-play',
		defaultMessage: 'Add and play',
	},
})

defineExpose({ show, hide })
</script>
<template>
	<NewModal ref="modal" :header="formatMessage(messages.title)" width="500px" max-width="500px">
		<ServerModalBody
			v-model:name="name"
			v-model:address="address"
			v-model:resource-pack="resourcePack"
		/>
		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled type="outlined">
					<button @click="hide()">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled>
					<button :disabled="!address" @click="addServer(false)">
						<PlusIcon />
						{{ formatMessage(messages.addServer) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button :disabled="!address" @click="addServer(true)">
						<PlayIcon />
						{{ formatMessage(messages.addAndPlay) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>
