<script setup lang="ts">
import { TrashIcon, XIcon } from '@modrinth/assets'
import {
	Admonition,
	ButtonStyled,
	commonMessages,
	defineMessages,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import type { World } from '@/helpers/worlds.ts'

const { formatMessage } = useVIntl()

const props = defineProps<{
	world: World | null
}>()

const emit = defineEmits<{
	confirm: [world: World]
}>()

const messages = defineMessages({
	removeServerTitle: {
		id: 'app.instance.worlds.remove-server-modal.title',
		defaultMessage: 'Remove server',
	},
	deleteWorldTitle: {
		id: 'app.instance.worlds.delete-world-modal.title',
		defaultMessage: 'Delete world',
	},
	removeServerWarningHeader: {
		id: 'app.instance.worlds.remove-server-modal.warning-header',
		defaultMessage: 'Removing {name}',
	},
	deleteWorldWarningHeader: {
		id: 'app.instance.worlds.delete-world-modal.warning-header',
		defaultMessage: 'Deleting {name}',
	},
	removeServerWarningBody: {
		id: 'app.instance.worlds.remove-server-modal.warning-body',
		defaultMessage:
			'This server will be removed from your server list and from the in-game server list. You can add it again later if you know the address.',
	},
	deleteWorldWarningBody: {
		id: 'app.instance.worlds.delete-world-modal.warning-body',
		defaultMessage:
			'This world will be permanently deleted from this instance. This action cannot be undone.',
	},
	removeServerButton: {
		id: 'app.instance.worlds.remove-server-modal.remove-button',
		defaultMessage: 'Remove server',
	},
	deleteWorldButton: {
		id: 'app.instance.worlds.delete-world-modal.delete-button',
		defaultMessage: 'Delete world',
	},
})

const modal = ref<InstanceType<typeof NewModal>>()

const isServer = computed(() => props.world?.type === 'server')
const isSingleplayer = computed(() => props.world?.type === 'singleplayer')
const titleMessage = computed(() =>
	isServer.value ? messages.removeServerTitle : messages.deleteWorldTitle,
)
const actionMessage = computed(() =>
	isServer.value ? messages.removeServerButton : messages.deleteWorldButton,
)
const warningHeaderMessage = computed(() =>
	isServer.value ? messages.removeServerWarningHeader : messages.deleteWorldWarningHeader,
)
const warningBodyMessage = computed(() =>
	isServer.value ? messages.removeServerWarningBody : messages.deleteWorldWarningBody,
)

function show() {
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

function confirm() {
	if (!props.world) return
	emit('confirm', props.world)
	hide()
}

defineExpose({ show, hide })
</script>

<template>
	<NewModal ref="modal" :header="formatMessage(titleMessage)" fade="danger" max-width="500px">
		<div class="flex flex-col gap-4">
			<Admonition
				type="critical"
				:header="formatMessage(warningHeaderMessage, { name: world?.name })"
			>
				{{ formatMessage(warningBodyMessage) }}
			</Admonition>
		</div>

		<template #actions>
			<div class="flex gap-2 justify-end">
				<ButtonStyled type="outlined">
					<button @click="hide">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="red">
					<button :disabled="!isServer && !isSingleplayer" @click="confirm">
						<TrashIcon />
						{{ formatMessage(actionMessage) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>
