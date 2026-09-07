<script setup lang="ts">
import { ShredderIcon, TrashIcon, XIcon } from '@modrinth/assets'
import {
	Admonition,
	Button,
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
	confirm: [world: World, scope: 'here' | 'all']
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
	syncedServerHeader: {
		id: 'app.instance.worlds.remove-server-modal.synced-header',
		defaultMessage: 'This server is synced',
	},
	syncedServerDescription: {
		id: 'app.instance.worlds.remove-server-modal.synced-description',
		defaultMessage:
			'You can remove it from just this instance or from all synced instances. Removing it from only this instance will enable overrides, and this instance will no longer receive synced server changes.',
	},
	removeHere: {
		id: 'app.instance.worlds.remove-server-modal.remove-here',
		defaultMessage: 'Remove here',
	},
	removeEverywhere: {
		id: 'app.instance.worlds.remove-server-modal.remove-everywhere',
		defaultMessage: 'Remove everywhere',
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
const isSyncedServer = computed(
	() => props.world?.type === 'server' && props.world.source === 'user_synced',
)
const isSingleplayer = computed(() => props.world?.type === 'singleplayer')
const titleMessage = computed(() =>
	isSyncedServer.value
		? messages.syncedServerHeader
		: isServer.value
			? messages.removeServerTitle
			: messages.deleteWorldTitle,
)
const actionMessage = computed(() =>
	isServer.value ? messages.removeServerButton : messages.deleteWorldButton,
)
const warningHeaderMessage = computed(() =>
	isSyncedServer.value
		? messages.syncedServerHeader
		: isServer.value
			? messages.removeServerWarningHeader
			: messages.deleteWorldWarningHeader,
)
const warningBodyMessage = computed(() =>
	isSyncedServer.value
		? messages.syncedServerDescription
		: isServer.value
			? messages.removeServerWarningBody
			: messages.deleteWorldWarningBody,
)

function show() {
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

function confirm(scope: 'here' | 'all') {
	if (!props.world) return
	emit('confirm', props.world, scope)
	hide()
}

defineExpose({ show, hide })
</script>

<template>
	<NewModal
		ref="modal"
		:header="formatMessage(titleMessage)"
		:fade="isSyncedServer ? 'warning' : 'danger'"
		max-width="560px"
		no-padding
	>
		<div class="flex flex-col gap-4 px-6 pt-6">
			<p v-if="isSyncedServer" class="m-0 text-primary">
				{{ formatMessage(warningBodyMessage) }}
			</p>
			<Admonition
				v-else
				:type="isSyncedServer ? 'warning' : 'critical'"
				:header="formatMessage(warningHeaderMessage, { name: world?.name })"
			>
				{{ formatMessage(warningBodyMessage) }}
			</Admonition>
		</div>

		<template #actions>
			<div class="flex flex-wrap justify-end gap-2">
				<Button type="outlined" @click="hide">
					<XIcon />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<template v-if="isSyncedServer">
					<Button type="outlined" color="orange" @click="confirm('all')">
						<ShredderIcon aria-hidden="true" />
						{{ formatMessage(messages.removeEverywhere) }}
					</Button>
					<Button type="colored" color="orange" @click="confirm('here')">
						<TrashIcon aria-hidden="true" />
						{{ formatMessage(messages.removeHere) }}
					</Button>
				</template>
				<Button
					v-else
					type="colored"
					color="red"
					:disabled="!isServer && !isSingleplayer"
					@click="confirm('all')"
				>
					<TrashIcon />
					{{ formatMessage(actionMessage) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>
